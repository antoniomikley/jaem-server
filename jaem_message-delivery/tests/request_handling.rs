use jaem_config::{JaemConfig, MessageDeliveryConfig, DEFAULT_CONFIG_PATH};
use jaem_message_delivery::request_handling::receive_messages;

#[test]
fn sending_message_with_invalid_algorithm_byte() {
    let config = JaemConfig::create_default();
    let mut md_config = config.get_message_delivery_config();
    md_config.set_storage_path("testtest").unwrap();
    println!("{:?}", md_config);
}
