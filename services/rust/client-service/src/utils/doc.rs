use crate::handlers::auth::*;
use crate::handlers::client::*;
use crate::models::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, ResendVerificationRequest,
    ResendVerificationResponse, UpdateRoleRequest, UpdateRoleResponse, VerifyEmailRequest,
    VerifyEmailResponse,
};
use crate::models::client::{AddTicket, Client, CreateClient, SocialMedia, TicketRef, UpdateClient};
use crate::services::event_service::{EventInfo, PacketInfo, TicketDetails, TicketInfo};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        register,
        login,
        verify_email,
        resend_verification,
        list_clients,
        create_client,
        get_my_client,
        get_client,
        update_client,
        patch_client,
        delete_client,
        get_client_tickets,
        add_ticket_to_client,
        remove_ticket_from_client,
        update_user_role,
    ),
    components(schemas(
        Client,
        CreateClient,
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
        LoginRequest,
        LoginResponse,
        VerifyEmailRequest,
        VerifyEmailResponse,
        ResendVerificationRequest,
        ResendVerificationResponse,
        UpdateRoleRequest,
        UpdateRoleResponse,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication and authorization endpoints"),
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
