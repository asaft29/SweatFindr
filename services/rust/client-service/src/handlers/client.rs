use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Extension, Json, Router};
use validator::Validate;

use crate::AppState;
use crate::middleware::{Authorization, UserClaims};
use crate::models::client::{AddTicket, Client, ClientQuery, TicketRef, UpdateClient};
use crate::services::event_service;
use crate::utils::error::{ClientApiError, map_event_service_error, map_authorization_error};
use crate::utils::links::{Response, client_links, ticket_ref_links};

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::auth_service_client::AuthServiceClient;

async fn get_user_email(state: &AppState, user_id: i32) -> Option<String> {
    let mut auth_client = AuthServiceClient::connect(state.auth_service_url.clone())
        .await
        .ok()?;

    let request = auth::GetUserEmailRequest { user_id };

    let response = auth_client.get_user_email(request).await.ok()?.into_inner();

    if response.success {
        Some(response.email)
    } else {
        None
    }
}

pub fn client_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/clients", get(list_clients))
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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 403, description = "Forbidden - Admin role required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn list_clients(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<UserClaims>,
    Query(query): Query<ClientQuery>,
) -> Result<Json<Vec<Response<Client>>>, ClientApiError> {
    Authorization::can_list_all(&claims).map_err(map_authorization_error)?;

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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 403, description = "Forbidden - Can only access own profile unless admin"),
        (status = 404, description = "Client not found"),
        (status = 400, description = "Invalid ID format")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn get_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
) -> Result<Json<Response<Client>>, ClientApiError> {
    let client = state.client_repo.get_client(&id).await?;

    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_access_resource(&user_claims, &client, user_email.as_deref())
        .map_err(map_authorization_error)?;

    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found"),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn update_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateClient>, JsonRejection>,
) -> Result<Json<Response<Client>>, ClientApiError> {
    let existing_client = state.client_repo.get_client(&id).await?;

    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_modify_resource(&user_claims, &existing_client, user_email.as_deref())
        .map_err(map_authorization_error)?;

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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found"),
        (status = 422, description = "Validation error"),
        (status = 409, description = "Email already exists")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn patch_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateClient>, JsonRejection>,
) -> Result<Json<Response<Client>>, ClientApiError> {
    let existing_client = state.client_repo.get_client(&id).await?;

    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_modify_resource(&user_claims, &existing_client, user_email.as_deref())
        .map_err(map_authorization_error)?;

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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 403, description = "Forbidden - Admin role required"),
        (status = 404, description = "Client not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn delete_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
) -> Result<StatusCode, ClientApiError> {
    Authorization::can_delete_resource(&user_claims)
        .map_err(map_authorization_error)?;

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
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn get_client_tickets(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Response<TicketRef>>>, ClientApiError> {
    let client = state.client_repo.get_client(&id).await?;
    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_access_resource(&user_claims, &client, user_email.as_deref())
        .map_err(map_authorization_error)?;

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
        (status = 201, description = "Ticket created and added to client. Event/packet details auto-populated from event service. Seat count decremented.", body = Client),
        (status = 400, description = "Invalid event/packet ID"),
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found or event/packet not found"),
        (status = 409, description = "No seats available"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "External service error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn add_ticket_to_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(id): Path<String>,
    payload: Result<Json<AddTicket>, JsonRejection>,
) -> Result<(StatusCode, Json<Response<Client>>), ClientApiError> {
    let client = state.client_repo.get_client(&id).await?;
    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_modify_resource(&user_claims, &client, user_email.as_deref())
        .map_err(map_authorization_error)?;

    let Json(payload) = payload?;
    payload.validate()?;

    let ticket_details = match (payload.id_event, payload.id_pachet) {
        (Some(event_id), _) => {
            event_service::create_ticket_for_event(&state.event_service_url, event_id)
                .await
                .map_err(map_event_service_error)?
        }
        (_, Some(packet_id)) => {
            event_service::create_ticket_for_packet(&state.event_service_url, packet_id)
                .await
                .map_err(map_event_service_error)?
        }
        _ => {
            return Err(ClientApiError::BadRequest(
                "Must specify either evenimentid or pachetid".to_string(),
            ));
        }
    };

    let ticket_ref = TicketRef {
        cod: ticket_details.ticket.cod,
        nume_eveniment: ticket_details
            .event
            .as_ref()
            .map(|e| e.nume.clone())
            .or_else(|| ticket_details.packet.as_ref().map(|p| p.nume.clone())),
        locatie: ticket_details
            .event
            .as_ref()
            .map(|e| e.locatie.clone())
            .or_else(|| ticket_details.packet.as_ref().map(|p| p.locatie.clone())),
        descriere: ticket_details
            .event
            .as_ref()
            .map(|e| e.descriere.clone())
            .or_else(|| ticket_details.packet.as_ref().map(|p| p.descriere.clone())),
    };

    let client = state
        .client_repo
        .add_ticket_ref_to_client(&id, ticket_ref)
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
        (status = 200, description = "Ticket removed from client and deleted from event-service. Seat count incremented.", body = Client),
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found or ticket not found")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn remove_ticket_from_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((id, cod)): Path<(String, String)>,
) -> Result<Json<Response<Client>>, ClientApiError> {
    let client = state.client_repo.get_client(&id).await?;
    let user_email = get_user_email(&state, user_claims.user_id).await;
    Authorization::can_modify_resource(&user_claims, &client, user_email.as_deref())
        .map_err(map_authorization_error)?;

    event_service::delete_ticket(&state.event_service_url, &cod)
        .await
        .map_err(map_event_service_error)?;

    let client = state
        .client_repo
        .remove_ticket_from_client(&id, &cod)
        .await?;

    let client_id = client.id.to_hex();
    let links = client_links(&state.base_url, &client_id);

    Ok(Json(Response::new(client, links)))
}
