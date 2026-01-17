use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{auth::auth_service_client::AuthServiceClient, AppState};

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_auth_middleware(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut client = AuthServiceClient::new(state.auth_channel.clone());

    let validate_request = crate::auth::ValidateRequest {
        token_value: query.token,
    };

    match client.validate_token(validate_request).await {
        Ok(response) => {
            let inner = response.into_inner();
            if inner.success && inner.valid {
                request.extensions_mut().insert(inner.user_id);
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}
