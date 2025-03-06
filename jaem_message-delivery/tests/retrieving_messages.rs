use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

use ed25519_dalek::ed25519::signature::SignerMut;
use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::Request;
use hyper::StatusCode;
use jaem_config::JaemConfig;
use jaem_message_delivery::request_handling::receive_messages;
use jaem_message_delivery::request_handling::retrieve_messages;
use jaem_message_delivery::response_body::full;
use rand::rngs::OsRng;

#[tokio::test]
async fn successful_retrieval() {
    let test_dir = "./retrieve_messages_tests01";
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config.set_storage_path(test_dir).unwrap();

    let mut csprng = OsRng;
    let mut signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    let mut test_message = Vec::new();
    test_message.push(0);
    test_message.append(&mut verifying_key.as_bytes().to_vec());
    test_message.append(&mut "test_message".as_bytes().to_vec());
    let send_request = Request::builder().body(full(test_message)).unwrap();
    receive_messages(send_request, &md_config).await.unwrap();

    let mut auth_proof = Vec::new();
    let timestamp = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_be_bytes();

    let mut timestamp_with_pub_key = Vec::new();
    timestamp_with_pub_key.append(&mut verifying_key.as_bytes().to_vec());
    timestamp_with_pub_key.append(&mut timestamp.to_vec());

    let signature = signing_key.sign(&timestamp_with_pub_key);
    auth_proof.push(0);
    auth_proof.append(&mut signature.to_vec());
    auth_proof.append(&mut timestamp_with_pub_key);
    let deletions = Arc::new(Mutex::new(HashMap::new()));

    let get_messages_request = Request::builder().body(full(auth_proof)).unwrap();
    let response = retrieve_messages(get_messages_request, &md_config, deletions)
        .await
        .unwrap();
    let status_code = response.status();
    let mut buf = Vec::new();
    let mut response_body = response.boxed().collect().await.unwrap().aggregate();
    while response_body.has_remaining() {
        buf.push(response_body.get_u8());
    }
    let mut length: [u8; 8] = [0; 8];
    length.copy_from_slice(&buf[0..8]);
    println!("length: {}", u64::from_be_bytes(length));
    println!("{}", String::from_utf8(buf[8..].to_vec()).unwrap());
    assert_eq!(StatusCode::OK, status_code);
    std::fs::remove_dir_all(test_dir).unwrap();
}
