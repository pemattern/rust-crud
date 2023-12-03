use crate::routes::jwt;
use axum::{
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use chrono::{self, DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, types::Uuid};
use tower::ServiceBuilder;

#[derive(Serialize, Deserialize)]
pub struct GetUser {
    pub uuid: Uuid,
    pub name: String,
    pub password: String,
    pub created_on: DateTime<Local>,
    pub updated_on: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct PostUser {
    pub name: String,
    pub password: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/user", get(get_user).post(post_user))
        .route("/users", get(get_all_users))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(jwt::authorize)))
}

pub async fn get_user(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<jwt::Claims>,
) -> Response {
    let uuid = match Uuid::parse_str(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid uuid").into_response(),
    };

    match sqlx::query_as!(GetUser, "SELECT * FROM users WHERE uuid = $1", &uuid)
        .fetch_one(&pool)
        .await
    {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, "could not find user").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get user").into_response(),
    }
}

pub async fn get_all_users(Extension(pool): Extension<PgPool>) -> Response {
    match sqlx::query_as!(GetUser, "SELECT * FROM users")
        .fetch_all(&pool)
        .await
    {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "no users found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get users").into_response(),
    }
}

pub async fn post_user(Extension(pool): Extension<PgPool>, Json(user): Json<PostUser>) -> Response {
    let now = chrono::offset::Local::now();

    match sqlx::query!("INSERT INTO users (uuid, name, password, created_on, updated_on) VALUES ($1, $2, $3, $4, $5)", 
        &Uuid::new_v4(), 
        &user.name, 
        &user.password, 
        &now, 
        &now)
        .execute(&pool)
        .await
    {
        Ok(_) => (StatusCode::CREATED, "created user").into_response(),
        Err(sqlx::Error::Database(error)) => {
            // Unique violation code
            if error.code().unwrap() == "23505" {
                (StatusCode::CONFLICT, "user already exists").into_response()
            } else {
                (StatusCode::BAD_REQUEST, "could not create user").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create user").into_response(),
    }
}
