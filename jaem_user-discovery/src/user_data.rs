use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStorage {
    users: Vec<UserData>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserData {
    id: String,
    public_keys: Vec<PubKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PubKey {
    algorithm: PubKeyAlgo,
    key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum PubKeyAlgo {
    ED25519,
}

impl UserData {
    pub fn add_pub_key(&mut self, key: PubKey) {
        self.public_keys.push(key);
    }
}

impl UserStorage {
    pub fn add_pub_keys(&mut self, user_data: UserData) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&user_data.id, |user| user.id.clone())
        {
            Ok(i) => {
                for key in user_data.public_keys {
                    self.users[i].add_pub_key(key);
                }
            }
            Err(i) => {
                self.users.insert(i, user_data);
            }
        }
        self.save_to_file("users.json")?;
        Ok(())
    }

    pub fn delete_entry(&mut self, username: String) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.id.clone())
        {
            Ok(i) => {
                self.users.remove(i);
                self.save_to_file("users.json")?;
                Ok(())
            }
            Err(_) => Err(anyhow!("User not found")),
        }
    }

    pub fn delete_pub_key(&mut self, username: String, key: String) -> Result<(), anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.id.clone())
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
                        self.save_to_file("users.json")?;
                        Ok(())
                    }
                    Err(_) => Err(anyhow!("Key not found")),
                }
            }
            Err(_) => Err(anyhow!("User not found")),
        }
    }

    pub fn get_entry(&self, username: String) -> Result<UserData, anyhow::Error> {
        match self
            .users
            .binary_search_by_key(&username, |user| user.id.clone())
        {
            Ok(i) => Ok(self.users[i].clone()),
            Err(_) => Err(anyhow!("User not found")),
        }
    }

    pub fn get_entries_by_pattern(&self, pattern: &str) -> Option<Vec<UserData>> {
        let result: Vec<UserData> = self
            .users
            .iter()
            .filter(|user| user.id.to_lowercase().contains(&pattern.to_lowercase()))
            .cloned()
            .collect();
        if result.len() > 0 {
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn read_from_file(file_path: &str) -> Result<UserStorage, anyhow::Error> {
        let file = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(_) => {
                let mut default_storage = UserStorage { users: Vec::new() };
                let default_user = UserData {
                    id: "admin".to_string(),
                    public_keys: Vec::new(),
                };
                let default_key = PubKey {
                    algorithm: PubKeyAlgo::ED25519,
                    key: "default_key".to_string(),
                };

                default_storage.users.push(default_user);
                default_storage.users[0].add_pub_key(default_key);
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
