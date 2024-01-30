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
pub struct GetRecipe {
    pub uuid: Uuid,
    pub owner: Uuid,
    pub title: String,
    pub dose_in_grams: i16,
    pub yield_in_grams: i16,
    pub duration_in_seconds: i16,
    pub roast_level: String,
    pub grind_setting: String,
    pub rating_out_of_ten: i16,
    pub created_on: DateTime<Local>,
    pub updated_on: DateTime<Local>,
}

#[derive(Serialize, Deserialize)]
pub struct PostRecipe {
    pub title: String,
    pub dose_in_grams: i16,
    pub yield_in_grams: i16,
    pub duration_in_seconds: i16,
    pub roast_level: String,
    pub grind_setting: String,
    pub rating_out_of_ten: i16,
}

#[derive(Serialize, Deserialize)]
pub struct PatchRecipe {
    pub uuid: Uuid,
    pub title: Option<String>,
    pub dose_in_grams: Option<i16>,
    pub yield_in_grams: Option<i16>,
    pub duration_in_seconds: Option<i16>,
    pub roast_level: Option<String>,
    pub grind_setting: Option<String>,
    pub rating_out_of_ten: Option<i16>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteRecipe {
    pub uuid: Uuid,
}

pub fn router() -> Router {
    Router::new()
        .route(
            "/recipes",
            get(get_recipes)
                .post(post_recipe)
                .patch(patch_recipe)
                .delete(delete_recipe),
        )
        .layer(ServiceBuilder::new().layer(middleware::from_fn(jwt::authorize)))
}

pub async fn get_recipes(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
) -> Response {
    match sqlx::query_as!(GetRecipe, "SELECT * FROM recipes WHERE owner = $1", &user)
        .fetch_all(&pool)
        .await
    {
        Ok(todos) => (StatusCode::OK, Json(todos)).into_response(),
        Err(sqlx::Error::RowNotFound) => {
            (StatusCode::NOT_FOUND, "could not find recipes").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to get recipes").into_response(),
    }
}

pub async fn post_recipe(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
    Json(recipe): Json<PostRecipe>,
) -> Response {
    let now = chrono::offset::Local::now();

    match sqlx::query!("INSERT INTO recipes (uuid, owner, title, dose_in_grams, yield_in_grams, duration_in_seconds, roast_level, grind_setting, rating_out_of_ten, created_on, updated_on) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        &Uuid::new_v4(),
        &user,
        &recipe.title,
        &recipe.dose_in_grams,
        &recipe.yield_in_grams,
        &recipe.duration_in_seconds,
        &recipe.roast_level,
        &recipe.grind_setting,
        &recipe.rating_out_of_ten,
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

pub async fn patch_recipe(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
    Json(recipe): Json<PatchRecipe>,
) -> Response {
    let now = chrono::offset::Local::now();

    match sqlx::query!(
        "UPDATE recipes SET title = $1, dose_in_grams = $2, yield_in_grams = $3, duration_in_seconds = $4, roast_level = $5, grind_setting = $6, rating_out_of_ten = $7, updated_on = $8 WHERE uuid = $9 AND owner = $10",
        recipe.title,
        recipe.dose_in_grams,
        recipe.yield_in_grams,
        recipe.duration_in_seconds,
        recipe.roast_level,
        recipe.grind_setting,
        recipe.rating_out_of_ten,
        &now,
        &recipe.uuid,
        &user,
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::OK, "updated recipe").into_response(),
        Err(sqlx::Error::Database(_)) => {
            (StatusCode::BAD_REQUEST, "could not update recipe").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to create recipe").into_response(),
    }
}

pub async fn delete_recipe(
    Extension(pool): Extension<PgPool>,
    Extension(user): Extension<Uuid>,
    Json(recipe): Json<DeleteRecipe>,
) -> Response {
    match sqlx::query!(
        "DELETE FROM recipes WHERE uuid = $1 and owner = $2",
        recipe.uuid,
        &user,
    )
    .execute(&pool)
    .await
    {
        Ok(_) => (StatusCode::OK, "deleted recipe").into_response(),
        Err(sqlx::Error::Database(_)) => {
            (StatusCode::BAD_REQUEST, "could not delete recipe").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed to delete recipe").into_response(),
    }
}
