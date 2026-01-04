use crate::models::event_packets::{
    CreateEventPacket, EventPacketQuery, EventPackets, PaginationParams, UpdateEventPacket,
};
use crate::utils::error::*;
use anyhow::Result;
use sqlx::{Error, PgPool, Postgres, QueryBuilder};

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_ITEMS_PER_PAGE: i64 = 10;

pub struct EventPacketRepo {
    pool: PgPool,
}

impl EventPacketRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_event_packets(
        &self,
        params: EventPacketQuery,
    ) -> Result<Vec<EventPackets>, EventPacketRepoError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT ID, ID_OWNER, nume, locatie, descriere, numarlocuri FROM PACHETE",
        );

        let mut has_condition = false;

        let type_filter = params.descriere.filter(|s| !s.is_empty());

        if let Some(desc_filter) = type_filter {
            query_builder.push(" WHERE unaccent(descriere) ILIKE unaccent(");
            query_builder.push_bind(format!("%{}%", desc_filter));
            query_builder.push(")");
            has_condition = true;
        }

        if let Some(min_tickets) = params.bilete {
            if has_condition {
                query_builder.push(" AND ");
            } else {
                query_builder.push(" WHERE ");
            }

            query_builder.push(" PACHETE.numarlocuri >= ");
            query_builder.push_bind(min_tickets);
        }

        self.apply_pagination(&mut query_builder, params.paginare);

        let query = query_builder.build_query_as::<EventPackets>();
        let packets = query
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_packet_error)?;

        Ok(packets)
    }

    fn apply_pagination(
        &self,
        query_builder: &mut QueryBuilder<Postgres>,
        pagination: PaginationParams,
    ) {
        query_builder.push(" ORDER BY nume ASC");

        let page = pagination.page.unwrap_or(DEFAULT_PAGE).max(DEFAULT_PAGE);

        let items_per_page = pagination
            .items_per_page
            .unwrap_or(DEFAULT_ITEMS_PER_PAGE)
            .max(1);

        let offset = (page - 1) * items_per_page;

        query_builder.push(" LIMIT ");
        query_builder.push_bind(items_per_page);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);
    }

    pub async fn get_event_packet(
        &self,
        packet_id: i32,
    ) -> Result<EventPackets, EventPacketRepoError> {
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            SELECT id, id_owner, nume, locatie, descriere, numarlocuri
            FROM PACHETE
            WHERE id = $1
            "#,
        )
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(packet) => Ok(packet),
            Err(Error::RowNotFound) => Err(EventPacketRepoError::NotFound),
            Err(e) => Err(EventPacketRepoError::InternalError(e)),
        }
    }

    pub async fn create_event_packet(
        &self,
        id_owner: i32,
        payload: CreateEventPacket,
    ) -> Result<EventPackets, EventPacketRepoError> {
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            INSERT INTO PACHETE (id_owner, nume, locatie, descriere, numarlocuri)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, id_owner, nume, locatie, descriere, numarlocuri
            "#,
        )
        .bind(id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .bind(payload.numarlocuri)
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
            SET
                id_owner = COALESCE($1, id_owner),
                nume = COALESCE($2, nume),
                locatie = COALESCE($3, locatie),
                descriere = COALESCE($4, descriere),
                numarlocuri = COALESCE($5, numarlocuri)
            WHERE id = $6
            RETURNING id, id_owner, nume, locatie, descriere, numarlocuri
            "#,
        )
        .bind(payload.id_owner)
        .bind(&payload.nume)
        .bind(&payload.locatie)
        .bind(&payload.descriere)
        .bind(payload.numarlocuri)
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_packet_error)
    }

    pub async fn patch_event_packet(
        &self,
        packet_id: i32,
        payload: crate::models::event_packets::PatchEventPacket,
    ) -> Result<EventPackets, EventPacketRepoError> {
        let result = sqlx::query_as::<_, EventPackets>(
            r#"
            UPDATE PACHETE
            SET
                id_owner = COALESCE($1, id_owner),
                nume = COALESCE($2, nume),
                locatie = COALESCE($3, locatie),
                descriere = COALESCE($4, descriere),
                numarlocuri = COALESCE($5, numarlocuri)
            WHERE id = $6
            RETURNING id, id_owner, nume, locatie, descriere, numarlocuri
            "#,
        )
        .bind(payload.id_owner)
        .bind(payload.nume.as_deref())
        .bind(payload.locatie.as_deref())
        .bind(payload.descriere.as_deref())
        .bind(payload.numarlocuri)
        .bind(packet_id)
        .fetch_one(&self.pool)
        .await;

        result.map_err(map_sqlx_packet_error)
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
