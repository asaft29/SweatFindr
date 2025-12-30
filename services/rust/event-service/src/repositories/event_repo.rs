use crate::models::event::{CreateEvent, Event, EventQuery, UpdateEvent};
use crate::utils::error::*;
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
            query_builder.push_bind(format!("%{}%", location));
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

        query_builder.push(" ORDER BY nume ASC");

        let page = params.paginare.page.unwrap_or(1);
        let items_per_page = params.paginare.items_per_page.unwrap_or(10);
        let offset = (page - 1) * items_per_page;

        query_builder.push(" LIMIT ");
        query_builder.push_bind(items_per_page);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let query = query_builder.build_query_as::<Event>();
        let events = query
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_event_error)?;
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

    pub async fn create_event(
        &self,
        id_owner: i32,
        payload: CreateEvent,
    ) -> Result<Event, EventRepoError> {
        let result = sqlx::query_as::<_, Event>(
            r#"
            INSERT INTO EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarlocuri)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING ID, ID_OWNER, nume, locatie, descriere, numarlocuri
            "#,
        )
        .bind(id_owner)
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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(EventRepoError::InternalError)?;

        if let Some(new_seats) = payload.locuri {
            let tickets_sold: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM BILETE WHERE evenimentid = $1")
                    .bind(event_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(EventRepoError::InternalError)?;

            if new_seats < tickets_sold as i32 {
                return Err(EventRepoError::ConstraintViolation(format!(
                    "Cannot reduce seats to {} when {} tickets are already sold",
                    new_seats, tickets_sold
                )));
            }
        }

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
        .fetch_one(&mut *tx)
        .await;

        let event = match result {
            Ok(event) => event,
            Err(Error::RowNotFound) => return Err(EventRepoError::NotFound),
            Err(e) => return Err(EventRepoError::InternalError(e)),
        };

        if payload.locuri.is_some() {
            self.update_packet_seats_for_event(&mut tx, event_id)
                .await?;
        }

        tx.commit().await.map_err(EventRepoError::InternalError)?;

        Ok(event)
    }

    pub async fn patch_event(
        &self,
        event_id: i32,
        payload: crate::models::event::PatchEvent,
    ) -> Result<Event, EventRepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(EventRepoError::InternalError)?;

        if let Some(new_seats) = payload.locuri {
            let tickets_sold: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM BILETE WHERE evenimentid = $1")
                    .bind(event_id)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(EventRepoError::InternalError)?;

            if new_seats < tickets_sold as i32 {
                return Err(EventRepoError::ConstraintViolation(format!(
                    "Cannot reduce seats to {} when {} tickets are already sold",
                    new_seats, tickets_sold
                )));
            }
        }

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
        .bind(payload.nume.as_deref())
        .bind(payload.locatie.as_deref())
        .bind(payload.descriere.as_deref())
        .bind(payload.locuri)
        .bind(event_id)
        .fetch_one(&mut *tx)
        .await;

        let event = match result {
            Ok(event) => event,
            Err(Error::RowNotFound) => return Err(EventRepoError::NotFound),
            Err(e) => return Err(EventRepoError::InternalError(e)),
        };

        if payload.locuri.is_some() {
            self.update_packet_seats_for_event(&mut tx, event_id)
                .await?;
        }

        tx.commit().await.map_err(EventRepoError::InternalError)?;

        Ok(event)
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

    async fn update_packet_seats_for_event(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        event_id: i32,
    ) -> Result<(), EventRepoError> {
        let packet_ids: Vec<(i32,)> =
            sqlx::query_as("SELECT DISTINCT pachetid FROM JOIN_PE WHERE evenimentid = $1")
                .bind(event_id)
                .fetch_all(&mut **tx)
                .await
                .map_err(EventRepoError::InternalError)?;

        for (packet_id,) in packet_ids {
            let min_seats: Option<i32> = sqlx::query_scalar(
                r#"
                SELECT MIN(e.numarlocuri)
                FROM EVENIMENTE e
                JOIN JOIN_PE j ON e.id = j.evenimentid
                WHERE j.pachetid = $1
                "#,
            )
            .bind(packet_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(EventRepoError::InternalError)?
            .flatten();

            sqlx::query("UPDATE PACHETE SET numarlocuri = $1 WHERE id = $2")
                .bind(min_seats)
                .bind(packet_id)
                .execute(&mut **tx)
                .await
                .map_err(EventRepoError::InternalError)?;
        }

        Ok(())
    }
}
