use axum::{extract::Path, Json, http::StatusCode, Extension};
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

pub async fn get_user(
    Path(uuid): Path<Uuid>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<GetUser>, StatusCode> {
    let query = "SELECT * FROM users WHERE uuid = $1";

    let user = match sqlx::query_as::<_, GetUser>(query).bind(&uuid).fetch_one(&pool).await {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(user))
}

pub async fn get_all_users(Extension(pool): Extension<PgPool>) -> Result<Json<Vec<GetUser>>, StatusCode> {
    let query = "SELECT * FROM users";

    let users = match sqlx::query_as::<_, GetUser>(query).fetch_all(&pool).await {
        Ok(users) => users,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(users))    
}

pub async fn post_user(Extension(pool): Extension<PgPool>, Json(user): Json<PostUser>) -> StatusCode {
    let query = "INSERT INTO users (uuid, name, password, created_on, updated_on) VALUES ($1, $2, $3, $4, $5)";

    match sqlx::query(query)
    .bind(&Uuid::new_v4())
    .bind(&user.name)
    .bind(&user.password)
    .bind(&chrono::offset::Local::now())
    .bind(&chrono::offset::Local::now())
    .execute(&pool).await {
        Ok(_) => StatusCode::CREATED,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

