use std::env;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response}
};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, DecodingKey, decode, Validation};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: String,
}

pub async fn new() -> Response {
    let now = Utc::now();
    let iat = now.timestamp() as usize;

    let expires_at = now + Duration::seconds(3600);
    let exp = expires_at.timestamp() as usize;

    let iss = match env::var("JWT_ISSUER") {
        Ok(issuer) => issuer,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let claims = Claims { sub: "Subject".to_string(), exp, iat, iss };
    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let key = EncodingKey::from_secret(secret.as_bytes());

    match encode(&Header::default(), &claims, &key) {
        Ok(jwt) => (StatusCode::OK, jwt).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn authorize<B>(request: Request<B>, next: Next<B>) -> Response {
    let secret = match env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let key = DecodingKey::from_secret(secret.as_bytes());
    let issuer = match env::var("JWT_ISSUER") {
        Ok(issuer) => issuer,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response()
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.set_issuer(&[issuer]);

    let authorization_header = match request.headers().get("authorization") {
        Some(v) => v,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let authorization = match authorization_header.to_str() {
        Ok(v) => v,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if !authorization.starts_with("Bearer ") {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let jwt = authorization.trim_start_matches("Bearer ");

    let _claims = match decode::<Claims>(&jwt, &key, &Validation::new(jsonwebtoken::Algorithm::HS256)) {
        Ok(token_data) => token_data.claims,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };  

    next.run(request).await
}