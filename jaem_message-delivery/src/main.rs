use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use std::{net::SocketAddr, str::FromStr};

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use jaem_config::{JaemConfig, MessageDeliveryConfig, DEFAULT_CONFIG_PATH};
use jaem_message_delivery::message_deletion::{remove_expired_deletions, OutstandingDeletion};
use jaem_message_delivery::request_handling::{
    delete_messages, receive_messages, retrieve_messages,
};
use jaem_message_delivery::response_body::empty;

async fn handle_request(
    req: Request<Incoming>,
    config: &MessageDeliveryConfig,
    outstanding_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/send_message") => Ok(receive_messages(req, config).await?),
        (&Method::POST, "/get_messages") => {
            Ok(retrieve_messages(req, config, outstanding_deletions).await?)
        }
        (&Method::POST, "/delete_messaages") => {
            Ok(delete_messages(req, config, outstanding_deletions).await?)
        }
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() {
    let outstanding_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let addr = SocketAddr::from_str("127.0.0.1:8081").unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let global_config = match JaemConfig::read_from_file(DEFAULT_CONFIG_PATH) {
        Ok(config) => config,
        Err(_) => {
            let config = JaemConfig::create_default();
            config.save_to_file(DEFAULT_CONFIG_PATH).unwrap();
            config
        }
    };
    let global_config = Arc::new(global_config);
    loop {
        let outstanding_deletions_mv = Arc::clone(&outstanding_deletions);
        let global_config_mv = Arc::clone(&global_config);
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        tokio::task::spawn(async move {
            let config = &global_config_mv.message_delivery_config.clone().unwrap();
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| {
                        handle_request(req, &config, outstanding_deletions_mv.clone())
                    }),
                )
                .await
            {
                eprintln!("{}", err);
            }
        });

        let current_time = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        remove_expired_deletions(&mut outstanding_deletions.lock().unwrap(), current_time)
    }
}
