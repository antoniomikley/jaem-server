use std::{
    fs,
    sync::{Arc, OnceLock},
};

use ctor::dtor;
use hyper::{Method, Request, StatusCode};
use jaem_user_discovery::user_data::UserStorage;
use tokio::sync::Mutex;

const BASE_URI: &str = "http://127.0.0.1:8080";

fn get_users() -> &'static Arc<Mutex<UserStorage>> {
    static USERS: OnceLock<Arc<Mutex<UserStorage>>> = OnceLock::new();
    USERS.get_or_init(|| {
        let _ = fs::File::create("temp_users.json").unwrap();
        let _ = fs::copy("tests/test_users.json", "temp_users.json");

        Arc::new(Mutex::new(
            UserStorage::read_from_file("temp_users.json").unwrap(),
        ))
    })
}

#[dtor]
fn after_all_tests() {
    println!("✅ All tests finished!");
    let _ = fs::remove_file("temp_users.json");
}

// Test GET request to search by name

#[tokio::test]
async fn test_filter_by_name_success() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users/test", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();
    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_filter_by_name_not_found() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users/not_found", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();
    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_filter_by_no_name_bad_request() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users/", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();
    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Test POST requests by adding user

#[tokio::test]
async fn test_add_user_success() {
    let body =
        r#"{"uid":"12", "username":"Hello", "public_keys":[{"key":"test","algorithm":"ED25519"}]}"#;
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/create_user", BASE_URI))
        .body(body.to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_add_user_without_pub_keys() {
    let body = r#"{"username":"test"}"#;
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/add_pub_key", BASE_URI))
        .body(body.to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_user_without_username() {
    let body = r#"{"public_keys":[{"key":"test","algorithm":"ED25519"}]}"#;
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/add_pub_key", BASE_URI))
        .body(body.to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_add_user_without_body() {
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/add_pub_key", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Test DELETE request

#[tokio::test]
async fn test_delete_non_existing_pub_key_bad_request() {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/user/my_user/my_key", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}

#[tokio::test]
async fn test_delete_pub_key_success() {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/user/Lennard%20Stubbe/jaem-key123", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}

#[tokio::test]
async fn test_delete_user_success() {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/user/Max%20Mustermann", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}

#[tokio::test]
async fn test_delete_pub_key_bad_request() {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/user", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}

#[tokio::test]
async fn test_delete_non_existing_user_bad_request() {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("{}/user/test%20user", BASE_URI))
        .body("".to_string())
        .unwrap();

    let users = get_users();

    let response = jaem_user_discovery::handle_connection::handle_connection(
        request,
        users.clone(),
        "temp_users.json",
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST)
}
