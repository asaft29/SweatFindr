use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::{Timestamp, Uuid};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: i32,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub role: String,
}

pub struct JwtService {
    secret: String,
    issuer: String,
}

impl JwtService {
    pub fn new(secret: String, issuer: String) -> Self {
        JwtService { secret, issuer }
    }

    pub fn generate_token(
        &self,
        user_id: i32,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("Valid Timestamp")
            .timestamp();

        let now = Utc::now();
        let ts = Timestamp::from_unix(
            uuid::NoContext,
            now.timestamp() as u64,
            now.timestamp_subsec_nanos(),
        );

        let claims = Claims {
            iss: self.issuer.clone(),
            sub: user_id,
            exp: expiration,
            iat: now.timestamp(),
            jti: Uuid::new_v7(ts).to_string(),
            role: role.to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }
}
