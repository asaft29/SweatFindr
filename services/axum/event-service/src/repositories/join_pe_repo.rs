use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
use crate::models::join_pe::EventPacketRelation;
use crate::utils::error::{JoinPeRepoError, map_sqlx_join_pe_error};
use anyhow::Result;
use rayon::prelude::*;
use sqlx::PgPool;

pub struct JoinPeRepo {
    pool: PgPool,
}

impl JoinPeRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn calculate_packet_effective_capacity(
        &self,
        pachet_id: i32,
    ) -> Result<Option<i32>, JoinPeRepoError> {
        let events = self.get_events_for_packet(pachet_id).await?;

        if events.is_empty() {
            return Ok(None);
        }

        let min_capacity = events.par_iter().filter_map(|event| event.locuri).min();

        Ok(min_capacity)
    }

    async fn update_packet_capacity(
        &self,
        pachet_id: i32,
        new_capacity: Option<i32>,
    ) -> Result<(), JoinPeRepoError> {
        sqlx::query(
            r#"
            UPDATE PACHETE
            SET numarlocuri = $1
            WHERE id = $2
            "#,
        )
        .bind(new_capacity)
        .bind(pachet_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)?;

        Ok(())
    }

    pub async fn get_events_for_packet(
        &self,
        pachet_id: i32,
    ) -> Result<Vec<Event>, JoinPeRepoError> {
        sqlx::query_as::<_, Event>(
            r#"
            SELECT e.id, e.id_owner, e.nume, e.locatie, e.descriere, e.numarlocuri
            FROM EVENIMENTE e
            JOIN JOIN_PE j ON e.id = j.evenimentid
            WHERE j.pachetid = $1
            "#,
        )
        .bind(pachet_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)
    }

    pub async fn get_packets_for_event(
        &self,
        eveniment_id: i32,
    ) -> Result<Vec<EventPackets>, JoinPeRepoError> {
        sqlx::query_as::<_, EventPackets>(
            r#"
            SELECT p.id, p.id_owner, p.nume, p.locatie, p.descriere, p.numarlocuri
            FROM PACHETE p
            JOIN JOIN_PE j ON p.id = j.pachetid
            WHERE j.evenimentid = $1
            "#,
        )
        .bind(eveniment_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)
    }

    pub async fn add_event_to_packet(
        &self,
        pachet_id: i32,
        eveniment_id: i32,
    ) -> Result<EventPacketRelation, JoinPeRepoError> {
        let relation = sqlx::query_as::<_, EventPacketRelation>(
            r#"
            INSERT INTO JOIN_PE (pachetid, evenimentid)
            VALUES ($1, $2)
            RETURNING pachetid, evenimentid
            "#,
        )
        .bind(pachet_id)
        .bind(eveniment_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)?;

        let effective_capacity = self.calculate_packet_effective_capacity(pachet_id).await?;

        self.update_packet_capacity(pachet_id, effective_capacity)
            .await?;

        Ok(relation)
    }

    pub async fn remove_event_from_packet(
        &self,
        pachet_id: i32,
        eveniment_id: i32,
    ) -> Result<(), JoinPeRepoError> {
        let packet_exists = sqlx::query("SELECT 1 FROM PACHETE WHERE id = $1")
            .bind(pachet_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_join_pe_error)?;

        if packet_exists.is_none() {
            return Err(JoinPeRepoError::InvalidPacket);
        }

        let event_exists = sqlx::query("SELECT 1 FROM EVENIMENTE WHERE id = $1")
            .bind(eveniment_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_join_pe_error)?;

        if event_exists.is_none() {
            return Err(JoinPeRepoError::InvalidEvent);
        }

        let result = sqlx::query(
            r#"
            DELETE FROM JOIN_PE
            WHERE pachetid = $1 AND evenimentid = $2
            "#,
        )
        .bind(pachet_id)
        .bind(eveniment_id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)?;

        if result.rows_affected() == 0 {
            return Err(JoinPeRepoError::NotFound);
        }

        let effective_capacity = self.calculate_packet_effective_capacity(pachet_id).await?;

        self.update_packet_capacity(pachet_id, effective_capacity)
            .await?;

        Ok(())
    }

    pub async fn update_packets_before_event_deletion(
        &self,
        eveniment_id: i32,
    ) -> Result<(), JoinPeRepoError> {
        let packets = self.get_packets_for_event(eveniment_id).await?;

        for packet in packets {
            sqlx::query(
                r#"
                DELETE FROM JOIN_PE
                WHERE pachetid = $1 AND evenimentid = $2
                "#,
            )
            .bind(packet.id)
            .bind(eveniment_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_join_pe_error)?;

            let effective_capacity = self.calculate_packet_effective_capacity(packet.id).await?;

            self.update_packet_capacity(packet.id, effective_capacity)
                .await?;
        }

        Ok(())
    }
}
