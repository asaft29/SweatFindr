use crate::models::client::{Client, ClientQuery, TicketRef};

pub use common::links::{Link, Links, Response, ResponseBuilder};
use hateoas_macros::{hateoas_simple, hateoas_nested, hateoas_filtered};

#[hateoas_simple(
    resource = "api/client-manager/clients",
    id_field = "id",
    id_to_string = "to_hex()",
    self_methods = "[GET, PUT, PATCH, DELETE]",
    parent_methods = "[GET, POST]",
    links(
        ("tickets", "tickets", "[GET, POST]")
    )
)]
pub fn build_simple_client(client: Client, base_url: &str) -> Response<Client> {}

#[hateoas_filtered(
    resource = "api/client-manager/clients",
    self_methods = "GET",
    parent_methods = "[GET, POST]",
    query_fields(
        ("email", "email"),
        ("prenume", "prenume"),
        ("nume", "nume")
    )
)]
pub fn build_filtered_client(
    clients: Vec<Client>,
    params: &ClientQuery,
    base_url: &str,
) -> Vec<Response<Client>> {}

#[hateoas_nested(
    parent_resource = "api/client-manager/clients",
    parent_id_field = "client_id",
    resource = "tickets",
    id_field = "cod",
    self_methods = "[GET, PUT, DELETE]",
    parent_methods = "[GET, POST]"
)]
pub fn build_ticket_ref(ticket: TicketRef, client_id: &str, base_url: &str) -> Response<TicketRef> {}
