use crate::models::event::Event;
use crate::models::event_packets::EventPackets;
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
    #[serde(rename = "self")]
    pub link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Link>,
    #[serde(flatten)]
    pub others: HashMap<String, Link>,
}

#[derive(Serialize, Debug)]
pub struct EventResponse {
    #[serde(flatten)]
    pub event: Event,
    #[serde(rename = "_links")]
    pub links: Links,
}

#[derive(Serialize, Debug)]
pub struct EventPacketResponse {
    #[serde(flatten)]
    pub event_packet: EventPackets,
    #[serde(rename = "_links")]
    pub links: Links,
}

impl EventResponse {
    pub fn new(event: Event, base_url: &str) -> Self {
        let event_id = event.id;
        let self_href = format!("{}/events/{}", base_url, event_id);

        let mut other_links = HashMap::new();

        other_links.insert(
            "event-packets".to_string(),
            Link {
                href: format!("{}/events/{}/event-packets", base_url, event_id),
                r#type: Some("GET".to_string()),
            },
        );

        other_links.insert(
            "tickets".to_string(),
            Link {
                href: format!("{}/events/{}/tickets", base_url, event_id),
                r#type: Some("GET".to_string()),
            },
        );

        Self {
            event,
            links: Links {
                link: Link {
                    href: self_href,
                    r#type: None,
                },
                parent: Some(Link {
                    href: format!("{}/events", base_url),
                    r#type: None,
                }),
                others: other_links,
            },
        }
    }
}

impl EventPacketResponse {
    pub fn new(event_packet: EventPackets, base_url: &str) -> Self {
        let packet_id = event_packet.id;
        let self_href = format!("{}/event-packets/{}", base_url, packet_id);

        let mut other_links = HashMap::new();

        other_links.insert(
            "events".to_string(),
            Link {
                href: format!("{}/event-packets/{}/events", base_url, packet_id),
                r#type: Some("GET".to_string()),
            },
        );

        other_links.insert(
            "tickets".to_string(),
            Link {
                href: format!("{}/event-packets/{}/tickets", base_url, packet_id),
                r#type: Some("GET".to_string()),
            },
        );

        Self {
            event_packet,
            links: Links {
                link: Link {
                    href: self_href,
                    r#type: None,
                },
                parent: Some(Link {
                    href: format!("{}/event-packets", base_url),
                    r#type: None,
                }),
                others: other_links,
            },
        }
    }
}
