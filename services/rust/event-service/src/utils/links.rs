use crate::models::event::{Event, EventQuery};
use crate::models::event_packets::{EventPacketQuery, EventPackets};
use crate::models::ticket::Ticket;

pub use common::links::{Link, Links, Response, ResponseBuilder};
use hateoas_macros::{hateoas_simple, hateoas_nested, hateoas_collection, hateoas_filtered};

#[hateoas_simple(
    resource = "tickets",
    id_field = "cod",
    self_methods = "[GET, PUT, POST, DELETE]",
    parent_methods = "[GET, POST]"
)]
pub fn build_simple_ticket(ticket: Ticket, base_url: &str) -> Response<Ticket> {}

#[hateoas_simple(
    resource = "events",
    id_field = "id",
    self_methods = "[GET, PUT, POST, DELETE]",
    parent_methods = "[GET, POST]",
    links(
        ("event-packets", "event-packets", "[GET, POST]"),
        ("tickets", "tickets", "[GET, POST]")
    )
)]
pub fn build_simple_event(event: Event, base_url: &str) -> Response<Event> {}

#[hateoas_filtered(
    resource = "events",
    self_methods = "GET",
    parent_methods = "[GET, POST]",
    query_fields(
        ("locatie", "location"),
        ("nume", "name"),
        ("paginare.page", "page"),
        ("paginare.items_per_page", "items_per_page")
    )
)]
pub fn build_filtered_event(
    events: Vec<Event>,
    params: &EventQuery,
    base_url: &str,
) -> Vec<Response<Event>> {}

#[hateoas_collection(
    parent_resource = "events",
    parent_id_field = "id",
    resource = "event-packets",
    self_methods = "[GET, POST]",
    parent_methods = "[GET, PUT, POST, DELETE]"
)]
pub fn build_packet_over_event(
    event: EventPackets,
    id: i32,
    base_url: &str,
) -> Response<EventPackets> {}

#[hateoas_nested(
    parent_resource = "events",
    parent_id_field = "event_id",
    resource = "tickets",
    id_field = "cod",
    self_methods = "[GET, PUT, POST, DELETE]",
    parent_methods = "[GET, POST]"
)]
pub fn build_ticket_over_event(ticket: Ticket, event_id: i32, base_url: &str) -> Response<Ticket> {}

#[hateoas_simple(
    resource = "event-packets",
    id_field = "id",
    self_methods = "[GET, PUT, POST, DELETE]",
    parent_methods = "[GET, POST]",
    links(
        ("events", "events", "[GET, POST]"),
        ("tickets", "tickets", "[GET, POST]")
    )
)]
pub fn build_simple_event_packet(packet: EventPackets, base_url: &str) -> Response<EventPackets> {}

#[hateoas_collection(
    parent_resource = "event-packets",
    parent_id_field = "packet_id",
    resource = "events",
    self_methods = "[GET, POST]",
    parent_methods = "[GET, PUT, POST, DELETE]"
)]
pub fn build_event_over_packet(event: Event, packet_id: i32, base_url: &str) -> Response<Event> {}

#[hateoas_nested(
    parent_resource = "event-packets",
    parent_id_field = "packet_id",
    resource = "tickets",
    id_field = "cod",
    self_methods = "[GET, PUT, POST, DELETE]",
    parent_methods = "[GET, POST]"
)]
pub fn build_ticket_over_packet(
    ticket: Ticket,
    packet_id: i32,
    base_url: &str,
) -> Response<Ticket> {}

#[hateoas_filtered(
    resource = "event-packets",
    self_methods = "GET",
    parent_methods = "[GET, POST]",
    query_fields(
        ("paginare.page", "page"),
        ("paginare.items_per_page", "items_per_page"),
        ("bilete", "available_tickets"),
        ("descriere", "type")
    )
)]
pub fn build_filtered_event_packets(
    packets: Vec<EventPackets>,
    params: &EventPacketQuery,
    base_url: &str,
) -> Vec<Response<EventPackets>> {}
