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
use sqlx::PgPool;
use tower::ServiceBuilder;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct GetToDo {
    pub uuid: Uuid,
    pub owner: Uuid,
    pub title: String,
    pub content: String,
    pub created_on: DateTime<Local>,
    pub updated_on: DateTime<Local>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
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
    let query = "SELECT * FROM todos WHERE owner = $1";

    let uuid = match Uuid::parse_str(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid uuid").into_response(),
    };

    match sqlx::query_as::<_, GetToDo>(query)
        .bind(&uuid)
        .fetch_all(&pool)
        .await
    {
        Ok(todo) => (StatusCode::OK, Json(todo)).into_response(),
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
    let query = "INSERT INTO todos (uuid, owner, title, content, created_on, updated_on) VALUES ($1, $2, $3, $4, $5, $6)";

    let now = chrono::offset::Local::now();
    let uuid = match Uuid::parse_str(&claims.sub) {
        Ok(uuid) => uuid,
        Err(_) => return (StatusCode::BAD_REQUEST, "invalid uuid").into_response(),
    };

    match sqlx::query(query)
        .bind(&Uuid::new_v4())
        .bind(&uuid)
        .bind(&todo.title)
        .bind(&todo.content)
        .bind(&now)
        .bind(&now)
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
