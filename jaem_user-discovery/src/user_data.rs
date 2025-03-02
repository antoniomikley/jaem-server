use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStorage {
    pub users: Vec<UserData>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    pub uid: String,
    pub username: String,
    pub public_keys: Vec<PubKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubKey {
    pub algorithm: PubKeyAlgo,
    pub key: String,
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
            _ => Err(anyhow!("Invalid username algorithm")),
        }
    }
}

impl UserData {
    pub fn add_pub_key(&mut self, key: PubKey) {
        self.public_keys.push(key);
    }
}

impl UserStorage {
    pub fn add_entry(&mut self, user_data: UserData, file_path: &str) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&user_data.uid, |user| user.uid.clone())
        {
            Ok(_) => {
                let code = 1;
                let message = "User already exists";
                let response_body = format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
                Err(anyhow!(response_body))
            }
            Err(i) => {
                self.users.insert(i, user_data);
                self.save_to_file(file_path)?;
                Ok(())
            }
        }
    }
    pub fn add_pub_keys(
        &mut self,
        user_data: UserData,
        file_path: &str,
    ) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&user_data.username, |user| user.username.clone())
        {
            Ok(i) => {
                for key in user_data.public_keys {
                    self.users[i].add_pub_key(key);
                    self.save_to_file(file_path)?;
                }
            }
            Err(_) => {
                return Err(anyhow!("User not found"));
            }
        }
        Ok(())
    }

    pub fn delete_entry(&mut self, username: String, file_path: &str) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.username.clone())
        {
            Ok(i) => {
                self.users.remove(i);
                let _ = self.save_to_file(file_path);
                Ok(())
            }
            Err(_) => Err(anyhow!("User not found")),
        }
    }

    pub fn delete_pub_key(
        &mut self,
        username: String,
        key: String,
        file_path: &str,
    ) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.username.clone())
        {
            Ok(i) => {
                println!("Username: {}, Key: {}", username, key);
                let user = &mut self.users[i];
                match user
                    .public_keys
                    .binary_search_by_key(&key, |key| key.key.clone())
                {
                    Ok(j) => {
                        user.public_keys.remove(j);
                        self.save_to_file(file_path)?;
                        Ok(())
                    }
                    Err(_) => Err(anyhow!("Key not found")),
                }
            }
            Err(_) => Err(anyhow!("User not found")),
        }
    }

    pub fn get_users(&self) -> Vec<UserData> {
        self.users.clone()
    }

    pub fn get_entry(&self, username: String) -> Option<UserData> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.username.clone())
        {
            Ok(i) => return Some(self.users[i].clone()),
            Err(_) => return None,
        }
    }

    pub fn get_entries_by_pattern(&self, pattern: String) -> Option<Vec<UserData>> {
        let result: Vec<UserData> = self
            .users
            .iter()
            .filter(|user| {
                user.username
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            })
            .cloned()
            .collect();
        if result.len() > 0 {
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn get_entry_by_uid(&self, uid: String) -> Option<UserData> {
        for user in &self.users {
            if user.uid == uid {
                return Some(user.clone());
            }
        }
        None
    }

    pub fn read_from_file(file_path: &str) -> Result<UserStorage, anyhow::Error> {
        let file = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                let default_storage = UserStorage { users: Vec::new() };
                default_storage.save_to_file(file_path)?;
                std::fs::File::open(file_path)?
            }
        };
        let reader = std::io::BufReader::new(file);
        let storage = serde_json::from_reader(reader)?;
        Ok(storage)
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), anyhow::Error> {
        let file = std::fs::File::create(file_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
}
