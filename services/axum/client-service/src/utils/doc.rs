use crate::handlers::client::*;
use crate::models::client::{
    AddTicket, Client, CreateClient, SocialMedia, TicketRef, UpdateClient,
};
use crate::services::event_service::{EventInfo, PacketInfo, TicketDetails, TicketInfo};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_clients,
        get_client,
        create_client,
        update_client,
        patch_client,
        delete_client,
        get_client_tickets,
        add_ticket_to_client,
        remove_ticket_from_client,
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
        PacketInfo
    )),
    tags(
        (name = "clients", description = "Client management endpoints"),
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
