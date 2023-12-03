use crate::routes::jwt;
use axum::{
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, types::Uuid};
use tower::ServiceBuilder;

#[derive(Serialize, Deserialize)]
pub struct GetToDo {
    pub uuid: Uuid,
    pub owner: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub created_on: DateTime<Local>,
    pub updated_on: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct PostToDo {
    pub title: String,
    pub content: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/todos", get(get_todos).post(post_todo))
        .layer(ServiceBuilder::new().layer(middleware::from_fn(jwt::authorize)))
}

pub async fn get_todos(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<jwt::Claims>,
) -> Response {
    let uuid = match Uuid::parse_str(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid uuid").into_response(),
    };

    match sqlx::query_as!(GetToDo, "SELECT * FROM todos WHERE owner = $1", &uuid)
        .fetch_all(&pool)
        .await
    {
        Ok(todos) => (StatusCode::OK, Json(todos)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, "could not find todos").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get user").into_response(),
    }
}

pub async fn post_todo(
    Extension(pool): Extension<PgPool>,
    Extension(claims): Extension<jwt::Claims>,
    Json(todo): Json<PostToDo>,
) -> Response {
    let now = chrono::offset::Local::now();
    let uuid = match Uuid::parse_str(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid uuid").into_response(),
    };

    match sqlx::query!("INSERT INTO todos (uuid, owner, title, content, created_on, updated_on) VALUES ($1, $2, $3, $4, $5, $6)",
        &Uuid::new_v4(),
        &uuid,
        &todo.title,
        &todo.content,
        &now,
        &now)
        .execute(&pool)
        .await
    {
        Ok(_) => (StatusCode::CREATED, "created todo").into_response(),
        Err(sqlx::Error::Database(error)) => {
            // Unique violation code
            if error.code().unwrap() == "23505" {
                (StatusCode::CONFLICT, "todo already exists").into_response()
            } else {
                (StatusCode::BAD_REQUEST, "could not create todo").into_response()
            }
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create todo").into_response(),
    }
}
