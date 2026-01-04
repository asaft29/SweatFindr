use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct Link {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
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

#[derive(Serialize, Debug, ToSchema)]
pub struct HateoasResponse<T>
where
    T: ToSchema + Serialize,
{
    #[serde(flatten)]
    pub data: T,
    #[serde(rename = "_links")]
    pub links: Links,
}

impl<T> HateoasResponse<T>
where
    T: ToSchema + Serialize,
{
    pub fn new(data: T, links: Links) -> Self {
        Self { data, links }
    }
}

pub type Response<T> = HateoasResponse<T>;

pub struct ResponseBuilder<T> {
    data: T,
    self_link: Link,
    parent_link: Option<Link>,
    other_links: HashMap<String, Link>,
}

impl<T> ResponseBuilder<T>
where
    T: ToSchema + Serialize,
{
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

    pub fn build(self) -> HateoasResponse<T> {
        HateoasResponse {
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
}

#[derive(Debug, Deserialize)]
pub struct HateoasRequest<T> {
    #[serde(flatten)]
    pub data: T,
}
