use crate::models::{User, UserRole};
use anyhow::Result;
use std::str::FromStr;
use tokio_postgres::Client;
pub struct UserRepository {
    pub client: Client,
}

impl UserRepository {
    pub fn new(client: Client) -> Self {
        UserRepository { client }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let query = "SELECT id, email, parola, rol FROM UTILIZATORI WHERE email = $1";

        match self.client.query_opt(query, &[&email]).await {
            Ok(Some(row)) => {
                let rol_str: String = row.get(3);
                let rol =
                    UserRole::from_str(&rol_str).map_err(|e| format!("Invalid role: {}", e))?;

                Ok(Some(User {
                    id: row.get(0),
                    email: row.get(1),
                    parola: row.get(2),
                    rol,
                }))
            }
            Ok(_) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let query = "SELECT id, email, parola, rol FROM UTILIZATORI WHERE id = $1";

        match self.client.query_opt(query, &[&id]).await {
            Ok(Some(row)) => {
                let rol_str: String = row.get(3);
                let rol =
                    UserRole::from_str(&rol_str).map_err(|e| format!("Invalid role: {}", e))?;

                Ok(Some(User {
                    id: row.get(0),
                    email: row.get(1),
                    parola: row.get(2),
                    rol,
                }))
            }
            Ok(_) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    pub async fn create_user(
        &self,
        email: &str,
        hashed_password: &str,
        role: &UserRole,
    ) -> Result<i32, String> {
        let query = "INSERT INTO UTILIZATORI (email, parola, rol) VALUES ($1, $2, $3) RETURNING id";
        let role_str = role.to_string();

        match self
            .client
            .query_one(query, &[&email, &hashed_password, &role_str])
            .await
        {
            Ok(row) => {
                let user_id: i32 = row.get(0);
                Ok(user_id)
            }
            Err(e) => Err(format!("Failed to create user: {}", e)),
        }
    }

    pub async fn update_role(&self, user_id: i32, role: &UserRole) -> Result<bool, String> {
        let query = "UPDATE UTILIZATORI SET rol = $1 WHERE id = $2";
        let role_str = role.to_string();

        match self.client.execute(query, &[&role_str, &user_id]).await {
            Ok(rows_affected) => match rows_affected {
                0 => Ok(false),
                _ => Ok(true),
            },
            Err(e) => Err(format!("Failed to update user role: {}", e)),
        }
    }
}
