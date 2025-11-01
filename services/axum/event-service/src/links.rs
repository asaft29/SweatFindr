use crate::models::event::{Event, EventQuery};
use crate::models::event_packets::{EventPacketQuery, EventPackets};
use crate::models::ticket::Ticket;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Links {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Link>,
    #[serde(rename = "self")]
    pub link: Link,
    #[serde(flatten)]
    pub others: Option<HashMap<String, Link>>,
}

impl Link {
    pub fn new(href: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            r#type: None,
        }
    }

    pub fn with_type(mut self, r#type: impl Into<String>) -> Self {
        self.r#type = Some(r#type.into());
        self
    }
    pub fn with_types(mut self, types: &[&str]) -> Self {
        self.r#type = Some(types.join(", "));
        self
    }
}

#[derive(Serialize, Debug)]
pub struct Response<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(rename = "_links")]
    pub links: Links,
}

pub struct ResponseBuilder<T> {
    data: T,
    self_link: Link,
    parent_link: Option<Link>,
    other_links: HashMap<String, Link>,
}

impl<T> ResponseBuilder<T> {
    pub fn new(data: T, self_href: impl Into<String>) -> Self {
        Self {
            data,
            self_link: Link::new(self_href),
            parent_link: None,
            other_links: HashMap::new(),
        }
    }

    pub fn self_type(mut self, r#type: impl Into<String>) -> Self {
        self.self_link.r#type = Some(r#type.into());
        self
    }

    pub fn self_types(mut self, types: &[&str]) -> Self {
        self.self_link.r#type = Some(types.join(", "));
        self
    }

    pub fn parent(mut self, href: impl Into<String>) -> Self {
        self.parent_link = Some(Link::new(href));
        self
    }

    pub fn parent_with_type(mut self, href: impl Into<String>, r#type: impl Into<String>) -> Self {
        self.parent_link = Some(Link::new(href).with_type(r#type));
        self
    }

    pub fn parent_with_types(mut self, href: impl Into<String>, types: &[&str]) -> Self {
        self.parent_link = Some(Link::new(href).with_types(types));
        self
    }

    pub fn link(mut self, name: impl Into<String>, href: impl Into<String>) -> Self {
        self.other_links.insert(name.into(), Link::new(href));
        self
    }

    pub fn link_with_type(
        mut self,
        name: impl Into<String>,
        href: impl Into<String>,
        r#type: impl Into<String>,
    ) -> Self {
        self.other_links
            .insert(name.into(), Link::new(href).with_type(r#type));
        self
    }

    pub fn link_with_types(
        mut self,
        name: impl Into<String>,
        href: impl Into<String>,
        types: &[&str],
    ) -> Self {
        self.other_links
            .insert(name.into(), Link::new(href).with_types(types));
        self
    }

    pub fn build(self) -> Response<T> {
        Response {
            data: self.data,
            links: Links {
                link: self.self_link,
                parent: self.parent_link,
                others: if self.other_links.is_empty() {
                    None
                } else {
                    Some(self.other_links)
                },
            },
        }
    }
    pub fn builder(entity: T, self_href: impl Into<String>) -> ResponseBuilder<T> {
        ResponseBuilder::new(entity, self_href)
    }
}

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
