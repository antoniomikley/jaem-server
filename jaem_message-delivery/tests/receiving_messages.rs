use std::fs;

use hyper::{Request, StatusCode};
use jaem_config::JaemConfig;
use jaem_message_delivery::{request_handling::receive_messages, response_body::full};

#[tokio::test]
async fn sending_message_with_invalid_algorithm_byte() {
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config
        .set_storage_path("./receive_message_tests01")
        .unwrap();

    let input: Vec<u8> = vec![254; 33];
    let request = Request::builder().body(full(input)).unwrap();
    let response = receive_messages(request, &md_config).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Clean up
    fs::remove_dir_all("./receive_message_tests01").unwrap();
}

#[tokio::test]
async fn sending_too_short_public_key() {
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config
        .set_storage_path("./receive_message_tests02")
        .unwrap();

    let input: Vec<u8> = vec![0; 32];
    let request = Request::builder().body(full(input)).unwrap();
    let response = receive_messages(request, &md_config).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Clean up
    fs::remove_dir_all("./receive_message_tests02").unwrap();
}
#[tokio::test]
async fn reject_empty_messages() {
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config
        .set_storage_path("./receive_message_tests03")
        .unwrap();

    let input: Vec<u8> = vec![0; 33];
    let request = Request::builder().body(full(input)).unwrap();
    let response = receive_messages(request, &md_config).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Clean up
    fs::remove_dir_all("./receive_message_tests03").unwrap();
}
#[tokio::test]
async fn sending_valid_message() {
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config
        .set_storage_path("./receive_message_tests04")
        .unwrap();

    let input: Vec<u8> = vec![0; 34];
    let request = Request::builder().body(full(input)).unwrap();
    let response = receive_messages(request, &md_config).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Clean up
    fs::remove_dir_all("./receive_message_tests04").unwrap();
}
