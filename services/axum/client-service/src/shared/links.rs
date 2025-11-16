use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Response<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(rename = "_links")]
    pub links: Links,
}

#[derive(Serialize)]
pub struct Links {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Link>,
    #[serde(rename = "self")]
    pub link: Link,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub others: Option<HashMap<String, Link>>,
}

#[derive(Serialize)]
pub struct Link {
    pub href: String,
}

impl Link {
    pub fn new(href: impl Into<String>) -> Self {
        Self { href: href.into() }
    }
}

pub struct LinksBuilder {
    self_link: Link,
    parent: Option<Link>,
    others: Option<HashMap<String, Link>>,
}

impl LinksBuilder {
    pub fn new(self_href: impl Into<String>) -> Self {
        Self {
            self_link: Link::new(self_href),
            parent: None,
            others: None,
        }
    }

    pub fn parent(mut self, href: impl Into<String>) -> Self {
        self.parent = Some(Link::new(href));
        self
    }

    pub fn add_other(mut self, rel: impl Into<String>, href: impl Into<String>) -> Self {
        self.others
            .get_or_insert_with(HashMap::new)
            .insert(rel.into(), Link::new(href));
        self
    }

    pub fn build(self) -> Links {
        Links {
            parent: self.parent,
            link: self.self_link,
            others: self.others,
        }
    }
}

impl<T> Response<T> {
    pub fn new(data: T, links: Links) -> Self {
        Self { data, links }
    }
}

pub fn client_links(base_url: &str, client_id: &str) -> Links {
    LinksBuilder::new(format!("{}/api/client-manager/clients/{}", base_url, client_id))
        .parent(format!("{}/api/client-manager/clients", base_url))
        .add_other(
            "tickets",
            format!("{}/api/client-manager/clients/{}/tickets", base_url, client_id),
        )
        .build()
}

pub fn client_tickets_links(base_url: &str, client_id: &str) -> Links {
    LinksBuilder::new(format!(
        "{}/api/client-manager/clients/{}/tickets",
        base_url, client_id
    ))
    .parent(format!("{}/api/client-manager/clients/{}", base_url, client_id))
    .build()
}

pub fn ticket_ref_links(base_url: &str, client_id: &str, ticket_cod: &str) -> Links {
    LinksBuilder::new(format!(
        "{}/api/client-manager/clients/{}/tickets/{}",
        base_url, client_id, ticket_cod
    ))
    .parent(format!(
        "{}/api/client-manager/clients/{}/tickets",
        base_url, client_id
    ))
    .build()
}
