use axum::{
    extract::Path,
    Json,
    response::{
        IntoResponse,
        Response
    },
    http::StatusCode,
    Extension,
    Router,
    routing::{
        get,
        post
    }
};
use chrono::{self, DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::postgres::PgPool;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct GetUser {
    pub uuid: Uuid,
    pub name: String,
    pub password: String,
    pub created_on: DateTime<Local>,
    pub updated_on: DateTime<Local>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct PostUser {
    pub name: String,
    pub password: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/user/:uuid", get(get_user))
        .route("/users", get(get_all_users))
        .route("/user", post(post_user))
}

pub async fn get_user(
    Extension(pool): Extension<PgPool>,
    Path(uuid): Path<Uuid>,
) -> Response {
    let query = "SELECT * FROM users WHERE uuid = $1";

    match sqlx::query_as::<_, GetUser>(query).bind(&uuid).fetch_one(&pool).await {
        Ok(user) => (StatusCode::OK, Json(user)).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "could not find user").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get user").into_response(),
    }    
}

pub async fn get_all_users(Extension(pool): Extension<PgPool>) -> Response {
    let query = "SELECT * FROM users";

    match sqlx::query_as::<_, GetUser>(query).fetch_all(&pool).await {
        Ok(users) => (StatusCode::OK, Json(users)).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "no users found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get users").into_response(),
    }   
}

pub async fn post_user(Extension(pool): Extension<PgPool>, Json(user): Json<PostUser>) -> Response {
    let query = "INSERT INTO users (uuid, name, password, created_on, updated_on) VALUES ($1, $2, $3, $4, $5)";

    let now = chrono::offset::Local::now();

    match sqlx::query(query)
    .bind(&Uuid::new_v4())
    .bind(&user.name)
    .bind(&user.password)
    .bind(&now)
    .bind(&now)
    .execute(&pool).await {
        Ok(_) => (StatusCode::CREATED, "created user").into_response(),
        Err(sqlx::Error::Database(error)) => {
            // Unique violation code
            if error.code().unwrap() == "23505" {
                (StatusCode::CONFLICT, "user already exists").into_response()
            } else {
                (StatusCode::BAD_REQUEST, "could not create user").into_response()
            }
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create user").into_response()
    }
}

