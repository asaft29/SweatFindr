use crate::error::*;
use crate::models::event_packets::{CreateEventPacket, EventPackets, UpdateEventPacket};
use anyhow::Result;
use sqlx::{Error, PgPool};

pub struct EventPacketRepo {
    pool: PgPool,
}

impl EventPacketRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_event_packet(
        &self,
        packet_id: i32,
    ) -> Result<EventPackets, EventPacketRepoError> {
        // <-- Renamed here
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            SELECT id, id_owner, nume, locatie, descriere
            FROM PACHETE
            WHERE id = $1
            "#,
        )
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(packet) => Ok(packet),
            Err(Error::RowNotFound) => Err(EventPacketRepoError::NotFound), // <-- Renamed here
            Err(e) => Err(EventPacketRepoError::InternalError(e)),          // <-- Renamed here
        }
    }

    pub async fn create_event_packet(
        &self,
        payload: CreateEventPacket,
    ) -> Result<EventPackets, EventPacketRepoError> {
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            INSERT INTO PACHETE (id_owner, nume, locatie, descriere)
            VALUES ($1, $2, $3, $4)
            RETURNING id, id_owner, nume, locatie, descriere
            "#,
        )
        .bind(payload.id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_packet_error)
    }

    pub async fn update_event_packet(
        &self,
        packet_id: i32,
        payload: UpdateEventPacket,
    ) -> Result<EventPackets, EventPacketRepoError> {
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            UPDATE PACHETE
            SET id_owner = COALESCE($1, id_owner), nume = $2, locatie = $3, descriere = $4
            WHERE id = $5
            RETURNING id, id_owner, nume, locatie, descriere
            "#,
        )
        .bind(payload.id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(packet) => Ok(packet),
            Err(Error::RowNotFound) => Err(EventPacketRepoError::NotFound),
            Err(e) => Err(EventPacketRepoError::InternalError(e)),
        }
    }

    pub async fn delete_event_packet(&self, packet_id: i32) -> Result<(), EventPacketRepoError> {
        let result = sqlx::query("DELETE FROM PACHETE WHERE id = $1")
            .bind(packet_id)
            .execute(&self.pool)
            .await
            .map_err(EventPacketRepoError::InternalError)?;

        if result.rows_affected() == 0 {
            Err(EventPacketRepoError::NotFound)
        } else {
            Ok(())
        }
    }
}
