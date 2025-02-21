use std::{net::SocketAddr, sync::Arc};

use hyper::{server::conn::http1, service::service_fn};
use jaem_user_discovery::{handle_connection, user_data::UserStorage};
use tokio::sync::Mutex;

const USERS_FILE: &str = "users.json";

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    let users = UserStorage::read_from_file("users.json").unwrap();
    let user_mutex = Arc::new(Mutex::new(users));

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);

        let user_mutex = Arc::clone(&user_mutex);
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
