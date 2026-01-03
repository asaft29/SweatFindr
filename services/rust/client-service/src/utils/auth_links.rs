use crate::models::auth::{
    LoginResponse, RegisterResponse, ResendVerificationResponse, UpdateRoleResponse,
    VerifyEmailResponse,
};

pub use common::links::{Link, Links, Response, ResponseBuilder};
use hateoas_macros::{hateoas_action, hateoas_lookup};

#[hateoas_action(
    resource = "auth/register",
    self_methods = "POST",
    parent_resource = "auth",
    parent_methods = "[POST]",
    links(
        ("login", "auth/login", "POST"),
        ("verify", "auth/verify", "POST")
    )
)]
pub fn build_register_response(
    response: RegisterResponse,
    base_url: &str,
) -> Response<RegisterResponse> {
}

#[hateoas_action(
    resource = "auth/login",
    self_methods = "POST",
    parent_resource = "auth",
    parent_methods = "[POST]",
    links(
        ("register", "auth/register", "POST"),
        ("my-client", "client-manager/clients/me", "GET")
    )
)]
pub fn build_login_response(response: LoginResponse, base_url: &str) -> Response<LoginResponse> {}

#[hateoas_action(
    resource = "auth/verify",
    self_methods = "POST",
    parent_resource = "auth",
    parent_methods = "[POST]",
    links(
        ("login", "auth/login", "POST"),
        ("resend", "auth/resend", "POST")
    )
)]
pub fn build_verify_email_response(
    response: VerifyEmailResponse,
    base_url: &str,
) -> Response<VerifyEmailResponse> {
}

#[hateoas_action(
    resource = "auth/resend",
    self_methods = "POST",
    parent_resource = "auth",
    parent_methods = "[POST]",
    links(
        ("verify", "auth/verify", "POST")
    )
)]
pub fn build_resend_verification_response(
    response: ResendVerificationResponse,
    base_url: &str,
) -> Response<ResendVerificationResponse> {
}

#[hateoas_lookup(
    resource = "auth/role",
    lookup_param = "user_id",
    self_methods = "PUT",
    parent_resource = "auth",
    parent_methods = "[POST]"
)]
pub fn build_update_role_response(
    response: UpdateRoleResponse,
    user_id: i32,
    base_url: &str,
) -> Response<UpdateRoleResponse> {
}
