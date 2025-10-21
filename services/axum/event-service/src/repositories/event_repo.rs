use crate::models::event::{CreateEvent, Event, UpdateEvent};
use anyhow::Result;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::{Error, PgPool};

#[derive(Debug)]
pub enum RepoError {
    NotFound,
    InvalidReference,
    DuplicateEntry,
    InternalError(Error),
}

impl IntoResponse for RepoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RepoError::NotFound => (
                StatusCode::NOT_FOUND,
                json!({ "error": "The requested resource was not found." }),
            ),
            RepoError::InvalidReference => (
                StatusCode::BAD_REQUEST,
                json!({ "error": "A provided reference, such as an owner ID, is invalid." }),
            ),
            RepoError::DuplicateEntry => (
                StatusCode::CONFLICT,
                json!({ "error": "An event with this name already exists." }),
            ),
            RepoError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "error": "An internal server error occurred." }),
            ),
        };

        (status, Json(error_message)).into_response()
    }
}

fn map_sqlx_error(err: Error) -> RepoError {
    if let Some(db_err) = err.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                "23503" => return RepoError::InvalidReference,
                "23505" => return RepoError::DuplicateEntry,
                _ => {}
            }
        }
    }
    match err {
        Error::RowNotFound => RepoError::NotFound,
        e => RepoError::InternalError(e),
    }
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

    pub async fn create_event(&self, payload: CreateEvent) -> Result<Event, RepoError> {
        let result = sqlx::query_as!(
            Event,
            r#"
            INSERT INTO EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarlocuri)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING ID, ID_OWNER, nume, locatie, descriere, numarlocuri as locuri
            "#,
            payload.id_owner,
            payload.nume,
            payload.locatie,
            payload.descriere,
            payload.locuri
        )
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_error)
    }

    pub async fn get_event(&self, event_id: i32) -> Result<Event, RepoError> {
        let result = sqlx::query_as!(
            Event,
            r#"
            SELECT ID, ID_OWNER, nume, locatie, descriere, numarlocuri as locuri
            FROM EVENIMENTE
            WHERE ID = $1
            "#,
            event_id
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(event) => Ok(event),
            Err(Error::RowNotFound) => Err(RepoError::NotFound),
            Err(e) => Err(RepoError::InternalError(e)),
        }
    }

    pub async fn update_event(
        &self,
        event_id: i32,
        payload: UpdateEvent,
    ) -> Result<Event, RepoError> {
        let result = sqlx::query_as!(
            Event,
            r#"
            UPDATE EVENIMENTE
            SET nume = $1, locatie = $2, descriere = $3, numarlocuri = $4
            WHERE ID = $5
            RETURNING ID, ID_OWNER, nume, locatie, descriere, numarlocuri as locuri
            "#,
            payload.nume,
            payload.locatie,
            payload.descriere,
            payload.locuri,
            event_id
        )
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(event) => Ok(event),
            Err(Error::RowNotFound) => Err(RepoError::NotFound),
            Err(e) => Err(RepoError::InternalError(e)),
        }
    }

    pub async fn delete_event(&self, event_id: i32) -> Result<(), RepoError> {
        let result = sqlx::query!("DELETE FROM EVENIMENTE WHERE ID = $1", event_id)
            .execute(&self.pool)
            .await
            .map_err(RepoError::InternalError)?;

        if result.rows_affected() == 0 {
            Err(RepoError::NotFound)
        } else {
            Ok(())
        }
    }
}
