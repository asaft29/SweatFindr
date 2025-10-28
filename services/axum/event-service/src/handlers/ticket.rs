use crate::AppState;
use crate::error::TicketRepoError;
use crate::links::TicketResponse;
use crate::models::ticket::{CreateTicket, UpdateTicket};
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
) -> Result<Json<TicketResponse>, TicketRepoError> {
    let ticket = state.ticket_repo.get_ticket(cod).await?;

    let ticket_response = TicketResponse::new(ticket, &state.base_url);
    Ok(Json(ticket_response))
}

pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>,
    Json(payload): Json<UpdateTicket>,
) -> Result<Json<TicketResponse>, TicketRepoError> {
    let ticket = state.ticket_repo.update_ticket(cod, payload).await?;

    let ticket_response = TicketResponse::new(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

pub async fn create_ticket(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTicket>,
) -> Result<impl IntoResponse, TicketRepoError> {
    let ticket = state.ticket_repo.create_ticket(payload).await?;

    let ticket_response = TicketResponse::new(ticket, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
}

pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Path(cod): Path<String>, // Note: Path<String>
) -> Result<impl IntoResponse, TicketRepoError> {
    state.ticket_repo.delete_ticket(cod).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn ticket_manager_router() -> Router<Arc<AppState>> {
    Router::new().route("/tickets", post(create_ticket)).route(
        "/tickets/{cod}",
        get(get_ticket).put(update_ticket).delete(delete_ticket),
    )
}
