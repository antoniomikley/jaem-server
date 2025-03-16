use std::{env, net::SocketAddr, sync::Arc};

use hyper::{server::conn::http1, service::service_fn};
use jaem_user_discovery::handle_connection;
use tokio::sync::RwLock;

const PORT: u16 = 3000;

/*
 * Run Server on Port 3000
 * Used Address 0.0.0.0 for deploying with docker
 *
 * TODO: ConfigFile for Setting different Port, Address and USER_FILE path
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = env::var("POSTGRES_USER").unwrap_or("test_user".to_string());
    let password = env::var("POSTGRES_PASSWORD").unwrap_or("test_password".to_string());
    let db = env::var("POSTGRES_DB").unwrap_or("test_db".to_string());

    // Connect to database
    let connection_str = format!(
        "host=user-discovery-db port=5432 user={} password={} dbname={}",
        user, password, db
    );

    let (client, connection) =
        tokio_postgres::connect(&connection_str, tokio_postgres::NoTls).await?;

    // Spawn a new task to run connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let client = Arc::new(RwLock::new(client));

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    // Main loop
    loop {
        // Listen on Port
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);

        // Clone the Arc to pass to new thread
        let client_lock = Arc::clone(&client);

        // Spawn handle_connection task on new thread
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(|req| handle_connection::handle_connection(req, &client_lock)),
                )
                .await
            {
                eprintln!("{}", err);
            }
        });
    }
}
