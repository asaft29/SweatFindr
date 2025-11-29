use crate::models::event::{Event, EventQuery};
use crate::models::event_packets::{EventPacketQuery, EventPackets};
use crate::models::ticket::Ticket;

pub use common::links::{Link, Links, Response, ResponseBuilder};

pub fn build_simple_ticket(ticket: Ticket, base_url: &str) -> Response<Ticket> {
    let code = ticket.cod.clone();
    ResponseBuilder::new(ticket, format!("{}/tickets/{}", base_url, code))
        .self_types(&["[GET, PUT, POST, DELETE]"])
        .parent_with_types(format!("{}/tickets", base_url), &["[GET, POST]"])
        .build()
}

pub fn build_simple_event(event: Event, base_url: &str) -> Response<Event> {
    let id = event.id;
    ResponseBuilder::new(event, format!("{}/events/{}", base_url, id))
        .self_types(&["[GET, PUT, POST, DELETE]"])
        .parent_with_types(format!("{}/events", base_url), &["[GET, POST]"])
        .link_with_types(
            "event-packets",
            format!("{}/events/{}/event-packets", base_url, id),
            &["[GET, POST]"],
        )
        .link_with_types(
            "tickets",
            format!("{}/events/{}/tickets", base_url, id),
            &["[GET, POST]"],
        )
        .build()
}

pub fn build_filtered_event(
    events: Vec<Event>,
    params: &EventQuery,
    base_url: &str,
) -> Vec<Response<Event>> {
    let mut responses = Vec::with_capacity(events.len());

    for event in events {
        let mut self_href = format!("{}/events", base_url);
        let mut query_parts = vec![];

        if let Some(loc) = &params.locatie {
            query_parts.push(format!("location={}", loc));
        }
        if let Some(name) = &params.nume {
            query_parts.push(format!("name={}", name));
        }

        if !query_parts.is_empty() {
            self_href = format!("{}?{}", self_href, query_parts.join("&"));
        }

        let response = ResponseBuilder::new(event, self_href)
            .self_types(&["GET"])
            .parent_with_types(format!("{}/events", base_url), &["[GET", "POST]"])
            .build();

        responses.push(response);
    }

    responses
}

pub fn build_packet_over_event(
    event: EventPackets,
    id: i32,
    base_url: &str,
) -> Response<EventPackets> {
    let self_url = format!("{}/events/{}/event-packets", base_url, id);

    ResponseBuilder::new(event, self_url)
        .self_types(&["[GET", "POST]"])
        .parent_with_types(
            format!("{}/events/{}", base_url, id),
            &["[GET, PUT, POST, DELETE]"],
        )
        .build()
}

pub fn build_ticket_over_event(ticket: Ticket, event_id: i32, base_url: &str) -> Response<Ticket> {
    let code = ticket.cod.clone();
    let self_url = format!("{}/events/{}/tickets/{}", base_url, event_id, code);
    let parent_url = format!("{}/events/{}/tickets", base_url, event_id);

    ResponseBuilder::new(ticket, self_url)
        .self_types(&["[GET", "PUT", "POST", "DELETE]"])
        .parent_with_types(parent_url, &["[GET, POST]"])
        .build()
}

pub fn build_simple_event_packet(packet: EventPackets, base_url: &str) -> Response<EventPackets> {
    let packet_id = packet.id;

    ResponseBuilder::new(packet, format!("{}/event-packets/{}", base_url, packet_id))
        .self_types(&["[GET", "PUT", "POST", "DELETE]"])
        .parent_with_types(format!("{}/event-packets", base_url), &["[GET", "POST]"])
        .link_with_types(
            "events",
            format!("{}/event-packets/{}/events", base_url, packet_id),
            &["[GET", "POST]"],
        )
        .link_with_types(
            "tickets",
            format!("{}/event-packets/{}/tickets", base_url, packet_id),
            &["[GET", "POST]"],
        )
        .build()
}

pub fn build_event_over_packet(event: Event, packet_id: i32, base_url: &str) -> Response<Event> {
    let self_url = format!("{}/event-packets/{}/events", base_url, packet_id);

    ResponseBuilder::new(event, self_url)
        .self_types(&["[GET", "POST]"])
        .parent_with_types(
            format!("{}/event-packets/{}", base_url, packet_id),
            &["[GET", "PUT", "POST", "DELETE]"],
        )
        .build()
}

pub fn build_ticket_over_packet(
    ticket: Ticket,
    packet_id: i32,
    base_url: &str,
) -> Response<Ticket> {
    let ticket_cod = ticket.cod.clone();
    let self_url = format!(
        "{}/event-packets/{}/tickets/{}",
        base_url, packet_id, ticket_cod
    );
    let parent_url = format!("{}/event-packets/{}/tickets", base_url, packet_id);

    ResponseBuilder::new(ticket, self_url)
        .self_types(&["[GET", "PUT", "POST", "DELETE]"])
        .parent_with_types(parent_url, &["[GET", "POST]"])
        .build()
}

pub fn build_filtered_event_packets(
    packets: Vec<EventPackets>,
    params: &EventPacketQuery,
    base_url: &str,
) -> Vec<Response<EventPackets>> {
    let mut responses = Vec::with_capacity(packets.len());

    for packet in packets {
        let mut self_href = format!("{}/event-packets", base_url);
        let mut query_parts = vec![];

        if let Some(page) = params.paginare.page {
            query_parts.push(format!("page={}", page));
        }
        if let Some(items) = params.paginare.items_per_page {
            query_parts.push(format!("items_per_page={}", items));
        }
        if let Some(tickets) = params.bilete {
            query_parts.push(format!("available_tickets={}", tickets));
        }
        if let Some(desc) = &params.descriere {
            query_parts.push(format!("type={}", desc));
        }

        if !query_parts.is_empty() {
            self_href = format!("{}?{}", self_href, query_parts.join("&"));
        }

        let response = ResponseBuilder::new(packet, self_href)
            .self_types(&["GET"])
            .parent_with_types(format!("{}/event-packets", base_url), &["[GET", "POST]"])
            .build();

        responses.push(response);
    }

    responses
}
