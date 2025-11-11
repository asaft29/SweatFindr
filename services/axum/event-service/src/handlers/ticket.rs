use crate::AppState;
use crate::models::ticket::{CreateTicket, Ticket, UpdateTicket};
use crate::shared::error::ApiError;
use crate::shared::links;
use crate::shared::links::{Response, build_ticket_over_event, build_ticket_over_packet};
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/event-manager/tickets/{cod}",
    params(
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket found", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let ticket = state.ticket_repo.get_ticket(&cod).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/tickets",
    responses(
        (status = 200, description = "List of all tickets", body = [Response<Ticket>]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let tickets = state.ticket_repo.list_tickets().await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|e| links::build_simple_ticket(e, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

#[utoipa::path(
    put,
    path = "/api/event-manager/tickets/{cod}",
    request_body = UpdateTicket,
    params(
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket updated", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
    payload: Result<Json<UpdateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state.ticket_repo.update_ticket(&cod, payload).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/tickets",
    request_body = CreateTicket,
    responses(
        (status = 201, description = "Ticket created", body = Response<Ticket>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<CreateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state.ticket_repo.create_ticket(payload).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/tickets/{cod}",
    params(
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 204, description = "Ticket deleted"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    state.ticket_repo.delete_ticket(&cod).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/event-manager/events/{event_id}/tickets/{ticket_cod}",
    params(
        ("event_id" = i32, Path, description = "Event ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Get ticket for event", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn get_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let ticket = state
        .ticket_repo
        .get_ticket_for_event(event_id, &ticket_cod)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/events/{event_id}/tickets",
    params(
        ("event_id" = i32, Path, description = "Event ID")
    ),
    responses(
        (status = 200, description = "List of tickets for the event", body = [Response<Ticket>]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn list_tickets_for_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let tickets = state.ticket_repo.list_tickets_for_event(event_id).await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|t| build_ticket_over_event(t, event_id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

#[utoipa::path(
    put,
    path = "/api/event-manager/events/{event_id}/tickets/{ticket_cod}",
    request_body = UpdateTicket,
    params(
        ("event_id" = i32, Path, description = "Event ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket updated for event", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn update_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
    payload: Result<Json<UpdateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state
        .ticket_repo
        .update_ticket_for_event(event_id, &ticket_cod, payload)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/events/{event_id}/tickets",
    request_body = CreateTicket,
    params(
        ("event_id" = i32, Path, description = "Event ID")
    ),
    responses(
        (status = 201, description = "Ticket created for event", body = Response<Ticket>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn create_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<i32>,
    payload: Result<Json<CreateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state
        .ticket_repo
        .create_ticket_for_event(event_id, payload)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/events/{event_id}/tickets/{ticket_cod}",
    params(
        ("event_id" = i32, Path, description = "Event ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 204, description = "Ticket deleted for event"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn delete_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    state
        .ticket_repo
        .delete_ticket_for_event(event_id, ticket_cod)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/event-manager/event-packets/{packet_id}/tickets",
    params(
        ("packet_id" = i32, Path, description = "Packet ID")
    ),
    responses(
        (status = 200, description = "List of tickets for the packet", body = [Response<Ticket>]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn list_tickets_for_packet(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let tickets = state.ticket_repo.list_tickets_for_packet(packet_id).await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|t| build_ticket_over_packet(t, packet_id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/event-packets/{packet_id}/tickets/{ticket_cod}",
    params(
        ("packet_id" = i32, Path, description = "Packet ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Get ticket for packet", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn get_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let ticket = state
        .ticket_repo
        .get_ticket_for_packet(packet_id, &ticket_cod)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/event-packets/{packet_id}/tickets",
    request_body = CreateTicket,
    params(
        ("packet_id" = i32, Path, description = "Packet ID")
    ),
    responses(
        (status = 201, description = "Ticket created for packet", body = Response<Ticket>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn create_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<i32>,
    payload: Result<Json<CreateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state
        .ticket_repo
        .create_ticket_for_packet(packet_id, payload)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

#[utoipa::path(
    put,
    path = "/api/event-manager/event-packets/{packet_id}/tickets/{ticket_cod}",
    request_body = UpdateTicket,
    params(
        ("packet_id" = i32, Path, description = "Packet ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket updated for packet", body = Response<Ticket>),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn update_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
    payload: Result<Json<UpdateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    let Json(payload) = payload?;

    payload.validate()?;

    let ticket = state
        .ticket_repo
        .update_ticket_for_packet(packet_id, &ticket_cod, payload)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/event-packets/{packet_id}/tickets/{ticket_cod}",
    params(
        ("packet_id" = i32, Path, description = "Packet ID"),
        ("ticket_cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 204, description = "Ticket deleted for packet"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets"
)]
pub async fn delete_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }
    state
        .ticket_repo
        .delete_ticket_for_packet(packet_id, &ticket_cod)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn ticket_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tickets", post(create_ticket).get(list_tickets))
        .route(
            "/tickets/{cod}",
            get(get_ticket).put(update_ticket).delete(delete_ticket),
        )
}
