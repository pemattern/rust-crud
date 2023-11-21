mod config;
mod user;

use std::{net::SocketAddr, env};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::postgres::PgPoolOptions;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    let database_url = format!("postgres://{}:{}@{}:{}/{}",
        config.postgres.user,
        config.postgres.password,
        config.postgres.host,
        config.postgres.port,
        config.postgres.database,
    );

    env::set_var("DATABASE_URL", &database_url);

    println!("connection URL: {database_url}");

    let pool = PgPoolOptions::new()
        .connect(database_url.as_str())
        .await?;

    println!("successfully connected to the postgres database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/user/:uuid", get(user::get_user))
        .route("/users", get(user::get_all_users))
        .route("/user", post(user::post_user))
        .layer(Extension(pool));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
