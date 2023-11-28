mod config;
mod jwt;
mod keygen;
mod user;

use axum::{Extension, Router};
use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.postgres.user,
        config.postgres.password,
        config.postgres.host,
        config.postgres.port,
        config.postgres.database,
    );

    println!("connection URL: {database_url}");

    let pool = PgPoolOptions::new().connect(database_url.as_str()).await?;

    println!("successfully connected to the postgres database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let router = Router::new()
        .merge(user::router())
        .merge(jwt::router())
        .layer(
            ServiceBuilder::new()
                .layer(Extension(pool))
                .layer(CompressionLayer::new()),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
