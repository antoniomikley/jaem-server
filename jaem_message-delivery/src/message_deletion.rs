use std::path::PathBuf;
use std::{collections::HashMap, fs};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[derive(Clone)]
pub struct OutstandingDeletion {
    pub timestamp: u64,
    pub pub_key: Vec<u8>,
}

impl OutstandingDeletion {
    pub fn new(timestamp: u64, pub_key: &[u8]) -> OutstandingDeletion {
        Self {
            timestamp,
            pub_key: pub_key.to_vec(),
        }
    }

    pub fn delete(&self, message_path: PathBuf) -> Result<(), anyhow::Error> {
        let encoded_pub_key = URL_SAFE.encode(self.pub_key.as_slice());
        let mut message_path = message_path;
        message_path.push(encoded_pub_key);
        fs::remove_file(message_path)?;
        Ok(())
    }
}

pub fn remove_expired_deletions(
    outstanding: &mut HashMap<Vec<u8>, OutstandingDeletion>,
    current_time: u64,
) {
    for (key, deletion) in outstanding.clone() {
        if deletion.timestamp + 20 >= current_time {
            outstanding.remove(&key);
        }
    }
}
