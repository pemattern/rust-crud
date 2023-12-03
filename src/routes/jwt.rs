use crate::{config::Config, routes::users::GetUser};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetJWT {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

pub fn router() -> Router {
    Router::new().route("/token", get(new))
}

pub async fn new(
    Extension(pool): Extension<PgPool>,
    Extension(config): Extension<Config>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
) -> Response {
    let expires_in_seconds = 3600;

    let sub = match sqlx::query_as!(
        GetUser,
        "SELECT * FROM users WHERE name = $1 AND password = $2",
        &authorization.username(),
        &authorization.password()
    )
    .fetch_one(&pool)
    .await
    {
        Ok(user) => user.uuid.to_string(),
        Err(_) => {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    };

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let expires_at = now + Duration::seconds(expires_in_seconds);
    let exp = expires_at.timestamp() as usize;
    let iss = config.jwt.issuer;
    let claims = Claims { sub, exp, iat, iss };

    let secret = config.jwt.secret;
    let key = EncodingKey::from_secret(secret.as_bytes());

    match encode(&Header::default(), &claims, &key) {
        Ok(jwt) => (
            StatusCode::OK,
            Json(GetJWT {
                access_token: jwt,
                token_type: String::from("Bearer"),
                expires_in: expires_in_seconds,
            }),
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn authorize(
    Extension(config): Extension<Config>,
    mut request: Request,
    next: Next,
) -> Response {
    let secret = config.jwt.secret;
    let key = DecodingKey::from_secret(secret.as_bytes());
    let issuer = config.jwt.issuer;
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

    let claims =
        match decode::<Claims>(&jwt, &key, &Validation::new(jsonwebtoken::Algorithm::HS256)) {
            Ok(token_data) => token_data.claims,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        };

    request.extensions_mut().insert(claims);
    next.run(request).await
}
