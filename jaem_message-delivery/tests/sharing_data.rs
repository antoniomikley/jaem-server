use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::Request;
use hyper::StatusCode;
use jaem_config::JaemConfig;
use jaem_message_delivery::request_handling::get_shared_data;
use jaem_message_delivery::request_handling::share_data;
use jaem_message_delivery::response_body::empty;
use jaem_message_delivery::response_body::full;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[tokio::test]
async fn successful_share() {
    let test_dir = "./share_data_tests01";
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config.set_share_dir(test_dir).unwrap();

    // Construct a test data to share and send it
    let mut test_message = Vec::new();
    test_message.append(&mut "test_data".as_bytes().to_vec());
    let share_request = Request::builder().body(full(test_message.clone())).unwrap();
    let deletions = Arc::new(Mutex::new(HashMap::new()));
    let share_response = share_data(share_request, &md_config, deletions.clone())
        .await
        .unwrap();

    assert_eq!(StatusCode::OK, share_response.status());
    // Share data should be staged for deletion
    assert_eq!(1, deletions.lock().unwrap().len());

    // check response body for the link
    let mut buf = Vec::new();
    let mut response_body = share_response.boxed().collect().await.unwrap().aggregate();
    while response_body.has_remaining() {
        buf.push(response_body.get_u8());
    }
    // Retrieve shared data
    let get_shared_req = Request::builder()
        .uri(format!("/share/{}", String::from_utf8(buf).unwrap()))
        .body(empty())
        .unwrap();
    let share_data_response = get_shared_data(get_shared_req, &md_config).await.unwrap();

    // check response body to be the test share data
    let mut buf = Vec::new();
    let mut response_body = share_data_response
        .boxed()
        .collect()
        .await
        .unwrap()
        .aggregate();
    while response_body.has_remaining() {
        buf.push(response_body.get_u8());
    }

    assert_eq!(
        String::from_utf8(test_message).unwrap(),
        String::from_utf8(buf).unwrap()
    );

    // Clean up
    std::fs::remove_dir_all(test_dir).unwrap();
}
