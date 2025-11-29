use crate::handlers::auth::*;
use crate::handlers::client::*;
use crate::models::client::{AddTicket, Client, SocialMedia, TicketRef, UpdateClient};
use crate::services::event_service::{EventInfo, PacketInfo, TicketDetails, TicketInfo};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        register,
        list_clients,
        get_client,
        update_client,
        patch_client,
        delete_client,
        get_client_tickets,
        add_ticket_to_client,
        remove_ticket_from_client,
    ),
    components(schemas(
        Client,
        UpdateClient,
        AddTicket,
        TicketRef,
        SocialMedia,
        TicketDetails,
        TicketInfo,
        EventInfo,
        PacketInfo,
        RegisterRequest,
        RegisterResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "clients", description = "Client management endpoints"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
