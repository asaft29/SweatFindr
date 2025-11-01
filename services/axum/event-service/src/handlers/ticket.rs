use crate::error::TicketRepoError;
use crate::links::{Response, build_ticket_over_event};
use crate::models::ticket::{CreateTicket, Ticket, UpdateTicket};
use crate::{AppState, links};
use axum::response::IntoResponse;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

pub async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state.ticket_repo.get_ticket(&cod).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn get_tickets(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let tickets = state.ticket_repo.list_tickets().await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|e| links::build_simple_ticket(e, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}
pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
    Json(payload): Json<UpdateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state.ticket_repo.update_ticket(&cod, payload).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state.ticket_repo.create_ticket(payload).await?;

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, TicketRepoError> {
    state.ticket_repo.delete_ticket(&cod).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .get_ticket_for_event(event_id, &ticket_cod)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn get_tickets_for_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<i32>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let tickets = state.ticket_repo.list_tickets_for_event(event_id).await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|t| build_ticket_over_event(t, event_id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

pub async fn update_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
    Json(payload): Json<UpdateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .update_ticket_for_event(event_id, &ticket_cod, payload)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn create_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path(event_id): Path<i32>,
    Json(payload): Json<CreateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state
        .ticket_repo
        .create_ticket_for_event(event_id, payload)
        .await?;

    let ticket_response = build_ticket_over_event(ticket, event_id, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

pub async fn delete_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, TicketRepoError> {
    state
        .ticket_repo
        .delete_ticket_for_event(event_id, ticket_cod)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn ticket_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tickets", post(create_ticket).get(get_tickets))
        .route(
            "/tickets/{cod}",
            get(get_ticket).put(update_ticket).delete(delete_ticket),
        )
}
