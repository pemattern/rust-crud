mod schemas;

use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use schemas::User;
use sqlx::postgres::{PgPool, PgPoolOptions};

async fn create_user(Extension(pool): Extension<PgPool>, Json(user): Json<User>) -> StatusCode {
    let query = format!(
        "INSERT INTO users (email, password) VALUES ('{}', '{}')",
        user.email, user.password
    );

    println!("{}", query);

    match sqlx::query(query.as_str()).execute(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST,
    }
}

async fn get_user_from_id(
    Path(id): Path<i64>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<User>, StatusCode> {
    let query = format!("SELECT * FROM users WHERE id = {}", id);

    let user: User = match sqlx::query_as(query.as_str()).fetch_one(&pool).await {
        Ok(user) => user,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    Ok(Json(user))
}

#[tokio::main]
async fn main() {
    let database_url = "postgres://postgres:blackdahlia1993@localhost:5432/rustcrud";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(tokio::time::Duration::from_secs(5))
        .connect(database_url)
        .await
        .unwrap();

    let app = Router::new()
        .route("/user/:id", get(get_user_from_id))
        .route("/user", post(create_user))
        .layer(Extension(pool));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
