use crate::handlers::{event::*, event_packets::*, join_pe::*, ticket::*};
use crate::models::{event::Event, event_packets::EventPackets, ticket::Ticket};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(

        create_event,
        get_event,
        update_event,
        patch_event,
        delete_event,
        list_events,


        create_event_packet,
        get_event_packet,
        update_event_packet,
        patch_event_packet,
        delete_event_packet,
        list_event_packets,


        get_ticket,
        update_ticket,
        delete_ticket,
        list_tickets,
        create_ticket_for_event,
        get_ticket_for_event,
        delete_ticket_for_event,
        list_tickets_for_packet,
        create_ticket_for_packet,
        get_ticket_for_packet,
        delete_ticket_for_packet,


        add_event_to_packet,
        remove_event_from_packet,
        list_events_for_packet,
        list_packets_for_event
    ),
    components(schemas(Event, EventPackets, Ticket)),
    modifiers(&SecurityAddon),
    tags(
        (name = "events", description = "Event management endpoints"),
        (name = "event_packets", description = "Event packet management endpoints"),
        (name = "tickets", description = "Ticket management endpoints"),
        (name = "joins", description = "Link events with packets")
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
