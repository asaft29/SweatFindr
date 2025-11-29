use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct UserClaims {
    pub user_id: i32,
    pub role: String,
}

#[derive(Debug)]
pub enum AuthorizationError {
    Forbidden(String),
    Unauthorized(String),
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorizationError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AuthorizationError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl std::error::Error for AuthorizationError {}

impl IntoResponse for AuthorizationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthorizationError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AuthorizationError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    List,
    Create,
    Update,
    Delete,
    Manage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Client,
    OwnerEvent,
}

impl Role {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "client" => Some(Role::Client),
            "owner-event" => Some(Role::OwnerEvent),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::Client => "client",
            Role::OwnerEvent => "owner-event",
        }
    }
}

impl UserClaims {
    pub fn role_enum(&self) -> Option<Role> {
        Role::from_string(&self.role)
    }

    pub fn has_role(&self, role: Role) -> bool {
        self.role_enum() == Some(role)
    }

    pub fn is_admin(&self) -> bool {
        self.has_role(Role::Admin)
    }

    pub fn is_client(&self) -> bool {
        self.has_role(Role::Client)
    }

    pub fn is_owner_event(&self) -> bool {
        self.has_role(Role::OwnerEvent)
    }
}

pub trait ResourceOwnership {
    fn is_owned_by(&self, claims: &UserClaims, user_email: Option<&str>) -> bool;

    fn resource_description(&self) -> String {
        "resource".to_string()
    }
}

pub struct AuthorizationPolicy;

impl AuthorizationPolicy {
    pub fn admin_only(claims: &UserClaims) -> Result<(), AuthorizationError> {
        if claims.is_admin() {
            Ok(())
        } else {
            Err(AuthorizationError::Forbidden(
                "Admin access required".to_string(),
            ))
        }
    }

    pub fn admin_or_role(
        claims: &UserClaims,
        allowed_role: Role,
    ) -> Result<(), AuthorizationError> {
        if claims.is_admin() || claims.has_role(allowed_role) {
            Ok(())
        } else {
            Err(AuthorizationError::Forbidden(format!(
                "Admin or {} access required",
                allowed_role.as_str()
            )))
        }
    }

    pub fn admin_or_owner<T: ResourceOwnership>(
        claims: &UserClaims,
        resource: &T,
        user_email: Option<&str>,
    ) -> Result<(), AuthorizationError> {
        if claims.is_admin() {
            return Ok(());
        }

        if resource.is_owned_by(claims, user_email) {
            return Ok(());
        }

        Err(AuthorizationError::Forbidden(format!(
            "You can only access your own {}",
            resource.resource_description()
        )))
    }

    pub fn check_permission<T: ResourceOwnership>(
        claims: &UserClaims,
        permission: Permission,
        resource: Option<&T>,
        user_email: Option<&str>,
    ) -> Result<(), AuthorizationError> {
        match permission {
            Permission::List | Permission::Create => Self::admin_only(claims),

            Permission::Read | Permission::Update | Permission::Delete | Permission::Manage => {
                match resource {
                    Some(res) => Self::admin_or_owner(claims, res, user_email),
                    _ => Err(AuthorizationError::Forbidden(
                        "Resource not found for authorization check".to_string(),
                    )),
                }
            }
        }
    }

    pub fn check_own_resource_permission(
        claims: &UserClaims,
        permission: Permission,
        allowed_role: Role,
    ) -> Result<(), AuthorizationError> {
        match permission {
            Permission::List | Permission::Create => Self::admin_or_role(claims, allowed_role),
            _ => Err(AuthorizationError::Forbidden(
                "This permission requires a specific resource".to_string(),
            )),
        }
    }
}

pub struct Authorization;

impl Authorization {
    pub fn require_admin(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_only(claims)
    }

    pub fn require_client_or_admin(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_role(claims, Role::Client)
    }

    pub fn require_owner_event_or_admin(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_role(claims, Role::OwnerEvent)
    }

    pub fn can_access_resource<T: ResourceOwnership>(
        claims: &UserClaims,
        resource: &T,
        user_email: Option<&str>,
    ) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_owner(claims, resource, user_email)
    }

    pub fn can_modify_resource<T: ResourceOwnership>(
        claims: &UserClaims,
        resource: &T,
        user_email: Option<&str>,
    ) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_owner(claims, resource, user_email)
    }

    pub fn can_delete_resource(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_only(claims)
    }

    pub fn can_list_all(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_only(claims)
    }

    pub fn can_create(claims: &UserClaims) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_only(claims)
    }

    pub fn can_create_own(
        claims: &UserClaims,
        allowed_role: Role,
    ) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_role(claims, allowed_role)
    }

    pub fn can_list_own(claims: &UserClaims, allowed_role: Role) -> Result<(), AuthorizationError> {
        AuthorizationPolicy::admin_or_role(claims, allowed_role)
    }
}
