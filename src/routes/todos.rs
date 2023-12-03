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
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PatchToDo {
    pub uuid: Uuid,
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteToDo {
    pub uuid: Uuid,
}

pub fn router() -> Router {
    Router::new()
        .route(
            "/todos",
            get(get_todos)
                .post(post_todo)
                .patch(patch_todo)
                .delete(delete_todo),
        )
        .layer(ServiceBuilder::new().layer(middleware::from_fn(jwt::authorize)))
}

pub async fn get_todos(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
) -> Response {
    match sqlx::query_as!(GetToDo, "SELECT * FROM todos WHERE owner = $1", &user)
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
    Extension(user): Extension<Uuid>,
    Json(todo): Json<PostToDo>,
) -> Response {
    let now = chrono::offset::Local::now();

    match sqlx::query!("INSERT INTO todos (uuid, owner, title, content, created_on, updated_on) VALUES ($1, $2, $3, $4, $5, $6)",
        &Uuid::new_v4(),
        &user,
        &todo.title,
        todo.content, // Why no reference possible?
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

pub async fn patch_todo(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
    Json(todo): Json<PatchToDo>,
) -> Response {
    let now = chrono::offset::Local::now();

    match sqlx::query!(
        "UPDATE todos SET title = $1, content = $2, updated_on = $3 WHERE uuid = $4 AND owner = $5",
        todo.title,
        todo.content,
        &now,
        &todo.uuid,
        &user,
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::OK, "updated todo").into_response(),
        Err(sqlx::Error::Database(_)) => {
            (StatusCode::BAD_REQUEST, "could not update todo").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create todo").into_response(),
    }
}

pub async fn delete_todo(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
    Json(todo): Json<DeleteToDo>,
) -> Response {
    match sqlx::query!(
        "DELETE FROM todos WHERE uuid = $1 and owner = $2",
        todo.uuid,
        &user,
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::OK, "deleted todo").into_response(),
        Err(sqlx::Error::Database(_)) => {
            (StatusCode::BAD_REQUEST, "could not delete todo").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete todo").into_response(),
    }
}
