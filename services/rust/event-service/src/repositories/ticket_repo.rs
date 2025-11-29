use crate::models::ticket::{CreateTicket, Ticket, UpdateTicket};
use crate::utils::error::{TicketRepoError, map_sqlx_ticket_error};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TicketRepo {
    pool: PgPool,
}

impl TicketRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_tickets_for_event(
        &self,
        event_id: i32,
    ) -> Result<Vec<Ticket>, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT cod, pachetid, evenimentid
            FROM BILETE
            WHERE evenimentid = $1
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn get_ticket_for_event(
        &self,
        event_id: i32,
        cod: &str,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT cod, pachetid, evenimentid
            FROM BILETE
            WHERE evenimentid = $1 AND cod = $2
            "#,
        )
        .bind(event_id)
        .bind(cod)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn create_ticket(&self, payload: CreateTicket) -> Result<Ticket, TicketRepoError> {
        let new_code = Uuid::now_v7().to_string();

        let result = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO BILETE (cod, pachetid, evenimentid)
            VALUES ($1, $2, $3)
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(new_code)
        .bind(payload.id_pachet)
        .bind(payload.id_event)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn create_ticket_for_event(&self, event_id: i32) -> Result<Ticket, TicketRepoError> {
        let new_code = Uuid::now_v7().to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(TicketRepoError::InternalError)?;

        let seats: Option<i32> =
            sqlx::query_scalar("SELECT numarlocuri FROM EVENIMENTE WHERE id = $1")
                .bind(event_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(TicketRepoError::InternalError)?;

        match seats {
            None => {
                return Err(TicketRepoError::InvalidReference);
            }
            Some(count) if count <= 0 => {
                return Err(TicketRepoError::NoSeatsAvailable);
            }
            Some(_) => {}
        }

        sqlx::query("UPDATE EVENIMENTE SET numarlocuri = numarlocuri - 1 WHERE id = $1")
            .bind(event_id)
            .execute(&mut *tx)
            .await
            .map_err(TicketRepoError::InternalError)?;

        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO BILETE (cod, pachetid, evenimentid)
            VALUES ($1, NULL, $2)
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(new_code)
        .bind(event_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_ticket_error)?;

        tx.commit().await.map_err(TicketRepoError::InternalError)?;

        Ok(ticket)
    }

    pub async fn get_ticket(&self, cod: &str) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT cod, pachetid, evenimentid
            FROM BILETE
            WHERE cod = $1
            "#,
        )
        .bind(cod)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn list_tickets(&self) -> Result<Vec<Ticket>, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT * FROM BILETE
            "#,
        )
        .fetch_all(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }
    pub async fn update_ticket(
        &self,
        cod: &str,
        payload: UpdateTicket,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE BILETE
            SET
                pachetid = $1,
                evenimentid = $2
            WHERE COD = $3
            RETURNING COD, pachetid, evenimentid
            "#,
        )
        .bind(payload.id_pachet)
        .bind(payload.id_event)
        .bind(cod)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn update_ticket_for_event(
        &self,
        event_id: i32,
        cod: &str,
        payload: UpdateTicket,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE BILETE
            SET
                pachetid = $1,
                evenimentid = NULL
            WHERE
                cod = $2 and evenimentid = $3
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(payload.id_pachet)
        .bind(cod)
        .bind(event_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn delete_ticket(&self, cod: &str) -> Result<(), TicketRepoError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(TicketRepoError::InternalError)?;

        let ticket: Option<Ticket> = sqlx::query_as::<_, Ticket>(
            "SELECT cod, pachetid, evenimentid FROM BILETE WHERE cod = $1",
        )
        .bind(cod)
        .fetch_optional(&mut *tx)
        .await
        .map_err(TicketRepoError::InternalError)?;

        let ticket = ticket.ok_or(TicketRepoError::NotFound)?;

        sqlx::query("DELETE FROM BILETE WHERE cod = $1")
            .bind(cod)
            .execute(&mut *tx)
            .await
            .map_err(TicketRepoError::InternalError)?;

        if let Some(event_id) = ticket.id_event {
            sqlx::query("UPDATE EVENIMENTE SET numarlocuri = numarlocuri + 1 WHERE id = $1")
                .bind(event_id)
                .execute(&mut *tx)
                .await
                .map_err(TicketRepoError::InternalError)?;
        } else if let Some(packet_id) = ticket.id_pachet {
            let event_ids: Vec<(i32,)> =
                sqlx::query_as("SELECT evenimentid FROM JOIN_PE WHERE pachetid = $1")
                    .bind(packet_id)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(TicketRepoError::InternalError)?;

            for (event_id,) in event_ids {
                sqlx::query("UPDATE EVENIMENTE SET numarlocuri = numarlocuri + 1 WHERE id = $1")
                    .bind(event_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(TicketRepoError::InternalError)?;
            }

            let min_seats: Option<i32> = sqlx::query_scalar(
                r#"
                SELECT MIN(e.numarlocuri)
                FROM EVENIMENTE e
                JOIN JOIN_PE j ON e.id = j.evenimentid
                WHERE j.pachetid = $1
                "#,
            )
            .bind(packet_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(TicketRepoError::InternalError)?
            .flatten();

            sqlx::query("UPDATE PACHETE SET numarlocuri = $1 WHERE id = $2")
                .bind(min_seats)
                .bind(packet_id)
                .execute(&mut *tx)
                .await
                .map_err(TicketRepoError::InternalError)?;
        }

        tx.commit().await.map_err(TicketRepoError::InternalError)?;

        Ok(())
    }

    pub async fn delete_ticket_for_event(
        &self,
        event_id: i32,
        cod: String,
    ) -> Result<(), TicketRepoError> {
        let result = sqlx::query("DELETE FROM BILETE WHERE evenimentid = $1 AND cod = $2")
            .bind(event_id)
            .bind(cod)
            .execute(&self.pool)
            .await
            .map_err(TicketRepoError::InternalError)?;

        if result.rows_affected() == 0 {
            Err(TicketRepoError::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn list_tickets_for_packet(
        &self,
        packet_id: i32,
    ) -> Result<Vec<Ticket>, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT cod, pachetid, evenimentid
            FROM BILETE
            WHERE pachetid = $1
            "#,
        )
        .bind(packet_id)
        .fetch_all(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn get_ticket_for_packet(
        &self,
        packet_id: i32,
        cod: &str,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT cod, pachetid, evenimentid
            FROM BILETE
            WHERE pachetid = $1 AND cod = $2
            "#,
        )
        .bind(packet_id)
        .bind(cod)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn create_ticket_for_packet(
        &self,
        packet_id: i32,
    ) -> Result<Ticket, TicketRepoError> {
        let new_code = Uuid::now_v7().to_string();

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(TicketRepoError::InternalError)?;

        let events: Vec<(i32, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT e.id, e.numarlocuri
            FROM EVENIMENTE e
            JOIN JOIN_PE j ON e.id = j.evenimentid
            WHERE j.pachetid = $1
            "#,
        )
        .bind(packet_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(TicketRepoError::InternalError)?;

        if events.is_empty() {
            return Err(TicketRepoError::InvalidReference);
        }

        let min_seats = events
            .iter()
            .filter_map(|(_, seats)| *seats)
            .min()
            .unwrap_or(0);

        if min_seats <= 0 {
            return Err(TicketRepoError::NoSeatsAvailable);
        }

        for (event_id, _) in &events {
            sqlx::query("UPDATE EVENIMENTE SET numarlocuri = numarlocuri - 1 WHERE id = $1")
                .bind(event_id)
                .execute(&mut *tx)
                .await
                .map_err(TicketRepoError::InternalError)?;
        }

        let new_min = min_seats - 1;
        sqlx::query("UPDATE PACHETE SET numarlocuri = $1 WHERE id = $2")
            .bind(new_min)
            .bind(packet_id)
            .execute(&mut *tx)
            .await
            .map_err(TicketRepoError::InternalError)?;

        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO BILETE (cod, pachetid, evenimentid)
            VALUES ($1, $2, NULL)
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(new_code)
        .bind(packet_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx_ticket_error)?;

        tx.commit().await.map_err(TicketRepoError::InternalError)?;

        Ok(ticket)
    }

    pub async fn update_ticket_for_packet(
        &self,
        packet_id: i32,
        cod: &str,
        payload: UpdateTicket,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE BILETE
            SET
                pachetid = NULL,
                evenimentid = $1
            WHERE
                cod = $2 AND pachetid = $3
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(payload.id_event)
        .bind(cod)
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn delete_ticket_for_packet(
        &self,
        packet_id: i32,
        cod: &str,
    ) -> Result<(), TicketRepoError> {
        let result = sqlx::query("DELETE FROM BILETE WHERE pachetid = $1 AND cod = $2")
            .bind(packet_id)
            .bind(cod)
            .execute(&self.pool)
            .await
            .map_err(TicketRepoError::InternalError)?;

        if result.rows_affected() == 0 {
            Err(TicketRepoError::NotFound)
        } else {
            Ok(())
        }
    }
}
