use crate::error::{JoinPeRepoError, map_sqlx_join_pe_error};
use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
use crate::models::join_pe::{AddEventToPacket, AddPacketToEvent, EventPacketRelation};
use anyhow::Result;
use sqlx::PgPool;

pub struct JoinPeRepo {
    pool: PgPool,
}

impl JoinPeRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
            SELECT p.id, p.id_owner, p.nume, p.locatie, p.descriere
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
        eveniment_id: i32,
        payload: AddEventToPacket,
    ) -> Result<EventPacketRelation, JoinPeRepoError> {
        sqlx::query_as::<_, EventPacketRelation>(
            r#"
            INSERT INTO JOIN_PE (pachetid, evenimentid, numarlocuri)
            VALUES ($1, $2, $3)
            RETURNING pachetid, evenimentid, numarlocuri
            "#,
        )
        .bind(payload.id_pachet)
        .bind(eveniment_id)
        .bind(payload.locuri)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)
    }

    pub async fn add_packet_to_event(
        &self,
        pachet_id: i32,
        payload: AddPacketToEvent,
    ) -> Result<EventPacketRelation, JoinPeRepoError> {
        sqlx::query_as::<_, EventPacketRelation>(
            r#"
            INSERT INTO JOIN_PE (pachetid, evenimentid, numarlocuri)
            VALUES ($1, $2, $3)
            RETURNING pachetid, evenimentid, numarlocuri
            "#,
        )
        .bind(pachet_id)
        .bind(payload.id_event)
        .bind(payload.locuri)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_join_pe_error)
    }
}
