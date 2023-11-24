use std::env;

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    TypedHeader, headers::{authorization::Basic, Authorization}, Extension,
};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, EncodingKey, Header, DecodingKey, decode, Validation};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

use crate::user::GetUser;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: String,
}

pub async fn new(Extension(pool): Extension<PgPool>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>) -> Response {
    let query = "SELECT * FROM users WHERE name = $1 AND password = $2";
    
    let sub = match sqlx::query_as::<_, GetUser>(query)
        .bind(&authorization.username())
        .bind(&authorization.password())
        .fetch_one(&pool)
        .await {
            Ok(user) => user.uuid.to_string(),
            Err(error) => {
                println!("{}", error);
                return StatusCode::UNAUTHORIZED.into_response();
            },
        };

    let now = Utc::now();
    let iat = now.timestamp() as usize;

    let expires_at = now + Duration::seconds(3600);
    let exp = expires_at.timestamp() as usize;

    let iss = match env::var("JWT_ISSUER") {
        Ok(issuer) => issuer,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let claims = Claims { sub, exp, iat, iss };
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

    let authorization_header = match request.headers().get("Authorization") {
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