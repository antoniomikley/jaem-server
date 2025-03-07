use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_PATH: &str = "testconfig.toml";
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JaemConfig {
    pub message_delivery_config: Option<MessageDeliveryConfig>,
    pub user_discovery_config: Option<UserDiscoveryConfig>,
}

impl JaemConfig {
    pub fn get_message_delivery_config(&self) -> MessageDeliveryConfig {
        self.message_delivery_config.clone().unwrap()
    }
    pub fn create_default() -> JaemConfig {
        return JaemConfig {
            message_delivery_config: Some(MessageDeliveryConfig::default()),
            user_discovery_config: Some(UserDiscoveryConfig::default()),
        };
    }

    pub fn read_from_file(file_path: &str) -> Result<JaemConfig, anyhow::Error> {
        let mut config_file = File::open(file_path)?;
        let mut file_contents = String::new();
        config_file.read_to_string(&mut file_contents)?;
        let config = toml::from_str(&file_contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), anyhow::Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)?;
        file.write(toml::to_string_pretty(self).unwrap().as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default = "MessageDeliveryConfig::default")]
pub struct MessageDeliveryConfig {
    #[serde(default = "MessageDeliveryConfig::default_share_directory")]
    pub share_directory: PathBuf,
    #[serde(default = "MessageDeliveryConfig::default_storage_path")]
    pub storage_path: PathBuf,
}

impl MessageDeliveryConfig {
    pub fn default() -> MessageDeliveryConfig {
        return MessageDeliveryConfig {
            storage_path: Self::default_storage_path(),
            share_directory: Self::default_share_directory(),
        };
    }

    fn default_share_directory() -> PathBuf {
        return PathBuf::from_str("./share").unwrap();
    }

    fn default_storage_path() -> PathBuf {
        return PathBuf::from_str("./messages").unwrap();
    }

    pub fn set_storage_path(&mut self, storage_path: &str) -> Result<(), anyhow::Error> {
        let new_path = PathBuf::from_str(storage_path)?;
        match new_path.try_exists() {
            Ok(true) => {}
            _ => fs::create_dir_all(&new_path)?,
        };
        self.storage_path = new_path;
        Ok(())
    }
    pub fn set_share_dir(&mut self, share_path: &str) -> Result<(), anyhow::Error> {
        let new_path = PathBuf::from_str(share_path)?;
        match new_path.try_exists() {
            Ok(true) => {}
            _ => fs::create_dir_all(&new_path)?,
        };
        self.share_directory = new_path;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDiscoveryConfig {
    #[serde(default = "UserDiscoveryConfig::default_port")]
    port: u16,
    #[serde(default = "UserDiscoveryConfig::default_storage_path")]
    storage_path: PathBuf,
}

impl UserDiscoveryConfig {
    fn default() -> UserDiscoveryConfig {
        return Self {
            port: Self::default_port(),
            storage_path: Self::default_storage_path(),
        };
    }
    fn default_port() -> u16 {
        return 8081;
    }

    fn default_storage_path() -> PathBuf {
        return PathBuf::from_str("/var/lib/jaem-server/user-discovery/users.json").unwrap();
    }
}
