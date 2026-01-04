use crate::models::refund::RefundRequest;
use sqlx::{Error, PgPool};

pub struct RefundRepo {
    pool: PgPool,
}

#[derive(Debug)]
pub enum RefundRepoError {
    NotFound,
    InternalError(Error),
}

impl RefundRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_refund_request(
        &self,
        ticket_cod: &str,
        requester_id: i32,
        requester_email: &str,
        event_id: Option<i32>,
        packet_id: Option<i32>,
        event_owner_id: i32,
        reason: &str,
    ) -> Result<RefundRequest, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            INSERT INTO REFUND_REQUESTS
                (ticket_cod, requester_id, requester_email, event_id, packet_id, event_owner_id, reason)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                      event_owner_id, status, reason, rejection_message,
                      created_at::text, resolved_at::text
            "#,
        )
        .bind(ticket_cod)
        .bind(requester_id)
        .bind(requester_email)
        .bind(event_id)
        .bind(packet_id)
        .bind(event_owner_id)
        .bind(reason)
        .fetch_one(&self.pool)
        .await;

        result.map_err(RefundRepoError::InternalError)
    }

    pub async fn get_refund_request(&self, id: i32) -> Result<RefundRequest, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            SELECT id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                   event_owner_id, status, reason, rejection_message,
                   created_at::text, resolved_at::text
            FROM REFUND_REQUESTS
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(request) => Ok(request),
            Err(Error::RowNotFound) => Err(RefundRepoError::NotFound),
            Err(e) => Err(RefundRepoError::InternalError(e)),
        }
    }

    pub async fn list_pending_refunds_for_owner(
        &self,
        owner_id: i32,
    ) -> Result<Vec<RefundRequest>, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            SELECT id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                   event_owner_id, status, reason, rejection_message,
                   created_at::text, resolved_at::text
            FROM REFUND_REQUESTS
            WHERE event_owner_id = $1 AND status = 'PENDING'
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await;

        result.map_err(RefundRepoError::InternalError)
    }

    pub async fn approve_refund(
        &self,
        id: i32,
        owner_id: i32,
    ) -> Result<RefundRequest, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            UPDATE REFUND_REQUESTS
            SET status = 'APPROVED', resolved_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND event_owner_id = $2 AND status = 'PENDING'
            RETURNING id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                      event_owner_id, status, reason, rejection_message,
                      created_at::text, resolved_at::text
            "#,
        )
        .bind(id)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(request) => Ok(request),
            Err(Error::RowNotFound) => Err(RefundRepoError::NotFound),
            Err(e) => Err(RefundRepoError::InternalError(e)),
        }
    }

    pub async fn reject_refund(
        &self,
        id: i32,
        owner_id: i32,
        message: &str,
    ) -> Result<RefundRequest, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            UPDATE REFUND_REQUESTS
            SET status = 'REJECTED', rejection_message = $3, resolved_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND event_owner_id = $2 AND status = 'PENDING'
            RETURNING id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                      event_owner_id, status, reason, rejection_message,
                      created_at::text, resolved_at::text
            "#,
        )
        .bind(id)
        .bind(owner_id)
        .bind(message)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(request) => Ok(request),
            Err(Error::RowNotFound) => Err(RefundRepoError::NotFound),
            Err(e) => Err(RefundRepoError::InternalError(e)),
        }
    }

    pub async fn get_event_name_for_refund(&self, refund: &RefundRequest) -> Option<String> {
        if let Some(event_id) = refund.event_id {
            sqlx::query_scalar::<_, String>("SELECT nume FROM EVENIMENTE WHERE id = $1")
                .bind(event_id)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()
        } else if let Some(packet_id) = refund.packet_id {
            sqlx::query_scalar::<_, String>("SELECT nume FROM PACHETE WHERE id = $1")
                .bind(packet_id)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten()
        } else {
            None
        }
    }

    pub async fn list_refunds_by_requester_email(
        &self,
        email: &str,
    ) -> Result<Vec<RefundRequest>, RefundRepoError> {
        let result = sqlx::query_as::<_, RefundRequest>(
            r#"
            SELECT id, ticket_cod, requester_id, requester_email, event_id, packet_id,
                   event_owner_id, status, reason, rejection_message,
                   created_at::text, resolved_at::text
            FROM REFUND_REQUESTS
            WHERE requester_email = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(email)
        .fetch_all(&self.pool)
        .await;

        result.map_err(RefundRepoError::InternalError)
    }
}
