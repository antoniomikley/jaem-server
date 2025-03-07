use std::{net::SocketAddr, sync::Arc};

use hyper::{server::conn::http1, service::service_fn};
use jaem_user_discovery::{handle_connection, user_data::UserStorage};
use tokio::sync::Mutex;

const USERS_FILE: &str = "users.json";

/*
 * Run Server on Port 3000
 * Used Address 0.0.0.0 for deploying with docker
 *
 * TODO: ConfigFile for Setting different Port, Address and USER_FILE path
*/

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Read users from file
    let users = UserStorage::read_from_file("users.json").unwrap();

    // Make user data mutex to avoid race conditions when accessing data
    let user_mutex = Arc::new(Mutex::new(users));

    // Main loop
    loop {
        // Listen on Port
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);

        // Clone the Arc to pass to new thread
        let user_mutex = Arc::clone(&user_mutex);

        // Spawn handle_connection task on new thread
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| {
                        handle_connection::handle_connection(req, user_mutex.clone(), USERS_FILE)
                    }),
                )
                .await
            {
                eprintln!("{}", err);
            }
        });
    }
}
