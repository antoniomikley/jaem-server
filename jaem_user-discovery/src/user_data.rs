use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReturnUserData {
    pub id: i32,
    pub uid: String,
    pub username: String,
    pub public_keys: Vec<PubKey>,
    pub profile_picture: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    pub uid: String,
    pub username: String,
    pub public_keys: Vec<PubKey>,
    pub profile_picture: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubKey {
    pub algorithm: PubKeyAlgo,
    pub signature_key: String,
    pub exchange_key: String,
    pub rsa_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PubKeyAlgo {
    ED25519,
}

impl FromStr for PubKeyAlgo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ED25519" => Ok(PubKeyAlgo::ED25519),
            _ => Err(anyhow!("Invalid algorithm")),
        }
    }
}

impl ToString for PubKeyAlgo {
    fn to_string(&self) -> String {
        match self {
            PubKeyAlgo::ED25519 => "ED25519".to_string(),
        }
    }
}
