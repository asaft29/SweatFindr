use std::sync::Arc;

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Extension, Json, Router};
use validator::Validate;

use crate::AppState;
use crate::middleware::{Authorization, UserClaims};
use crate::models::client::{AddTicket, Client, ClientQuery, CreateClient, TicketRef, UpdateClient};
use crate::services::event_service;
use crate::utils::error::{ClientApiError, map_authorization_error, map_event_service_error};
use crate::utils::links::{Response, build_simple_client, build_filtered_client, build_ticket_ref};

pub mod auth {
    tonic::include_proto!("auth");
}

use auth::auth_service_client::AuthServiceClient;

async fn get_user_email(state: &AppState, user_id: i32) -> Option<String> {
    tracing::info!("Getting email for user_id: {}", user_id);

    let mut auth_client = match AuthServiceClient::connect(state.auth_service_url.clone()).await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to auth service: {}", e);
            return None;
        }
    };

    let request = auth::GetUserEmailRequest { user_id };

    let response = match auth_client.get_user_email(request).await {
        Ok(resp) => resp.into_inner(),
        Err(e) => {
            tracing::error!("Failed to get user email from auth service: {}", e);
            return None;
        }
    };

    if response.success {
        tracing::info!("Successfully got email: {}", response.email);
        Some(response.email)
    } else {
        tracing::warn!("Auth service returned failure: {}", response.message);
        None
    }
}

async fn create_ticket_via_event_service(
    state: &AppState,
    payload: &AddTicket,
) -> Result<event_service::TicketDetails, ClientApiError> {
    match (payload.id_event, payload.id_pachet) {
        (Some(event_id), None) => event_service::create_ticket_for_event(
            &state.event_manager_client,
            event_id,
            &state.service_token,
        )
        .await
        .map_err(map_event_service_error),
        (None, Some(packet_id)) => event_service::create_ticket_for_packet(
            &state.event_manager_client,
            packet_id,
            &state.service_token,
        )
        .await
        .map_err(map_event_service_error),
        _ => Err(ClientApiError::BadRequest(
            "Must specify either evenimentid or pachetid".to_string(),
        )),
    }
}

fn build_ticket_ref_from_details(ticket_details: &event_service::TicketDetails) -> TicketRef {
    TicketRef {
        cod: ticket_details.ticket.cod.clone(),
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
    }
}

async fn add_ticket_with_rollback(
    state: &AppState,
    client_id: &str,
    ticket_ref: TicketRef,
    ticket_code: &str,
) -> Result<Client, ClientApiError> {
    match state
        .client_repo
        .add_ticket_ref_to_client(client_id, ticket_ref)
        .await
    {
        Ok(client) => Ok(client),
        Err(mongo_error) => {
            tracing::error!(
                "Failed to add ticket {} to client {}. Rolling back ticket creation: {:?}",
                ticket_code,
                client_id,
                mongo_error
            );

            if let Err(delete_error) = event_service::delete_ticket(
                &state.event_manager_client,
                ticket_code,
                &state.service_token,
            )
            .await
            {
                tracing::error!(
                    "CRITICAL: Failed to rollback ticket {} from event-service: {:?}. Manual cleanup required!",
                    ticket_code,
                    delete_error
                );
            } else {
                tracing::info!(
                    "Successfully rolled back ticket {} from event-service",
                    ticket_code
                );
            }

            Err(ClientApiError::Client(mongo_error))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/client-manager/clients",
    request_body = CreateClient,
    responses(
        (status = 201, description = "Client created successfully", body = Response<Client>),
        (status = 400, description = "Invalid request - Validation failed"),
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 409, description = "Conflict - Client with this email already exists")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_client(
    State(state): State<Arc<AppState>>,
    result: Result<Json<CreateClient>, JsonRejection>,
) -> Result<(StatusCode, Json<Response<Client>>), ClientApiError> {
    let Json(payload) = result?;
    payload.validate()?;

    let created_client = state.client_repo.create_client(payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(build_simple_client(created_client, &state.base_url)),
    ))
}

pub fn client_manager_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/clients", get(list_clients).post(create_client))
        .route("/clients/me", get(get_my_client))
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

    let clients = state.client_repo.list_clients(query.clone()).await?;

    let has_filters = query.email.is_some() || query.prenume.is_some() || query.nume.is_some();

    let responses: Vec<Response<Client>> = if has_filters {
        build_filtered_client(clients, &query, &state.base_url)
    } else {
        clients
            .into_iter()
            .map(|client| build_simple_client(client, &state.base_url))
            .collect()
    };

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

    Ok(Json(build_simple_client(client, &state.base_url)))
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

    Ok(Json(build_simple_client(client, &state.base_url)))
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

    Ok(Json(build_simple_client(client, &state.base_url)))
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
    Authorization::can_delete_resource(&user_claims).map_err(map_authorization_error)?;

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
        .map(|ticket| build_ticket_ref(ticket, &id, &state.base_url))
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

    let ticket_details = create_ticket_via_event_service(&state, &payload).await?;
    let ticket_code = ticket_details.ticket.cod.clone();
    let ticket_ref = build_ticket_ref_from_details(&ticket_details);

    let client = add_ticket_with_rollback(&state, &id, ticket_ref, &ticket_code).await?;

    Ok((StatusCode::CREATED, Json(build_simple_client(client, &state.base_url))))
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

    event_service::delete_ticket(&state.event_manager_client, &cod, &state.service_token)
        .await
        .map_err(map_event_service_error)?;

    let client = state
        .client_repo
        .remove_ticket_from_client(&id, &cod)
        .await?;

    Ok(Json(build_simple_client(client, &state.base_url)))
}

#[utoipa::path(
    get,
    path = "/api/client-manager/clients/me",
    responses(
        (status = 200, description = "Current user's client profile", body = Response<Client>),
        (status = 401, description = "Unauthorized - Missing or invalid token"),
        (status = 404, description = "Client not found for this user"),
        (status = 500, description = "Internal server error - Failed to get user email")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "clients"
)]
pub async fn get_my_client(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
) -> Result<Json<Response<Client>>, ClientApiError> {
    let user_email = get_user_email(&state, user_claims.user_id)
        .await
        .ok_or_else(|| {
            ClientApiError::InternalError("Failed to get user email".to_string())
        })?;

    tracing::info!("Searching for client with email: {}", user_email);

    let query = ClientQuery {
        email: Some(user_email.clone()),
        prenume: None,
        nume: None,
    };

    let clients = state.client_repo.list_clients(query).await?;
    tracing::info!("Found {} clients for email {}", clients.len(), user_email);

    let client = clients
        .into_iter()
        .next()
        .ok_or_else(|| {
            tracing::error!("No client found for email: {}", user_email);
            ClientApiError::NotFound("Client not found for this user".to_string())
        })?;

    tracing::info!("Successfully found client for email: {}", user_email);
    Ok(Json(build_simple_client(client, &state.base_url)))
}
