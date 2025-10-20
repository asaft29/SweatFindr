use crate::models::event::Event;
use anyhow::Result;
use sqlx::{Error, PgPool, Row};

#[derive(Debug)]
pub enum RepoError {
    NotFound,
    InternalError(Error),
}

pub struct EventRepo {
    pool: PgPool,
}

impl EventRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check(&self) -> Result<(), Error> {
        sqlx::query("select 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_event(&self, event_id: i32) -> Result<Event, RepoError> {
        let result = sqlx::query(
            r#"
        SELECT 
            ID, 
            ID_OWNER, 
            nume,  
            locatie, 
            descriere, 
            numarlocuri
        FROM EVENIMENTE 
        WHERE ID = $1
        "#,
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => {
                let event = Event {
                    id: row.get("id"),
                    owner_id: row.get("id_owner"),
                    name: row.get("nume"),
                    location: row.get("locatie"),
                    description: row.get("descriere"),
                    sit_count: row.get("numarlocuri"),
                };
                Ok(event)
            }
            Err(Error::RowNotFound) => Err(RepoError::NotFound),
            Err(e) => Err(RepoError::InternalError(e)),
        }
    }
}
