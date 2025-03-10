use std::{
    error::Error,
    fs,
    sync::{Arc, OnceLock},
};

use ctor::dtor;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::Bytes, Response};
use hyper::{Method, Request, StatusCode};
use jaem_user_discovery::user_data::UserStorage;
use serde_json::Value;
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

/// Test GET requests to get default page
#[tokio::test]
async fn get_users_default_page_success() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users", BASE_URI))
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

    let body = response.collect().await.unwrap().to_bytes();
    let binding = serde_json::from_slice::<Value>(&body).unwrap();
    let json = binding.as_array().unwrap();
    let size = json.len();

    assert_eq!(size, 20);

    let first_user = json[0].as_object().unwrap();
    let username = first_user.get("username").unwrap().as_str().unwrap();
    let description = first_user.get("description").unwrap().as_str().unwrap();
    let profile_pic = first_user.get("profile_picture").unwrap().as_str().unwrap();

    assert_eq!(username, "admin");
    assert_eq!(description, "Administrator");
    assert_eq!(profile_pic, "Im an Image\n");

    let last_user = json[size - 1].as_object().unwrap();
    let last_username = last_user.get("username").unwrap().as_str().unwrap();
    let last_description = last_user.get("description").unwrap().as_str().unwrap();
    let last_profile_pic = last_user.get("profile_picture").unwrap().as_str().unwrap();

    assert_eq!(last_username, "User 17");
    assert_eq!(last_description, "Additional User");
    assert_eq!(last_profile_pic, "Hello Im profile picture 1017\n");
}

#[tokio::test]
async fn get_users_from_10_to_14_success() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users/2/5", BASE_URI))
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

    let body = response.collect().await.unwrap().to_bytes();
    let binding = serde_json::from_slice::<Value>(&body).unwrap();
    let json = binding.as_array().unwrap();
    let size = json.len();

    assert_eq!(size, 5);

    let first_user = json[0].as_object().unwrap();
    let username = first_user.get("username").unwrap().as_str().unwrap();
    let description = first_user.get("description").unwrap().as_str().unwrap();
    let profile_pic = first_user.get("profile_picture").unwrap().as_str().unwrap();

    assert_eq!(username, "User 8");
    assert_eq!(description, "Additional User");
    assert_eq!(profile_pic, "1008\n");

    let last_user = json[4].as_object().unwrap();
    let last_username = last_user.get("username").unwrap().as_str().unwrap();
    let last_description = last_user.get("description").unwrap().as_str().unwrap();
    let last_profile_pic = last_user.get("profile_picture").unwrap().as_str().unwrap();

    assert_eq!(last_username, "User 12");
    assert_eq!(last_description, "Additional User");
    assert_eq!(last_profile_pic, "1012\n");
}

/// Test GET request to search by name
#[tokio::test]
async fn filter_by_name_success() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/search_users/1", BASE_URI))
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

    let body = response.collect().await.unwrap().to_bytes();
    let binding = serde_json::from_slice::<Value>(&body).unwrap();
    let json = binding.as_array().unwrap();
    let size = json.len();

    assert_eq!(size, 14);
}

#[tokio::test]
async fn filter_by_name_not_found() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/search_users/not_found", BASE_URI))
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
async fn filter_by_no_name_bad_request() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/search_users/", BASE_URI))
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

/// Test POST requests by adding user

#[tokio::test]
async fn add_user_success() {
    let body = r#"{"uid":"12", "username":"Hello", "public_keys":[{"algorithm":"ED25519", "signature_key":"test_sig","exchange_key":"test_ex","rsa_key":"test_rsa"}]}"#;
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
async fn add_user_without_pub_keys() {
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
async fn add_user_without_username() {
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
async fn add_user_without_body() {
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

/// Test DELETE request

#[tokio::test]
async fn delete_non_existing_pub_key_bad_request() {
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
async fn delete_pub_key_success() {
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
async fn delete_user_success() {
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
async fn delete_pub_key_bad_request() {
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
async fn delete_non_existing_user_bad_request() {
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
