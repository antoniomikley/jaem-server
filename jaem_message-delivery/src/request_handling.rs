use crate::{
    authentication::AuthProof,
    message_deletion::OutstandingDeletion,
    response_body::{empty, full},
    share_link::ShareLink,
    sign_algos::AlgoSign,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{Body, Buf, Bytes},
    Request, Response, StatusCode,
};
use jaem_config::MessageDeliveryConfig;
use std::{
    collections::HashMap,
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{Read, Write},
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

pub async fn body_as_vec<T: Body + Debug>(body: Request<T>) -> Vec<u8>
where
    <T as Body>::Error: Debug,
{
    let mut data = body.collect().await.unwrap().aggregate();
    let mut buf = Vec::new();
    while data.remaining() > 0 {
        buf.push(data.get_u8());
    }
    return buf;
}

pub async fn retrieve_messages<T: Body + Debug>(
    body: Request<T>,
    config: &MessageDeliveryConfig,
    outstanding_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <T as Body>::Error: Debug,
{
    let body = body_as_vec(body).await;
    let auth_proof = match AuthProof::new(&body) {
        Ok(auth_proof) => auth_proof,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full(e.to_string()))
                .unwrap())
        }
    };

    match auth_proof.verify() {
        Ok(true) => {
            let mut file_path = config.storage_path.clone();
            file_path.push(URL_SAFE.encode(auth_proof.pub_key.as_slice()));
            let mut file = match File::open(file_path) {
                Ok(file) => file,
                Err(_) => return Ok(Response::builder().body(empty()).unwrap()),
            };
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            let delete_later =
                OutstandingDeletion::new(auth_proof.current_time, &auth_proof.pub_key);
            let mut outstanding_deletions = outstanding_deletions.lock().unwrap();
            outstanding_deletions.insert(auth_proof.pub_key, delete_later);
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full(buffer))
                .unwrap());
        }
        Ok(false) => {
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(full("Signature or timestamp are invalid."))
                .unwrap())
        }
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full("The provided key is not valid."))
                .unwrap())
        }
    }
}

pub async fn receive_messages<T: Body + Debug>(
    body: Request<T>,
    config: &MessageDeliveryConfig,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <T as Body>::Error: Debug,
{
    let body = body_as_vec(body).await;

    let signing_algorithm = match AlgoSign::from_repr(body[0]) {
        Some(algo) => algo,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full(format!(
                "The specified signing algorithm is not supported. Currently supported are: \n{}\n",
                AlgoSign::list()
            )))
                .unwrap())
        }
    };
    if body.len() <= signing_algorithm.get_key_len() + 1 {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(full("The given key is too short."))
            .unwrap());
    }

    let pub_key = body[1..=signing_algorithm.get_key_len()].to_vec();
    let encoded_pub_key = URL_SAFE.encode(pub_key.as_slice());
    let mut file_path = config.storage_path.clone();
    file_path.push(encoded_pub_key);

    let mut file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
        .unwrap();

    let message = &body[signing_algorithm.get_key_len() + 1..];
    file.write(&message.len().to_be_bytes())
        .expect("could not write to file.");
    file.write(&message).expect("could not write to file.");

    Ok(Response::new(empty()))
}

pub async fn delete_messages<T: Body + Debug>(
    body: Request<T>,
    config: &MessageDeliveryConfig,
    outstanding_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <T as Body>::Error: Debug,
{
    let body = body_as_vec(body).await;
    let auth_proof = match AuthProof::new(&body) {
        Ok(auth_proof) => auth_proof,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full(e.to_string()))
                .unwrap())
        }
    };
    match auth_proof.verify() {
        Ok(true) => {
            let file_path = config.storage_path.clone();
            let mut outstanding_deletions = outstanding_deletions.lock().unwrap();
            let deletion = match outstanding_deletions.get(&auth_proof.pub_key) {
                Some(del) => del,
                None => {
                    return Ok(Response::builder()
                        .status(StatusCode::CONFLICT)
                        .body(full("You cannot delete unretrieved messages."))
                        .unwrap())
                }
            };
            match deletion.delete(file_path) {
                Err(_) => {
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(full("Could not delete Messages."))
                        .unwrap())
                }
                Ok(_) => {}
            }

            outstanding_deletions
                .remove(&auth_proof.pub_key)
                .expect("Deleted Messages, but could not remove from deletion queue.");

            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full("Messages deleted"))
                .unwrap());
        }
        Ok(false) => {
            return Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(full("Invalid signature."))
                .unwrap())
        }
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(full("The provided key is not valid."))
                .unwrap())
        }
    }
}

pub async fn share_data<T: Body + Debug>(
    body: Request<T>,
    config: &MessageDeliveryConfig,
    share_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <T as Body>::Error: Debug,
{
    let share_link_gen = ShareLink::new();
    let mut share_link = share_link_gen.generate_link();
    let mut path = config.share_directory.clone();
    path.push(share_link.clone());

    while path.exists() {
        share_link = share_link_gen.generate_link();
        path = config.share_directory.clone();
        path.push(share_link.clone());
    }

    let mut share_file = match File::create(path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Could not create share file."))
                .unwrap())
        }
    };

    let req_body = body_as_vec(body).await;
    match share_file.write_all(&req_body) {
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Could not write to share file."))
                .unwrap())
        }
        Ok(_) => {
            let current_time = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut share_deletions = share_deletions.lock().unwrap();
            let share_deletion = OutstandingDeletion::new(current_time, share_link.as_bytes());
            share_deletions.insert(share_link.clone().into_bytes(), share_deletion);

            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full(share_link))
                .unwrap());
        }
    };
}
pub async fn get_shared_data<T: Body + Debug>(
    req: Request<T>,
    config: &MessageDeliveryConfig,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>
where
    <T as Body>::Error: Debug,
{
    // cut of the /share/ part of the path
    let uri = &req.uri().path()[7..];
    if uri.contains('/') {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(empty())
            .unwrap());
    }

    let mut path = config.share_directory.clone();
    path.push(uri);

    let mut share_file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(empty())
                .unwrap())
        }
    };

    let mut buf = Vec::new();
    match share_file.read_to_end(&mut buf) {
        Ok(_) => {}
        Err(_) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Could not read from file"))
                .unwrap())
        }
    }

    return Ok(Response::builder()
        .status(StatusCode::OK)
        .body(full(buf))
        .unwrap());
}
