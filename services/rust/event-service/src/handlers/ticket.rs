use crate::AppState;
use crate::middleware::{Authorization, UserClaims};
use crate::models::ticket::{Ticket, UpdateTicket};
use crate::utils::error::{ApiError, map_authorization_error};
use crate::utils::links;
use crate::utils::links::{Response, build_ticket_over_event, build_ticket_over_packet};
use axum::extract::rejection::JsonRejection;
use axum::response::IntoResponse;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use std::sync::Arc;
use validator::Validate;

pub fn ticket_manager_router() -> Router<Arc<AppState>> {
    Router::new().route("/tickets", get(list_tickets)).route(
        "/tickets/{cod}",
        get(get_ticket).put(update_ticket).delete(delete_ticket),
    )
}

#[utoipa::path(
    get,
    path = "/api/event-manager/tickets/{cod}",
    params(
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 200, description = "Ticket found", body = Response<Ticket>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_ticket(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let ticket = state.ticket_repo.get_ticket(&cod).await?;

    if !user_claims.is_clients_service() {
        if let Some(event_id) = ticket.id_event {
            let event = state.event_repo.get_event(event_id).await?;
            Authorization::can_access_resource(&user_claims, &event, None)
                .map_err(map_authorization_error)?;
        } else if let Some(packet_id) = ticket.id_pachet {
            let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
            Authorization::can_access_resource(&user_claims, &packet, None)
                .map_err(map_authorization_error)?;
        }
    }

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);

    Ok(Json(ticket_response))
}

#[utoipa::path(
    get,
    path = "/api/event-manager/tickets",
    responses(
        (status = 200, description = "List of all tickets", body = [Response<Ticket>]),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tickets(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
) -> Result<impl IntoResponse, ApiError> {
    if !user_claims.is_admin() && !user_claims.is_clients_service() {
        return Err(ApiError::Forbidden(
            "Only admins can list all tickets".to_string(),
        ));
    }

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event/packet owner or admin can update tickets"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_ticket(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(cod): Path<String>,
    payload: Result<Json<UpdateTicket>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(payload) = payload?;
    payload.validate()?;

    let existing_ticket_opt = state.ticket_repo.get_ticket(&cod).await.ok();

    if let Some(existing_ticket) = existing_ticket_opt {
        if let Some(event_id) = existing_ticket.id_event {
            let event = state.event_repo.get_event(event_id).await?;
            Authorization::can_modify_resource(&user_claims, &event, None)
                .map_err(map_authorization_error)?;
        } else if let Some(packet_id) = existing_ticket.id_pachet {
            let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
            Authorization::can_modify_resource(&user_claims, &packet, None)
                .map_err(map_authorization_error)?;
        }

        let ticket = state.ticket_repo.update_ticket(&cod, payload).await?;
        let ticket_response = links::build_simple_ticket(ticket, &state.base_url);
        return Ok(Json(ticket_response));
    }

    if !user_claims.is_clients_service() {
        if let Some(event_id) = payload.id_event {
            let event = state.event_repo.get_event(event_id).await?;
            Authorization::can_modify_resource(&user_claims, &event, None)
                .map_err(map_authorization_error)?;
        } else if let Some(packet_id) = payload.id_pachet {
            let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
            Authorization::can_modify_resource(&user_claims, &packet, None)
                .map_err(map_authorization_error)?;
        }
    }

    let ticket = if let Some(event_id) = payload.id_event {
        state
            .ticket_repo
            .create_ticket_with_code_for_event(cod, event_id)
            .await?
    } else if let Some(packet_id) = payload.id_pachet {
        state
            .ticket_repo
            .create_ticket_with_code_for_packet(cod, packet_id)
            .await?
    } else {
        return Err(ApiError::BadRequest(
            "Must specify either evenimentid or pachetid".to_string(),
        ));
    };

    let ticket_response = links::build_simple_ticket(ticket, &state.base_url);
    Ok(Json(ticket_response))
}

#[utoipa::path(
    delete,
    path = "/api/event-manager/tickets/{cod}",
    params(
        ("cod" = String, Path, description = "Ticket code")
    ),
    responses(
        (status = 204, description = "Ticket deleted"),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event/packet owner or admin can delete tickets"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_ticket(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(cod): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let existing_ticket = state.ticket_repo.get_ticket(&cod).await?;

    // Check authorization: clients-service role OR admin OR event/packet owner
    if !user_claims.is_clients_service() {
        if let Some(event_id) = existing_ticket.id_event {
            let event = state.event_repo.get_event(event_id).await?;
            Authorization::can_modify_resource(&user_claims, &event, None)
                .map_err(map_authorization_error)?;
        } else if let Some(packet_id) = existing_ticket.id_pachet {
            let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
            Authorization::can_modify_resource(&user_claims, &packet, None)
                .map_err(map_authorization_error)?;
        }
    }

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    // Check authorization: clients-service role OR admin OR event owner
    if !user_claims.is_clients_service() {
        let event = state.event_repo.get_event(event_id).await?;
        Authorization::can_access_resource(&user_claims, &event, None)
            .map_err(map_authorization_error)?;
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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tickets_for_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(event_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    if !user_claims.is_clients_service() {
        let event = state.event_repo.get_event(event_id).await?;
        Authorization::can_access_resource(&user_claims, &event, None)
            .map_err(map_authorization_error)?;
    }

    let tickets = state.ticket_repo.list_tickets_for_event(event_id).await?;

    let wrapped: Vec<Response<Ticket>> = tickets
        .into_iter()
        .map(|t| build_ticket_over_event(t, event_id, &state.base_url))
        .collect();

    Ok(Json(wrapped))
}

#[utoipa::path(
    post,
    path = "/api/event-manager/events/{event_id}/tickets",
    responses(
        (status = 201, description = "Ticket created for event", body = Response<Ticket>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owner, admin, or clients-service can create tickets"),
        (status = 404, description = "Event not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(event_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    if !user_claims.is_clients_service() {
        let event = state.event_repo.get_event(event_id).await?;
        Authorization::can_modify_resource(&user_claims, &event, None)
            .map_err(map_authorization_error)?;
    }

    let ticket = state.ticket_repo.create_ticket_for_event(event_id).await?;

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only event owner or admin can delete tickets"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_ticket_for_event(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((event_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if event_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let event = state.event_repo.get_event(event_id).await?;
    Authorization::can_modify_resource(&user_claims, &event, None)
        .map_err(map_authorization_error)?;

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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_tickets_for_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(packet_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    if !user_claims.is_clients_service() {
        let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
        Authorization::can_access_resource(&user_claims, &packet, None)
            .map_err(map_authorization_error)?;
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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    // Check authorization: clients-service role OR admin OR packet owner
    if !user_claims.is_clients_service() {
        let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
        Authorization::can_access_resource(&user_claims, &packet, None)
            .map_err(map_authorization_error)?;
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
    responses(
        (status = 201, description = "Ticket created for packet", body = Response<Ticket>),
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only packet owner, admin, or clients-service can create tickets"),
        (status = 404, description = "Packet not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path(packet_id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    if !user_claims.is_clients_service() {
        let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
        Authorization::can_modify_resource(&user_claims, &packet, None)
            .map_err(map_authorization_error)?;
    }

    let ticket = state
        .ticket_repo
        .create_ticket_for_packet(packet_id)
        .await?;

    let ticket_response = build_ticket_over_packet(ticket, packet_id, &state.base_url);

    Ok((StatusCode::CREATED, Json(ticket_response)))
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
        (status = 401, description = "Missing or invalid authentication token"),
        (status = 403, description = "Forbidden - Only packet owner or admin can delete tickets"),
        (status = 404, description = "Ticket not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Tickets",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_ticket_for_packet(
    State(state): State<Arc<AppState>>,
    Extension(user_claims): Extension<UserClaims>,
    Path((packet_id, ticket_cod)): Path<(i32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    if packet_id < 0 {
        return Err(ApiError::BadRequest("ID cannot be negative".into()));
    }

    let packet = state.event_packet_repo.get_event_packet(packet_id).await?;
    Authorization::can_modify_resource(&user_claims, &packet, None)
        .map_err(map_authorization_error)?;

    state
        .ticket_repo
        .delete_ticket_for_packet(packet_id, &ticket_cod)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
