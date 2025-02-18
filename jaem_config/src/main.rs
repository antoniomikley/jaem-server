use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct JaemConfig {
    message_delivery_config: Option<MessageDeliveryConfig>,
    user_discovery_config: Option<UserDiscoveryConfig>,
}

impl JaemConfig {
    fn create_default() -> JaemConfig {
        return JaemConfig {
            message_delivery_config: Some(MessageDeliveryConfig::default()),
            user_discovery_config: Some(UserDiscoveryConfig::default()),
        };
    }

    pub fn save_to_file(&self) -> Result<(), anyhow::Error> {
        let mut file = OpenOptions::new().write(true).open("testconfig.toml")?;
        file.write(toml::to_string_pretty(self).unwrap().as_bytes());

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default = "MessageDeliveryConfig::default")]
struct MessageDeliveryConfig {
    storage_path: PathBuf,
}

impl MessageDeliveryConfig {
    fn default() -> MessageDeliveryConfig {
        let default_path = PathBuf::from_str("/var/lib/jaem-server/message-delivery/").unwrap();
        match default_path.try_exists() {
            Err(_) => fs::create_dir_all(&default_path).expect(&format!(
                "Could not create direcory {}",
                &default_path.to_str().unwrap()
            )),
            Ok(false) => panic!("Could read config path, but something is wrong."),
            Ok(true) => {}
        };
        return MessageDeliveryConfig {
            storage_path: default_path,
        };
    }

    fn set_storage_path(&mut self, storage_path: &str) -> Result<(), anyhow::Error> {
        let new_path = PathBuf::from_str(storage_path)?;
        match new_path.try_exists() {
            Err(_) => fs::create_dir_all(&new_path)?,
            Ok(false) => return Err(anyhow!("Could read Path but something different is wrong.")),
            Ok(true) => {}
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct UserDiscoveryConfig {
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
        let default_path =
            PathBuf::from_str("/var/lib/jaem-server/user-discovery/users.json").unwrap();
        match default_path.try_exists() {
            Err(_) => fs::create_dir_all(&default_path.parent().unwrap()).expect(&format!(
                "Could not create direcory {}",
                &default_path.to_str().unwrap()
            )),
            Ok(false) => panic!("Could read config path, but something is wrong."),
            Ok(true) => {}
        };
        return default_path;
    }
}

fn main() {
    let config = JaemConfig::create_default();
    println!("{:?}", configs);
}
