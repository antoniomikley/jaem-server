use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
    usize,
};

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{Body, Bytes},
    Method, Request, Response, StatusCode,
};

use serde_json::Value;
use tokio::sync::Mutex;

use crate::user_data::{PubKey, PubKeyAlgo, UserData, UserStorage};

// Processes an incoming Request
pub async fn handle_connection<B: Body + Debug>(
    req: Request<B>,
    users: Arc<Mutex<UserStorage>>,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <B as Body>::Error: Debug,
{
    // Turn request uri into iterator and get first uri parameter
    let mut path_it = Path::new(req.uri().path()).iter();
    let _path_root = path_it.next().unwrap().to_str().unwrap();
    let path_resource = match path_it.next() {
        Some(resource) => resource.to_str().unwrap(),
        None => return Ok(bad_request("Resource cannot be empty")),
    };

    // Match first parameter (path_resource) to the corresponding implementation
    match (req.method(), path_resource) {
        (&Method::GET, "users") => {
            let page = match path_it.next() {
                Some(page) => page.to_str().unwrap().parse::<usize>().unwrap_or(0),
                None => 0,
            };
            let page_size = match path_it.next() {
                Some(page_size) => page_size.to_str().unwrap().parse::<usize>().unwrap_or(20),
                None => 20,
            };

            return get_users(page, page_size, users.lock().await.deref());
        }
        /*
         * Request: search_users/{username}
         * Return Users that match the pattern from {username}
         */
        (&Method::GET, "search_users") => {
            let name = match path_it.next() {
                Some(name) => name.to_str().unwrap(),
                None => return Ok(bad_request("Name cannot be empty")),
            };
            let page = match path_it.next() {
                Some(page) => page.to_str().unwrap().parse::<usize>().unwrap_or(0),
                None => 0,
            };
            let page_size = match path_it.next() {
                Some(page_size) => page_size.to_str().unwrap().parse::<usize>().unwrap_or(20),
                None => 20,
            };

            return get_user_by_name_pattern(
                name.to_string(),
                page,
                page_size,
                users.lock().await.deref(),
            );
        }

        /*
         * Request: user_by_uid/{uid}
         * Return User with specified uid
         */
        (&Method::GET, "user_by_uid") => {
            let key = match path_it.next() {
                Some(key) => key.to_str().unwrap(),
                None => return Ok(bad_request("Key cannot be empty")),
            };
            return get_user_by_uid(key.to_string(), users.lock().await.deref());
        }

        /*
         * Request: add_pub_key @Body -> uid + PubKey
         * Add PubKey to user with uid
         */
        (&Method::POST, "add_pub_key") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => {
                    return add_pub_keys(json, users.lock().await.deref_mut(), file_path);
                }
                Err(_) => {
                    let code = "0";
                    let message = "Invalid Request Body";
                    let response_body =
                        format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
                    return Ok(bad_request(&response_body));
                }
            }
        }

        /*
         * Request: create_user @Body -> UserData
         * Add UserData to user storage
         */
        (&Method::POST, "create_user") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => {
                    return add_new_entry(json, users.lock().await.deref_mut(), file_path);
                }
                Err(_) => {
                    let code = "0";
                    let message = "Invalid Request Body";
                    let response_body =
                        format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
                    return Ok(bad_request(&response_body));
                }
            }
        }

        /*
         * Request: set_profile_picture @Body -> uid + profile_picture
         * Change users profile picture
         */
        (&Method::PATCH, "profile") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => return change_profile(json, users.lock().await.deref_mut(), file_path),
                Err(_) => {
                    let code = "0";
                    let message = "Invalid Request Body";
                    let response_body =
                        format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
                    return Ok(bad_request(&response_body));
                }
            }
        }

        /*
         * Request: user/{uid} + Optional(/{signature_key})
         * Delete user from UDS
         */
        (&Method::DELETE, "user") => {
            let uid = match path_it.next() {
                Some(uid) => uid.to_str().unwrap(),
                None => return Ok(bad_request("UID cannot be empty")),
            };
            let signature_key = match path_it.next() {
                Some(public_key) => Some(public_key.to_str().unwrap()),
                None => None,
            };

            match signature_key.is_none() {
                false => {
                    let public_key = signature_key.unwrap();
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

fn get_users(
    page: usize,
    page_size: usize,
    users: &UserStorage,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let results = users.get_users(page, page_size);
    let json = serde_json::to_string(&results).unwrap();

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
    page: usize,
    page_size: usize,
    users: &UserStorage,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    if name.is_empty() {
        return Ok(bad_request("Name cannot be empty"));
    }

    let results = users.get_entries_by_pattern(name, page, page_size);
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
    if uid.is_empty() {
        return Ok(bad_request("UID cannot be empty"));
    }

    let result = match users.get_entry_by_uid(uid) {
        Some(user) => user,
        None => return Ok(bad_request("User not Found")),
    };

    let json = serde_json::to_string(&result).unwrap();

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
    let profile_picture = json["profile_picture"].clone();
    let description = json["description"].as_str().unwrap_or("");

    if uid.is_empty() {
        let code = "1";
        let message = "UID cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    if username.is_empty() {
        let code = "1";
        let message = "Username cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    if public_keys.is_none() {
        let code = "1";
        let message = "Public keys cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    let keys: Result<Vec<PubKey>, String> = public_keys
        .unwrap()
        .iter()
        .map(|key| parse_pubkey(key))
        .collect();

    let public_keys = match keys {
        Ok(k) => k,
        Err(missing_field) => {
            let code = "1";
            let message = missing_field;
            let response_body = format!("code: {}, message: '{}'", code, message);
            return Ok(bad_request(&response_body));
        }
    };

    let mut user_data = UserData {
        uid: uid.to_string(),
        username: username.to_string(),
        public_keys,
        profile_picture: profile_picture.as_str().unwrap_or("").parse().unwrap(),
        description: description.to_string(),
    };

    match users.add_entry(&mut user_data, file_path) {
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

fn parse_pubkey(key: &Value) -> Result<PubKey, String> {
    let algorithm = key["algorithm"]
        .as_str()
        .ok_or("Algorithm cannot be empty!")?;
    let signature_key = key["signature_key"]
        .as_str()
        .ok_or("Signature key missing!")?;
    let exchange_key = key["exchange_key"]
        .as_str()
        .ok_or("Exchange key missing!")?;
    let rsa_key = key["rsa_key"].as_str().ok_or("RSA key missing!")?;

    let algo_parsed = algorithm
        .parse::<PubKeyAlgo>()
        .map_err(|_| "algorithm (invalid value)")?;

    Ok(PubKey {
        algorithm: algo_parsed,
        signature_key: signature_key.to_string(),
        exchange_key: exchange_key.to_string(),
        rsa_key: rsa_key.to_string(),
    })
}

fn change_profile(
    json: Value,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uid = json["uid"].as_str().unwrap_or("");
    let username = json["username"].as_str().unwrap_or("");
    let profile_picture = json["profile_picture"].as_str().unwrap_or("");
    let description = json["description"].as_str().unwrap_or("");

    if uid.is_empty() {
        let code = "1";
        let message = "UID cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    match users.update_profile(
        uid.to_string(),
        username.to_string(),
        profile_picture.to_string(),
        description.to_string(),
        file_path,
    ) {
        Ok(_) => {
            let response_body = full("message: 'Profile updated'");
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

fn add_pub_keys(
    json: Value,
    users: &mut UserStorage,
    file_path: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let uid = json["uid"].as_str().unwrap_or("");
    let public_keys = json["public_keys"].as_array();

    if uid.is_empty() {
        let code = "1";
        let message = "UID cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    if public_keys.is_none() {
        let code = "1";
        let message = "Public keys cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    let public_keys = public_keys.unwrap();

    let pub_keys = public_keys
        .iter()
        .map(|key| {
            let key = key.as_object().unwrap();
            let algorithm = key["algorithm"].as_str().unwrap();
            let signature_key = key["signature_key"].as_str().unwrap();
            let exchange_key = key["exchange_key"].as_str().unwrap();
            let rsa_key = key["rsa_key"].as_str().unwrap();

            PubKey {
                algorithm: algorithm.parse::<PubKeyAlgo>().unwrap(),
                signature_key: signature_key.to_string(),
                exchange_key: exchange_key.to_string(),
                rsa_key: rsa_key.to_string(),
            }
        })
        .collect();

    match users.add_pub_keys(uid.to_string(), pub_keys, file_path) {
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
            let code = "2";
            let message = "User not found";
            let response_body = format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
            return Ok(bad_request(&response_body));
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
        Err(err) => {
            return Ok(bad_request(&err.to_string()));
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
