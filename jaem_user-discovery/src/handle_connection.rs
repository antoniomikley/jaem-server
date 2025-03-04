use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{Body, Bytes},
    Method, Request, Response, StatusCode,
};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::user_data::{PubKey, PubKeyAlgo, UserData, UserStorage};

pub async fn handle_connection<B: Body + Debug>(
    req: Request<B>,
    users: Arc<Mutex<UserStorage>>,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <B as Body>::Error: Debug,
{
    let mut path_it = Path::new(req.uri().path()).iter();
    let _path_root = path_it.next().unwrap().to_str().unwrap();
    let path_resource = match path_it.next() {
        Some(resource) => resource.to_str().unwrap(),
        None => return Ok(bad_request("Resource cannot be empty")),
    };
    match (req.method(), path_resource) {
        (&Method::GET, "users") => {
            let name = match path_it.next() {
                Some(name) => name.to_str().unwrap(),
                None => return Ok(bad_request("Name cannot be empty")),
            };
            return get_user_by_name_pattern(name.to_string(), users.lock().await.deref());
        }
        (&Method::GET, "user") => {
            let name = match path_it.next() {
                Some(name) => name.to_str().unwrap(),
                None => return Ok(bad_request("Name cannot be empty")),
            };
            return get_user_by_name(name.to_string(), users.lock().await.deref());
        }
        (&Method::GET, "user_by_uid") => {
            let key = match path_it.next() {
                Some(key) => key.to_str().unwrap(),
                None => return Ok(bad_request("Key cannot be empty")),
            };
            return get_user_by_uid(key.to_string(), users.lock().await.deref());
        }
        (&Method::POST, "add_pub_key") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => {
                    return add_pub_keys(json, users.lock().await.deref_mut(), file_path);
                }
                Err(_) => return Ok(bad_request("Invalid Request Body")),
            }
        }
        (&Method::POST, "create_user") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => {
                    return add_new_entry(json, users.lock().await.deref_mut(), file_path);
                }
                Err(_) => return Ok(bad_request("Invalid Request Body")),
            }
        }
        (&Method::DELETE, "user") => {
            let uid = match path_it.next() {
                Some(uid) => uid.to_str().unwrap(),
                None => return Ok(bad_request("UID cannot be empty")),
            };
            let public_key = match path_it.next() {
                Some(public_key) => Some(public_key.to_str().unwrap()),
                None => None,
            };

            match public_key.is_none() {
                false => {
                    let public_key = public_key.unwrap();
                    return delete_pub_key_from_user(
                        uid.to_string(),
                        public_key.to_string(),
                        users.lock().await.deref_mut(),
                        file_path,
                    );
                }
                true => {
                    return delete_user(uid.to_string(), users.lock().await.deref_mut(), file_path);
                }
            }
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            return Ok(not_found);
        }
    }
}

fn get_user_by_name(
    name: String,
    users: &UserStorage,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if name.is_empty() {
        return Ok(bad_request("Name cannot be empty"));
    }

    let result = users.get_entry(name);
    let json = serde_json::to_string(&result).unwrap();

    let body: BoxBody<Bytes, hyper::Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

fn get_user_by_name_pattern(
    name: String,
    users: &UserStorage,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if name.is_empty() {
        return Ok(bad_request("Name cannot be empty"));
    }

    let results = users.get_entries_by_pattern(name);
    let json = serde_json::to_string(&results).unwrap();

    let body: BoxBody<Bytes, hyper::Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

fn get_user_by_uid(
    uid: String,
    users: &UserStorage,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let results = users.get_entry_by_uid(uid);
    let json = serde_json::to_string(&results).unwrap();

    let body: BoxBody<Bytes, hyper::Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

fn add_new_entry(
    json: Value,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uid = json["uid"].as_str().unwrap_or("");
    let username = json["username"].as_str().unwrap_or("");
    let public_keys = json["public_keys"].as_array();

    if uid.is_empty() {
        return Ok(bad_request("UID cannot be empty"));
    }

    if username.is_empty() {
        return Ok(bad_request("Username cannot be empty"));
    }

    if public_keys.is_none() {
        return Ok(bad_request("Public keys cannot be empty"));
    }

    let public_keys = public_keys.unwrap();

    let user_data = UserData {
        uid: uid.to_string(),
        username: username.to_string(),
        public_keys: public_keys
            .iter()
            .map(|key| {
                let key = key.as_object().unwrap();
                let algorithm = key["algorithm"].as_str().unwrap();
                let key = key["key"].as_str().unwrap();
                PubKey {
                    algorithm: algorithm.parse::<PubKeyAlgo>().unwrap(),
                    key: key.to_string(),
                }
            })
            .collect(),
    };

    match users.add_entry(user_data, file_path) {
        Ok(_) => {
            let response_body = full("message: 'User added'");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap();
            return Ok(response);
        }
        Err(_) => {
            return Ok(bad_request("User already exists"));
        }
    }
}

fn add_pub_keys(
    json: Value,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uid = json["uid"].as_str().unwrap_or("");
    let username = json["username"].as_str().unwrap_or("");
    let public_keys = json["public_keys"].as_array();

    if uid.is_empty() {
        return Ok(bad_request("UID cannot be empty"));
    }
    if username.is_empty() {
        return Ok(bad_request("Username cannot be empty"));
    }

    if public_keys.is_none() {
        return Ok(bad_request("Public keys cannot be empty"));
    }

    let public_keys = public_keys.unwrap();

    let user_data = UserData {
        uid: uid.to_string(),
        username: username.to_string(),
        public_keys: public_keys
            .iter()
            .map(|key| {
                let key = key.as_object().unwrap();
                let algorithm = key["algorithm"].as_str().unwrap();
                let key = key["key"].as_str().unwrap();
                PubKey {
                    algorithm: algorithm.parse::<PubKeyAlgo>().unwrap(),
                    key: key.to_string(),
                }
            })
            .collect(),
    };

    match users.add_pub_keys(user_data, file_path) {
        Ok(_) => {
            let response_body = full("message: 'Public keys added'");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap();
            return Ok(response);
        }
        Err(_) => {
            return Ok(bad_request("User not found"));
        }
    }
}

fn delete_user(
    uid: String,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match users.delete_entry(uid, file_path) {
        Ok(_) => {
            let response_body = full("message: 'User deleted'");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap();
            return Ok(response);
        }
        Err(_) => {
            return Ok(bad_request("User not found"));
        }
    }
}

fn delete_pub_key_from_user(
    uid: String,
    public_key: String,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match users.delete_pub_key(uid, public_key, file_path) {
        Ok(_) => {
            let response_body = full("message: 'Public key deleted'");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap();
            return Ok(response);
        }
        Err(_) => {
            return Ok(bad_request("User or public key not found"));
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(data: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(data.into())
        .map_err(|never| match never {})
        .boxed()
}

fn bad_request(message: &str) -> Response<BoxBody<Bytes, hyper::Error>> {
    let body: BoxBody<Bytes, hyper::Error> = full(Bytes::from(message.to_string()));
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap()
}
