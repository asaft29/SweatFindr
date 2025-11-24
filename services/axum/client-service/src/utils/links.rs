pub use common::links::{Link, Links, Response, ResponseBuilder};
use std::collections::HashMap;

pub fn client_links(base_url: &str, client_id: &str) -> Links {
    let mut others = HashMap::new();
    others.insert(
        "tickets".to_string(),
        Link::new(format!(
            "{}/api/client-manager/clients/{}/tickets",
            base_url, client_id
        )),
    );

    Links {
        parent: Some(Link::new(format!(
            "{}/api/client-manager/clients",
            base_url
        ))),
        link: Link::new(format!(
            "{}/api/client-manager/clients/{}",
            base_url, client_id
        )),
        others: Some(others),
    }
}

pub fn client_tickets_links(base_url: &str, client_id: &str) -> Links {
    Links {
        parent: Some(Link::new(format!(
            "{}/api/client-manager/clients/{}",
            base_url, client_id
        ))),
        link: Link::new(format!(
            "{}/api/client-manager/clients/{}/tickets",
            base_url, client_id
        )),
        others: None,
    }
}

pub fn ticket_ref_links(base_url: &str, client_id: &str, ticket_cod: &str) -> Links {
    Links {
        parent: Some(Link::new(format!(
            "{}/api/client-manager/clients/{}/tickets",
            base_url, client_id
        ))),
        link: Link::new(format!(
            "{}/api/client-manager/clients/{}/tickets/{}",
            base_url, client_id, ticket_cod
        )),
        others: None,
    }
}
