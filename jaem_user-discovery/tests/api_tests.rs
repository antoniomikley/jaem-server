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
        fs::File::create("tests/temp_users.json").unwrap();
        fs::copy("tests/test_users.json", "tests/temp_users.json").unwrap();

        Arc::new(Mutex::new(
            UserStorage::read_from_file("tests/temp_users.json").unwrap(),
        ))
    })
}

#[dtor]
fn after_all_tests() {
    println!("✅ All tests finished!");
    let _ = fs::remove_file("test/temp_users.json");
}

/* GET users/{username} Expects: 200 OK */
/*--------Returns existing user---------*/
#[tokio::test]
async fn get_by_name_pattern_success() {
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("{}/users/Lennard%20Stubbe", BASE_URI))
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

/* GET users/{username} Expects: 200 OK */
/*----------Returns no user-------------*/
#[tokio::test]
async fn get_by_name_not_found_success() {
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

/* GET users/{username} Expects: 400 BAD REQUEST */
/*-------Fails because of missing username-------*/
#[tokio::test]
async fn get_without_name_bad_request() {
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

/* POST /create_user Expects: 200 OK */
/*---------Creates new user----------*/
#[tokio::test]
async fn create_new_user_success() {
    let body =
        r#"{"uid":"55","username":"ImNew","public_keys":[{"key":"new_key","algorithm":"AES"}]}"#;
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

/* POST /create_user Expects: 200 OK */
/*----Returns user already exists----*/
#[tokio::test]
async fn create_already_existing_user_bad_request() {
    let body = r#"{"uid":"777","username":"Max Mustermann","public_keys":[{"key":"new_key","algorithm":"AES"}]}"#;
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
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/* POST /add_pub_key Expects: 200 OK */
/*-----Returns public key addedd-----*/
#[tokio::test]
async fn add_one_pub_key_success() {
    let body =
        r#"{"uid":"1","username":"test","public_keys":[{"key":"new test","algorithm":"AES"}]}"#;
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
    assert_eq!(response.status(), StatusCode::OK);
}

/* POST /add_pub_key Expects: 200 OK */
/*-----Returns public key addedd-----*/
#[tokio::test]
async fn add_multiple_pub_key_success() {
    let body = r#"{"uid":"2","username":"test2","public_keys":[{"key":"hello test","algorithm":"AES"},{"key":"hello test2","algorithm":"AES"}]}"#;
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
    assert_eq!(response.status(), StatusCode::OK);
}

/* POST /add_pub_key Expects: 400 BAD REQUEST */
/*--------Returns public key missing----------*/
#[tokio::test]
async fn add_user_without_pub_keys() {
    let body = r#"{"uid":"313","username":"test"}"#;
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

/* POST /add_pub_key Expects: 400 BAD REQUEST */
/*-----Returns no user-----*/
#[tokio::test]
async fn add_user_without_user() {
    let body = r#"{public_keys":[{"key":"test","algorithm":"AES"}]}"#;
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

// Test DELETE request

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
