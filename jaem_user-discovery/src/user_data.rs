use std::{
    fs::{self, OpenOptions},
    os::unix::fs::FileExt,
    path::Path,
    str::FromStr,
};

use anyhow::anyhow;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};

const PROFILE_PICTURE_ROOT: &str = "./src/profile_pictures/";

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStorage {
    pub users: Vec<UserData>,
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

impl UserData {
    pub fn add_pub_key(&mut self, key: PubKey) {
        self.public_keys.push(key);
    }
}

impl UserStorage {
    pub fn add_entry(
        &mut self,
        user_data: &mut UserData,
        file_path: &str,
    ) -> Result<(), anyhow::Error> {
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
                self.generate_profile_picture(user_data);
                self.check_description(user_data);
                self.users.insert(i, user_data.clone());

                self.save_to_file(file_path)?;
                Ok(())
            }
        }
    }

    fn generate_profile_picture(&self, user: &mut UserData) {
        if user.profile_picture.is_empty() {
            user.profile_picture = "default.png".to_string();
        } else {
            let file_path = format!("{}{}", PROFILE_PICTURE_ROOT, user.uid.clone() + ".png");
            let file_path = Path::new(&file_path);

            // Extract the parent directory from the file path
            if let Some(parent_dir) = file_path.parent() {
                // Create the parent directory and any missing ancestors
                fs::create_dir_all(parent_dir).unwrap();
            }

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)
                .unwrap();
            let _ = file.write_at(user.profile_picture.as_bytes(), 0);
            user.profile_picture = file_path.to_str().unwrap().to_string();
        }
    }

    fn check_description(&self, user: &mut UserData) {
        if user.description.is_empty() {
            user.description = "Hey there! Let`s have a Jaem.".to_string();
        }
    }

    pub fn update_profile_picture(
        &mut self,
        uid: String,
        profile_picture: String,
    ) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&uid, |user| user.uid.clone())
        {
            Ok(i) => {
                let user = &mut self.users[i];
                let file_path = format!("{}{}", PROFILE_PICTURE_ROOT, user.uid.clone() + ".png");
                let file = std::fs::File::create(&file_path).unwrap();
                let _ = file.write_at(profile_picture.as_bytes(), 0);
                user.profile_picture = file_path;
                Ok(())
            }
            Err(_) => {
                let code = 1;
                let message = "User not found";
                let response_body =
                    format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message).to_string();
                Err(anyhow!(response_body))
            }
        }
    }
    pub fn add_pub_keys(
        &mut self,
        uid: String,
        pub_keys: Vec<PubKey>,
        file_path: &str,
    ) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&uid, |user| user.uid.clone())
        {
            Ok(i) => {
                for key in pub_keys {
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

    pub fn delete_entry(&mut self, uid: String, file_path: &str) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&uid, |user| user.uid.clone())
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
        uid: String,
        signature_key: String,
        file_path: &str,
    ) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&uid, |user| user.uid.clone())
        {
            Ok(i) => {
                let user = &mut self.users[i];
                let decoded_pub_key = percent_decode_str(&signature_key)
                    .decode_utf8()
                    .expect("Failed to decode public key");
                println!("{}", decoded_pub_key);
                match user
                    .public_keys
                    .binary_search_by_key(&decoded_pub_key, |user| {
                        user.signature_key.clone().into()
                    }) {
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

    pub fn get_users(&self, page: usize, page_size: usize) -> Vec<UserData> {
        let start = page * page_size;
        let end = match start + page_size {
            end if end < self.users.len() => end,
            _ => self.users.len(),
        };

        self.users.get(start..end).unwrap_or(&[]).to_vec()
    }

    pub fn get_entry(&self, username: String) -> Option<UserData> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.username.clone())
        {
            Ok(i) => {
                let user = self.users[i].clone();
                let file_data = std::fs::read(&user.profile_picture).unwrap();
                let return_user = UserData {
                    uid: user.uid,
                    username: user.username,
                    public_keys: user.public_keys,
                    profile_picture: String::from_utf8(file_data).unwrap(),
                    description: user.description,
                };
                return Some(return_user);
            }
            Err(_) => return None,
        }
    }

    pub fn get_entries_by_pattern(
        &self,
        pattern: String,
        page: usize,
        page_size: usize,
    ) -> Option<Vec<UserData>> {
        let result: Vec<UserData> = self
            .users
            .iter()
            .filter(|user| {
                user.username
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
            })
            .map(|user| {
                let file_data =
                    std::fs::read(&user.profile_picture).unwrap_or("default.png".into());
                let return_user = UserData {
                    uid: user.uid.clone(),
                    username: user.username.clone(),
                    public_keys: user.public_keys.clone(),
                    profile_picture: String::from_utf8(file_data)
                        .unwrap_or("default.png".to_string()),
                    description: user.description.clone(),
                };
                return_user
            })
            .collect();
        let start = page * page_size;
        let end = match start + page_size {
            end if end < result.len() => end,
            _ => result.len(),
        };

        let result = result.get(start..end).unwrap_or(&[]).to_vec();

        if result.len() > 0 {
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn get_entry_by_uid(&self, uid: String) -> Option<UserData> {
        for user in &self.users {
            if user.uid == uid {
                let file_data =
                    std::fs::read(&user.profile_picture).unwrap_or("default.png".into());
                let return_user = UserData {
                    uid: user.uid.clone(),
                    username: user.username.clone(),
                    public_keys: user.public_keys.clone(),
                    profile_picture: String::from_utf8(file_data)
                        .unwrap_or("default_png".to_string()),
                    description: user.description.clone(),
                };
                println!("{:?}", return_user);
                return Some(return_user);
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
