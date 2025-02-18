use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use serde_json::Value;
use urlencoding::decode;
use UserDiscoveryService::user_data::{UserData, UserStorage};

fn main() {
    let mut users = UserStorage::read_from_file("users.json").unwrap();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream, &mut users);
            }
            Err(e) => {
                eprintln!("An error occurred while accepting a connection: {}", e);
                continue;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, users: &mut UserStorage) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();
    let buffer = String::from_utf8_lossy(&buffer);
    let request_line = buffer.lines().next().unwrap();
    let method = request_line.split_whitespace().nth(0).unwrap();
    let request_uri = request_line.split_whitespace().nth(1).unwrap();

    let request_path = request_uri.split('?').nth(0).unwrap_or_else(|| request_uri);

    if method == "GET" {
        let query_string = request_uri.split('?').nth(1).unwrap();
        if request_path == "/search_by_name" && query_string.starts_with("username=") {
            let username =
                decode(query_string.split("=").nth(1).unwrap()).expect("Failed to decode username");

            println!("Username: {}", username);

            match users.get_entries_by_pattern(&username) {
                Some(results) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\n\r\n{}",
                        serde_json::to_string(&results).unwrap()
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                    println!("Users sent successfully");
                }
                None => {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        }
    } else if method == "POST" {
        let parts: Vec<&str> = buffer.split("\r\n\r\n").collect();
        if parts.len() < 2 {
            let response = "HTTP/1.1 400 Bad Request\r\n\r\nMissing body";
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }

        if request_path == "/add_pub_key" {
            let body = parts[1].trim().trim_end_matches('\0');
            match serde_json::from_str::<Value>(body) {
                Ok(json) => {
                    let data: UserData = match serde_json::from_value(json) {
                        Ok(data) => data,
                        Err(_) => {
                            send_error(&mut stream);
                            return;
                        }
                    };
                    match users.add_pub_keys(data) {
                        Ok(_) => {
                            println!("{:?}", users)
                        }
                        Err(_) => {
                            send_error(&mut stream);
                            return;
                        }
                    }

                    let response = "HTTP/1.1 200 OK\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    println!("User added successfully");
                }
                Err(error) => {
                    println!("Error: {:?}", error);
                    let response = "HTTP/1.1 400 Bad Request\r\n\r\nInvalid JSON";
                    stream.write_all(response.as_bytes()).unwrap();
                    return;
                }
            }
        }
    } else if method == "DELETE" {
        let query_string = request_uri.split('?').nth(1).unwrap();
        let query_params = query_string.split('&').collect::<Vec<&str>>();
        if request_path == "/delete_pub_key"
            && query_string.contains("username=")
            && query_string.contains("public_key=")
            && query_params.len() == 2
        {
            let username = query_params
                .iter()
                .find(|&x| x.contains("username="))
                .unwrap()
                .split("=")
                .nth(1)
                .unwrap();
            let public_key = query_params
                .iter()
                .find(|&x| x.contains("public_key="))
                .unwrap()
                .split("=")
                .nth(1)
                .unwrap();

            let username = decode(username).expect("Failed to decode username");
            let public_key = decode(public_key).expect("Failed to decode public key");

            match users.delete_pub_key(username.to_string(), public_key.to_string()) {
                Ok(_) => {
                    let response = "HTTP/1.1 200 OK\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    println!("Public key deleted successfully");
                }
                Err(_) => {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        } else if request_path == "/delete_user" && query_string.starts_with("username=") {
            let username = query_params
                .iter()
                .find(|&x| x.contains("username="))
                .unwrap()
                .split("=")
                .nth(1)
                .unwrap();

            let username = decode(username).expect("Failed to decode username");

            match users.delete_entry(username.to_string()) {
                Ok(_) => {
                    let response = "HTTP/1.1 200 OK\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                    println!("User deleted successfully");
                }
                Err(_) => {
                    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
        }
    }
}

fn send_error(stream: &mut TcpStream) {
    let response = "HTTP/1.1 400 Bad Request\r\n\r\nInvalid JSON";
    stream.write_all(response.as_bytes()).unwrap();
}
