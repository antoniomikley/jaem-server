use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
    usize,
};

use anyhow::Error;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{
    body::{Body, Bytes},
    Method, Request, Response, StatusCode,
};

use serde_json::Value;
use tokio::sync::RwLock;
use tokio_postgres::Client;

use crate::{
    database::Database,
    user_data::{PubKey, PubKeyAlgo, UserData},
};

// Processes an incoming Request
pub async fn handle_connection<B: Body + Debug>(
    req: Request<B>,
    db_client: &Arc<RwLock<Client>>,
) -> Result<Response<BoxBody<Bytes, Error>>, Error>
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

            return get_users(db_client.read().await.deref(), page, page_size).await;
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

            return get_users_by_name_pattern(
                db_client.read().await.deref(),
                name.to_string(),
                page,
                page_size,
            )
            .await;
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
            return get_user_by_uid(db_client.read().await.deref(), key.to_string()).await;
        }

        /*
         * Request: add_pub_key @Body -> uid + PubKey
         * Add PubKey to user with uid
         */
        (&Method::POST, "add_pub_key") => {
            let body_bytes = req.collect().await.unwrap().to_bytes();
            match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => {
                    return add_pub_keys(db_client.write().await.deref_mut(), json).await;
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
                    return add_new_entry(db_client.write().await.deref_mut(), json).await;
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
                Ok(json) => {
                    return change_user_data(db_client.write().await.deref_mut(), json).await
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
                        db_client.write().await.deref_mut(),
                        uid.to_string(),
                        public_key.to_string(),
                    )
                    .await;
                }
                true => {
                    return delete_user(db_client.write().await.deref_mut(), uid.to_string()).await;
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

async fn get_users(
    db: &Client,
    page: usize,
    page_size: usize,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let results = Database::get_users(db, page, page_size)
        .await
        .map_err(Error::from)?;

    let json = serde_json::to_string(&results).unwrap();

    let body: BoxBody<Bytes, Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

async fn get_users_by_name_pattern(
    db: &Client,
    name: String,
    page: usize,
    page_size: usize,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    if name.is_empty() {
        return Ok(bad_request("Name cannot be empty"));
    }

    let results = Database::get_entries_by_pattern(db, &name, page, page_size)
        .await
        .map_err(Error::from)?;
    let json = serde_json::to_string(&results).unwrap();

    let body: BoxBody<Bytes, Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

async fn get_user_by_uid(
    db: &Client,
    uid: String,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    if uid.is_empty() {
        return Ok(bad_request("UID cannot be empty"));
    }

    let result = match Database::get_entry_by_uid(db, uid)
        .await
        .map_err(Error::from)?
    {
        Some(user) => user,
        None => return Ok(bad_request("User not Found")),
    };

    let json = serde_json::to_string(&result).unwrap();

    let body: BoxBody<Bytes, Error> = full(Bytes::from(json));

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap();

    Ok(response)
}

async fn add_new_entry(db: &Client, json: Value) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
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

    let user_data = UserData {
        uid: uid.to_string(),
        username: username.to_string(),
        public_keys,
        profile_picture: profile_picture.as_str().unwrap_or("").parse().unwrap(),
        description: description.to_string(),
    };

    match Database::add_new_entry(db, user_data).await {
        Ok(_) => {
            let response_body = full("message: 'User added'");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(response_body)
                .unwrap();
            return Ok(response);
        }
        Err(err) => {
            let code = "2";
            let message = err.to_string();
            let response_body = format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
            return Ok(bad_request(&response_body));
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

async fn change_user_data(
    db: &Client,
    json: Value,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    let uid = json["uid"].as_str().unwrap_or("");
    let username = json["username"].as_str();
    let profile_picture = json["profile_picture"].as_str();
    let description = json["description"].as_str();

    if uid.is_empty() {
        let code = "1";
        let message = "UID cannot be empty";
        let response_body = format!("code: {}, message: '{}'", code, message);
        return Ok(bad_request(&response_body));
    }

    match Database::update_users(db, uid.to_string(), username, profile_picture, description).await
    {
        Ok(_) => {
            let response_body = full("message: 'User data updated'");
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

async fn add_pub_keys(db: &Client, json: Value) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
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

    match Database::add_pub_keys(db, uid.to_string(), &public_keys).await {
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

async fn delete_user(db: &Client, uid: String) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    match Database::delete_entry(db, uid).await {
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

async fn delete_pub_key_from_user(
    db: &Client,
    uid: String,
    public_key: String,
) -> Result<Response<BoxBody<Bytes, Error>>, Error> {
    match Database::delete_pub_key(db, uid, &public_key).await {
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

fn empty() -> BoxBody<Bytes, Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(data: T) -> BoxBody<Bytes, Error> {
    Full::new(data.into())
        .map_err(|never| match never {})
        .boxed()
}

fn bad_request(message: &str) -> Response<BoxBody<Bytes, Error>> {
    let body: BoxBody<Bytes, Error> = full(Bytes::from(message.to_string()));
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap()
}
