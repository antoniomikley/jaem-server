use std::str::FromStr;

use anyhow::{anyhow, Error};
use tokio_postgres::{types::ToSql, Client};

use crate::user_data::{PubKey, PubKeyAlgo, ReturnUserData, UserData};

pub struct Database {}

impl Database {
    pub async fn get_users(
        client: &Client,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<ReturnUserData>, Error> {
        let start: i64 = (page * page_size) as i64;
        let stmt = client
            .prepare("SELECT uid, username, profile_picture, description FROM users ORDER BY id OFFSET $1 LIMIT $2")
            .await?;
        let rows = client.query(&stmt, &[&start, &(page_size as i64)]).await?;

        let mut users = Vec::new();
        let mut id_iterator = start as i32;
        for row in rows {
            let mut user = ReturnUserData {
                id: id_iterator,
                uid: row.get(0),
                username: row.get(1),
                profile_picture: row.get(2),
                description: row.get(3),
                public_keys: Vec::new(),
            };

            let stmt = client
                .prepare("SELECT * FROM public_keys WHERE user_id = $1")
                .await?;
            let rows = client.query(&stmt, &[&user.id]).await?;

            for row in rows {
                let key = PubKey {
                    algorithm: PubKeyAlgo::from_str(row.get(2)).unwrap(),
                    signature_key: row.get(3),
                    exchange_key: row.get(4),
                    rsa_key: row.get(5),
                };
                user.public_keys.push(key);
            }

            users.push(user);
            id_iterator += 1;
        }

        Ok(users)
    }

    pub async fn get_entries_by_pattern(
        client: &Client,
        pattern: &str,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<ReturnUserData>, Error> {
        let start: i64 = (page * page_size) as i64;
        let stmt = client
            .prepare("SELECT uid, username, profile_picture, description FROM users WHERE username LIKE $1 ORDER BY id OFFSET $2 LIMIT $3")
            .await?;
        let rows = client
            .query(
                &stmt,
                &[&format!("%{}%", pattern), &start, &(page_size as i64)],
            )
            .await?;

        let mut users = Vec::new();
        let mut id_iterator = start as i32;

        for row in rows {
            let mut user = ReturnUserData {
                id: id_iterator,
                uid: row.get(0),
                username: row.get(1),
                profile_picture: row.get(2),
                description: row.get(3),
                public_keys: Vec::new(),
            };

            let stmt = client
                .prepare("SELECT * FROM public_keys WHERE user_id = $1")
                .await?;
            let rows = client.query(&stmt, &[&user.id]).await?;

            for row in rows {
                let key = PubKey {
                    algorithm: PubKeyAlgo::from_str(row.get(2)).unwrap(),
                    signature_key: row.get(3),
                    exchange_key: row.get(4),
                    rsa_key: row.get(5),
                };
                user.public_keys.push(key);
            }

            users.push(user);
            id_iterator += 1;
        }

        Ok(users)
    }

    pub async fn get_entry_by_uid(
        client: &Client,
        uid: String,
    ) -> Result<Option<ReturnUserData>, Error> {
        let stmt = client.prepare("SELECT * FROM users WHERE uid = $1").await?;
        let row = client.query_opt(&stmt, &[&uid]).await?;

        match row {
            Some(row) => {
                let mut user = ReturnUserData {
                    id: row.get(0),
                    uid: row.get(1),
                    username: row.get(2),
                    profile_picture: row.get(3),
                    description: row.get(4),
                    public_keys: Vec::new(),
                };

                let stmt = client
                    .prepare("SELECT * FROM public_keys WHERE user_id = $1")
                    .await?;
                let rows = client.query(&stmt, &[&user.id]).await?;

                for row in rows {
                    let key = PubKey {
                        algorithm: PubKeyAlgo::from_str(row.get(2)).unwrap(),
                        signature_key: row.get(3),
                        exchange_key: row.get(4),
                        rsa_key: row.get(5),
                    };
                    user.public_keys.push(key);
                }

                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn add_new_entry(client: &Client, user: UserData) -> Result<(), Error> {
        match Database::user_exists_already(client, user.uid.to_string()).await? {
            true => {
                let code = 1;
                let message = "User already exists";
                let response_body = format!("{{\"code\": {}, \"message\": \"{}\"}}", code, message);
                return Err(anyhow!(response_body));
            }
            false => {
                println!("User does not exist");
                let stmt = client.prepare("INSERT INTO users (uid, username, profile_picture, description) VALUES ($1, $2, $3, $4)").await?;
                client
                    .execute(
                        &stmt,
                        &[
                            &user.uid,
                            &user.username,
                            &user.profile_picture,
                            &user.description,
                        ],
                    )
                    .await?;

                println!("User added");
                for key in &user.public_keys {
                    let stmt = client.prepare("INSERT INTO public_keys(user_id, algorithm, signature_key, exchange_key, rsa_key) 
                                            VALUES ((SELECT id FROM users WHERE uid=$1), $2, $3, $4, $5)").await?;
                    client
                        .execute(
                            &stmt,
                            &[
                                &user.uid,
                                &key.algorithm.to_string(),
                                &key.signature_key,
                                &key.exchange_key,
                                &key.rsa_key,
                            ],
                        )
                        .await?;
                    println!("Key added");
                }

                Ok(())
            }
        }
    }

    pub async fn add_pub_keys(
        client: &Client,
        uid: String,
        public_keys: &Vec<PubKey>,
    ) -> Result<(), Error> {
        for key in public_keys {
            let stmt = client.prepare("INSERT INTO public_keys(user_id, algorithm, signature_key, exchange_key, rsa_key) 
                                            VALUES ((SELECT id FROM users WHERE uid=$1), $2, $3, $4, $5)").await?;
            client
                .execute(
                    &stmt,
                    &[
                        &uid,
                        &key.algorithm.to_string(),
                        &key.signature_key,
                        &key.exchange_key,
                        &key.rsa_key,
                    ],
                )
                .await?;
        }
        Ok(())
    }

    pub async fn update_users(
        client: &Client,
        uid: String,
        username: Option<&str>,
        profile_picture: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), Error> {
        // Check if the user exists
        match Database::user_exists_already(client, uid.clone()).await? {
            false => Err(anyhow!("User does not exist")),
            true => {
                // Start building the SQL query and parameters vector
                let mut query = "UPDATE users SET".to_string();
                let mut params: Vec<String> = Vec::new();
                let mut param_refs: Vec<&(dyn ToSql + Sync)> = Vec::new();
                let mut counter = 1;

                // Add the fields to the query and params if they are not empty
                if let Some(username) = username {
                    query.push_str(&format!(" username = ${},", counter));
                    params.push(username.to_string());
                    counter += 1;
                }
                if let Some(profile_picture) = profile_picture {
                    query.push_str(&format!(" profile_picture = ${},", counter));
                    params.push(profile_picture.to_string());
                    counter += 1;
                }
                if let Some(description) = description {
                    query.push_str(&format!(" description = ${},", counter));
                    params.push(description.to_string());
                    counter += 1;
                }

                if params.is_empty() {
                    return Err(anyhow!("No fields provided to update"));
                }

                // Remove the last comma
                query.pop();

                // Add the WHERE clause
                query.push_str(&format!(" WHERE uid = ${}", counter));
                params.push(uid);

                // Convert params to ToSql references
                for param in &params {
                    param_refs.push(param);
                }

                // Prepare and execute the query
                let stmt = client.prepare(&query).await?;

                client.execute(&stmt, &param_refs).await?;

                Ok(())
            }
        }
    }

    pub async fn delete_entry(client: &Client, uid: String) -> Result<(), Error> {
        match Database::user_exists_already(client, uid.clone()).await? {
            false => Err(anyhow!("User does not exist")),
            true => {
                let stmt = client.prepare("DELETE FROM users WHERE uid = $1").await?;
                client.execute(&stmt, &[&uid]).await?;
                Ok(())
            }
        }
    }

    pub async fn delete_pub_key(
        client: &Client,
        uid: String,
        signature_key: &str,
    ) -> Result<(), Error> {
        match Database::public_key_exists(client, uid.clone(), signature_key).await? {
            true => Err(anyhow!("Key does not exist")),
            false => {
                let stmt = client
                .prepare("DELETE FROM public_keys WHERE user_id = (SELECT id FROM users WHERE uid = $1) AND signature_key = $2")
                .await?;
                client.execute(&stmt, &[&uid, &signature_key]).await?;
                Ok(())
            }
        }
    }

    pub async fn user_exists_already(client: &Client, uid: String) -> Result<bool, Error> {
        let stmt = client
            .prepare("SELECT * FROM users WHERE uid = $1")
            .await
            .map_err(Error::from)?;
        println!("Checking if user exists");
        let row = client.query(&stmt, &[&uid]).await.map_err(Error::from)?;

        println!("Row: {:?}", row);

        if !row.is_empty() {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn public_key_exists(
        client: &Client,
        uid: String,
        signature_key: &str,
    ) -> Result<bool, Error> {
        let stmt = client
            .prepare("SELECT * FROM public_keys WHERE user_id = (SELECT id FROM users WHERE uid = $1) AND signature_key = $2")
            .await
            .map_err(Error::from)?;
        let row = client
            .query_one(&stmt, &[&uid, &signature_key])
            .await
            .map_err(Error::from)?;

        if !row.is_empty() {
            return Ok(true);
        }

        Ok(false)
    }
}
