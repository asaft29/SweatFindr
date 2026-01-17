use axum::{body::Body, extract::ConnectInfo, http::Request};
use governor::middleware::NoOpMiddleware;
use std::net::SocketAddr;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::KeyExtractor, GovernorError, GovernorLayer,
};

#[derive(Clone)]
pub struct IpKeyExtractor;

impl KeyExtractor for IpKeyExtractor {
    type Key = String;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if let Some(forwarded) = req.headers().get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded.to_str() {
                if let Some(ip) = forwarded_str.split(',').next() {
                    return Ok(ip.trim().to_string());
                }
            }
        }

        if let Some(real_ip) = req.headers().get("x-real-ip") {
            if let Ok(ip) = real_ip.to_str() {
                return Ok(ip.to_string());
            }
        }

        if let Some(connect_info) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
            return Ok(connect_info.0.ip().to_string());
        }

        Ok("unknown".to_string())
    }
}
/// - 5 requests per 30 seconds for login/register
pub fn create_auth_rate_limit_layer() -> GovernorLayer<IpKeyExtractor, NoOpMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(30)
        .burst_size(5)
        .key_extractor(IpKeyExtractor)
        .finish()
        .expect("Failed to create governor config");

    GovernorLayer::new(config)
}

/// - 3 requests per 30 seconds (to prevent email spam pretty much)
pub fn create_email_rate_limit_layer() -> GovernorLayer<IpKeyExtractor, NoOpMiddleware, Body> {
    let config = GovernorConfigBuilder::default()
        .per_second(30)
        .burst_size(3)
        .key_extractor(IpKeyExtractor)
        .finish()
        .expect("Failed to create governor config");

    GovernorLayer::new(config)
}
