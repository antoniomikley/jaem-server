use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{net::SocketAddr, str::FromStr};

use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::{Buf, Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};

async fn handle_request(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => Ok(Response::new(full("Hello World"))),
        (&Method::POST, "/send_message") => receive_messages(req).await,
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

async fn receive_messages(
    body: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut data = body.collect().await.unwrap().aggregate();
    let mut buf = Vec::new();
    while data.remaining() > 0 {
        buf.push(data.get_u8());
    }

    let signing_algorithm = match AlgoSign::from_repr(buf[0]) {
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

    let pub_key = buf[1..=signing_algorithm.get_key_len()].to_vec();

    let mut file: File = OpenOptions::new()
        .create(true)
        .append(true)
        .open(String::from_utf8(pub_key).expect("Seems like not all public keys are valid UTF-8."))
        .unwrap();

    file.write(&buf[signing_algorithm.get_key_len() + 1..])
        .expect("Couldn't write to file.");

    Ok(Response::new(empty()))
}

#[derive(FromRepr, EnumIter, Debug)]
#[repr(u8)]
enum AlgoSign {
    ED25519 = 0,
}

impl AlgoSign {
    pub fn get_key_len(&self) -> usize {
        match self {
            Self::ED25519 => return 32,
        }
    }

    pub fn list() -> String {
        let mut list = String::from("");
        for algo in AlgoSign::iter() {
            list.push_str(&format!("{:?}", algo));
            list.push_str(", ");
        }
        return list[0..list.len() - 2].to_string();
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from_str("127.0.0.1:8081").unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = hyper_util::rt::TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("{}", err);
            }
        });
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
