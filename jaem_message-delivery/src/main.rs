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
use jaem_message_delivery::message_deletion::{
    delete_expired_deletions, remove_expired_deletions, OutstandingDeletion,
};
use jaem_message_delivery::request_handling::{
    delete_messages, get_shared_data, receive_messages, retrieve_messages, share_data,
};
use jaem_message_delivery::response_body::empty;

/// Route the requests to the correct functoin to deal with them.
async fn handle_request(
    req: Request<Incoming>,
    config: &MessageDeliveryConfig,
    message_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
    share_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/send_message") => Ok(receive_messages(req, config).await?),
        (&Method::POST, "/get_messages") => {
            Ok(retrieve_messages(req, config, message_deletions).await?)
        }
        (&Method::POST, "/delete_messages") => {
            Ok(delete_messages(req, config, message_deletions).await?)
        }
        (&Method::POST, "/share") => Ok(share_data(req, config, share_deletions).await?),
        _ => {
            if req.method() == &Method::GET && req.uri().path().starts_with("/share/") {
                return Ok(get_shared_data(req, config).await?);
            }
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() {
    // create ressources that are shared between threads
    let message_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let share_deletions: Arc<Mutex<HashMap<Vec<u8>, OutstandingDeletion>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // load application configuration from file. create a new one if it does not exist.
    let global_config = match JaemConfig::read_from_file(DEFAULT_CONFIG_PATH) {
        Ok(config) => config,
        Err(_) => {
            let config = JaemConfig::create_default();
            config.save_to_file(DEFAULT_CONFIG_PATH).unwrap();
            config
        }
    };

    // create the necessary directories.
    let mut md_config = global_config.message_delivery_config.clone().unwrap();
    md_config
        .create_dirs()
        .expect("Could not create necessary directories.");

    let addr =
        SocketAddr::from_str(format!("{}:{}", md_config.address, md_config.port).as_str()).unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let global_config = Arc::new(global_config);

    loop {
        let message_deletions_mv = Arc::clone(&message_deletions);
        let share_deletions_mv = Arc::clone(&share_deletions);
        let global_config_mv = Arc::clone(&global_config);
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        tokio::task::spawn(async move {
            let config = &global_config_mv.message_delivery_config.clone().unwrap();
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| {
                        handle_request(
                            req,
                            &config,
                            message_deletions_mv.clone(),
                            share_deletions_mv.clone(),
                        )
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

        // remove staged deletions of outstanding message deletoins after 20 seconds.
        remove_expired_deletions(&mut message_deletions.lock().unwrap(), current_time, 20);
        // delete shared data older than 10 minutes.
        delete_expired_deletions(
            &mut share_deletions.lock().unwrap(),
            current_time,
            600,
            global_config.get_message_delivery_config().share_directory,
        )
    }
}
