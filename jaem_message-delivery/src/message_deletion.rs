use std::path::PathBuf;
use std::{collections::HashMap, fs};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};

/// A Ressource that is intended to be deleted at a later time.
#[derive(Clone, Debug)]
pub struct OutstandingDeletion {
    pub timestamp: u64,
    pub identifier: Vec<u8>,
}

impl OutstandingDeletion {
    /// Construct a new OutstandingDeletion from a UNIX timestamp (probably the current time) and
    /// an identifier like e.g. a public key or file name.
    pub fn new(timestamp: u64, identifier: &[u8]) -> OutstandingDeletion {
        Self {
            timestamp,
            identifier: identifier.to_vec(),
        }
    }

    /// Delete the ressource from the file system. The file name has to be base64 urlsafe encoded.
    pub fn delete(&self, message_path: PathBuf) -> Result<(), anyhow::Error> {
        let encoded_pub_key = URL_SAFE.encode(self.identifier.as_slice());
        let mut message_path = message_path;
        message_path.push(encoded_pub_key);
        fs::remove_file(message_path)?;
        Ok(())
    }
}

/// Takes a Hashmap and removes the entries that are older than the specified timeout from itself.
pub fn remove_expired_deletions(
    outstanding: &mut HashMap<Vec<u8>, OutstandingDeletion>,
    current_time: u64,
    timeout: u64,
) {
    for (key, deletion) in outstanding.clone() {
        if deletion.timestamp + timeout <= current_time {
            outstanding.remove(&key);
        }
    }
}

/// Takes a Hashmap and deletes the ressources in te share directory that are older than the
/// specified timout.
pub fn delete_expired_deletions(
    outstanding: &mut HashMap<Vec<u8>, OutstandingDeletion>,
    current_time: u64,
    timeout: u64,
    share_directory: PathBuf,
) {
    for (key, deletion) in outstanding.clone() {
        if deletion.timestamp + timeout <= current_time {
            let mut share_file_path = share_directory.clone();
            share_file_path.push(String::from_utf8(deletion.identifier).unwrap());
            match std::fs::remove_file(share_file_path) {
                Ok(_) => {
                    outstanding.remove(&key).unwrap();
                    return;
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return;
                }
            }
        }
    }
}
