use crate::error::{TicketRepoError, map_sqlx_ticket_error};
use crate::models::ticket::{CreateTicket, Ticket, UpdateTicket};
use anyhow::Result;
use sqlx::PgPool;

pub struct TicketRepo {
    pool: PgPool,
}

impl TicketRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_ticket(&self, payload: CreateTicket) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO BILETE (cod, pachetid, evenimentid)
            VALUES ($1, $2, $3)
            RETURNING cod, pachetid, evenimentid
            "#,
        )
        .bind(payload.cod)
        .bind(payload.id_pachet)
        .bind(payload.id_event)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_ticket_error)
    }

    pub async fn get_ticket(&self, cod: String) -> Result<Ticket, TicketRepoError> {
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
        cod: String,
        payload: UpdateTicket,
    ) -> Result<Ticket, TicketRepoError> {
        let result = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE BILETE
            SET 
                pachetid = COALESCE($1, pachetid),
                evenimentid = COALESCE($2, evenimentid)
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

    pub async fn delete_ticket(&self, cod: String) -> Result<(), TicketRepoError> {
        let result = sqlx::query("DELETE FROM BILETE WHERE cod = $1")
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
