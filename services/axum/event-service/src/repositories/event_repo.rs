use crate::models::event::{CreateEvent, Event, EventQuery, UpdateEvent};
use crate::shared::error::*;
use anyhow::Result;
use sqlx::{Error, PgPool, Postgres, QueryBuilder};

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

    pub async fn list_events(&self, params: EventQuery) -> Result<Vec<Event>, EventRepoError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT ID, ID_OWNER, nume, locatie, descriere, numarlocuri FROM EVENIMENTE",
        );

        let mut has_condition = false;

        let location = params.locatie.filter(|s| !s.is_empty());
        let name = params.nume.filter(|s| !s.is_empty());

        if let Some(location) = location {
            query_builder.push(" WHERE unaccent(locatie) ILIKE unaccent(");
            query_builder.push_bind(format!("{}%", location));
            query_builder.push(")");
            has_condition = true;
        }

        if let Some(name) = name {
            if has_condition {
                query_builder.push(" AND unaccent(nume) ILIKE unaccent(");
            } else {
                query_builder.push(" WHERE unaccent(nume) ILIKE unaccent(");
            }
            query_builder.push_bind(format!("%{}%", name));
            query_builder.push(")");
        }

        let query = query_builder.build_query_as::<Event>();
        let events = query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| EventRepoError::InternalError(e))?;
        Ok(events)
    }

    pub async fn get_event(&self, event_id: i32) -> Result<Event, EventRepoError> {
        let result = sqlx::query_as::<_, Event>(
            r#"
            SELECT ID, ID_OWNER, nume, locatie, descriere, numarlocuri
            FROM EVENIMENTE
            WHERE ID = $1
            "#,
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(event) => Ok(event),
            Err(Error::RowNotFound) => Err(EventRepoError::NotFound),
            Err(e) => Err(EventRepoError::InternalError(e)),
        }
    }

    pub async fn create_event(&self, payload: CreateEvent) -> Result<Event, EventRepoError> {
        let result = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarlocuri)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING ID, ID_OWNER, nume, locatie, descriere, numarlocuri
            "#,
        )
        .bind(payload.id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .bind(payload.locuri)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_event_error)
    }

    pub async fn update_event(
        &self,
        event_id: i32,
        payload: UpdateEvent,
    ) -> Result<Event, EventRepoError> {
        let result = sqlx::query_as::<_, Event>(
            r#"
        UPDATE EVENIMENTE
        SET
            id_owner = COALESCE($1, id_owner),
            nume = COALESCE($2, nume),
            locatie = COALESCE($3, locatie),
            descriere = COALESCE($4, descriere),
            numarlocuri = COALESCE($5, numarlocuri)
        WHERE ID = $6
        RETURNING ID, ID_OWNER, nume, locatie, descriere, numarlocuri
        "#,
        )
        .bind(payload.id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .bind(payload.locuri)
        .bind(event_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(event) => Ok(event),
            Err(Error::RowNotFound) => Err(EventRepoError::NotFound),
            Err(e) => Err(EventRepoError::InternalError(e)),
        }
    }

    pub async fn delete_event(&self, event_id: i32) -> Result<(), EventRepoError> {
        let result = sqlx::query("DELETE FROM EVENIMENTE WHERE ID = $1")
            .bind(event_id)
            .execute(&self.pool)
            .await
            .map_err(EventRepoError::InternalError)?;

        if result.rows_affected() == 0 {
            Err(EventRepoError::NotFound)
        } else {
            Ok(())
        }
    }
}
