use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use validator::Validate;

use crate::AppState;
use crate::models::client::{
    AddTicket, Client, ClientQuery, CreateClient, TicketRef, UpdateClient,
};
use crate::services::event_service;
use crate::shared::error::ApiError;
use crate::shared::links::{client_links, ticket_ref_links, Response};

pub fn client_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/clients", get(list_clients).post(create_client))
        .route(
            "/clients/{id}",
            get(get_client)
                .put(update_client)
                .patch(patch_client)
                .delete(delete_client),
        )
        .route(
            "/clients/{id}/tickets",
            get(get_client_tickets).post(add_ticket_to_client),
        )
        .route(
            "/clients/{id}/tickets/{cod}",
            delete(remove_ticket_from_client),
        )
}

#[utoipa::path(
    get,
    path = "/api/client-manager/clients",
    params(
        ("email" = Option<String>, Query, description = "Filter by client email"),
        ("prenume" = Option<String>, Query, description = "Filter by first name"),
        ("nume" = Option<String>, Query, description = "Filter by last name")
    ),
    responses(
        (status = 200, description = "List clients (optionally filtered)", body = Vec<Client>),
        (status = 500, description = "Internal server error")
    ),
    tag = "clients"
)]
pub async fn list_clients(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ClientQuery>,
) -> Result<Json<Vec<Response<Client>>>, ApiError> {
    let clients = state.client_repo.list_clients(query).await?;

    let responses: Vec<Response<Client>> = clients
        .into_iter()
        .map(|client| {
            let client_id = client.id.to_hex();
            let links = client_links(&state.base_url, &client_id);
            Response::new(client, links)
        })
        .collect();

    Ok(Json(responses))
}

#[utoipa::path(
    get,
    path = "/api/client-manager/clients/{id}",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    responses(
        (status = 200, description = "Client found", body = Client),
        (status = 404, description = "Client not found"),
        (status = 400, description = "Invalid ID format")
    ),
    tag = "clients"
)]
pub async fn get_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Response<Client>>, ApiError> {
    let client = state.client_repo.get_client(&id).await?;
    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
}

#[utoipa::path(
    post,
    path = "/api/client-manager/clients",
    request_body = CreateClient,
    responses(
        (status = 201, description = "Client created successfully", body = Client),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    tag = "clients"
)]
pub async fn create_client(
    State(state): State<Arc<AppState>>,
    payload: Result<Json<CreateClient>, JsonRejection>,
) -> Result<(StatusCode, Json<Response<Client>>), ApiError> {
    let Json(payload) = payload?;
    payload.validate()?;

    let client = state.client_repo.create_client(payload).await?;
    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok((StatusCode::CREATED, Json(Response::new(client, links))))
}

#[utoipa::path(
    put,
    path = "/api/client-manager/clients/{id}",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    request_body = UpdateClient,
    responses(
        (status = 200, description = "Client updated successfully", body = Client),
        (status = 404, description = "Client not found"),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    tag = "clients"
)]
pub async fn update_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateClient>, JsonRejection>,
) -> Result<Json<Response<Client>>, ApiError> {
    let Json(payload) = payload?;
    payload.validate()?;

    let client = state.client_repo.update_client(&id, payload).await?;
    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
}

#[utoipa::path(
    patch,
    path = "/api/client-manager/clients/{id}",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    request_body = UpdateClient,
    responses(
        (status = 200, description = "Client partially updated successfully", body = Client),
        (status = 404, description = "Client not found"),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    tag = "clients"
)]
pub async fn patch_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateClient>, JsonRejection>,
) -> Result<Json<Response<Client>>, ApiError> {
    let Json(payload) = payload?;
    payload.validate()?;

    let client = state.client_repo.update_client(&id, payload).await?;
    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
}

#[utoipa::path(
    delete,
    path = "/api/client-manager/clients/{id}",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    responses(
        (status = 204, description = "Client deleted successfully"),
        (status = 404, description = "Client not found")
    ),
    tag = "clients"
)]
pub async fn delete_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    state.client_repo.delete_client(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/client-manager/clients/{id}/tickets",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    responses(
        (status = 200, description = "List of client tickets", body = Vec<TicketRef>),
        (status = 404, description = "Client not found")
    ),
    tag = "clients"
)]
pub async fn get_client_tickets(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Response<TicketRef>>>, ApiError> {
    let tickets = state.client_repo.get_client_tickets(&id).await?;

    let responses: Vec<Response<TicketRef>> = tickets
        .into_iter()
        .map(|ticket| {
            let links = ticket_ref_links(&state.base_url, &id, &ticket.cod);
            Response::new(ticket, links)
        })
        .collect();

    Ok(Json(responses))
}

#[utoipa::path(
    post,
    path = "/api/client-manager/clients/{id}/tickets",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)")
    ),
    request_body = AddTicket,
    responses(
        (status = 201, description = "Ticket added to client with auto-populated event/packet details from event service", body = Client),
        (status = 400, description = "Ticket code not found in event service"),
        (status = 404, description = "Client not found"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "External service error")
    ),
    tag = "clients"
)]
pub async fn add_ticket_to_client(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    payload: Result<Json<AddTicket>, JsonRejection>,
) -> Result<(StatusCode, Json<Response<Client>>), ApiError> {
    let Json(payload) = payload?;
    payload.validate()?;

    // Fetch ticket details from event service (validates ticket and gets event/packet info)
    let ticket_details = event_service::get_ticket_details(&state.event_service_url, &payload.cod)
        .await
        .map_err(|e| match e {
            event_service::EventServiceError::TicketNotFound(msg) => ApiError::BadRequest(msg),
            event_service::EventServiceError::HttpError(msg) => ApiError::ExternalServiceError(msg),
            event_service::EventServiceError::DeserializationError(msg) => {
                ApiError::ExternalServiceError(msg)
            }
        })?;

    // Populate ticket with event/packet information from event service
    let enriched_ticket = AddTicket {
        cod: payload.cod,
        nume_eveniment: if let Some(ref event) = ticket_details.event {
            Some(event.nume.clone())
        } else if let Some(ref packet) = ticket_details.packet {
            Some(packet.nume.clone())
        } else {
            payload.nume_eveniment
        },
        locatie: if let Some(ref event) = ticket_details.event {
            Some(event.locatie.clone())
        } else if let Some(ref packet) = ticket_details.packet {
            Some(packet.locatie.clone())
        } else {
            payload.locatie
        },
    };

    let client = state
        .client_repo
        .add_ticket_to_client(&id, enriched_ticket)
        .await?;

    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok((StatusCode::CREATED, Json(Response::new(client, links))))
}

#[utoipa::path(
    delete,
    path = "/api/client-manager/clients/{id}/tickets/{cod}",
    params(
        ("id" = String, Path, description = "Client ID (MongoDB ObjectId)"),
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket removed from client", body = Client),
        (status = 404, description = "Client not found")
    ),
    tag = "clients"
)]
pub async fn remove_ticket_from_client(
    State(state): State<Arc<AppState>>,
    Path((id, cod)): Path<(String, String)>,
) -> Result<Json<Response<Client>>, ApiError> {
    let client = state
        .client_repo
        .remove_ticket_from_client(&id, &cod)
        .await?;

    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
}
