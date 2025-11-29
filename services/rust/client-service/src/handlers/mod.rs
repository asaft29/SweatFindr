pub mod auth;
pub mod client;

use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;
use crate::utils::doc::ApiDoc;

pub fn api_router() -> Router<Arc<AppState>> {
    Router::new().merge(client::client_manager_router())
}

pub fn auth_router() -> Router<Arc<AppState>> {
    Router::new().merge(auth::auth_router())
}

pub fn swagger_router() -> Router<Arc<AppState>> {
    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
}
