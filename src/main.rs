mod config;
mod keygen;
mod routes;

use axum::{Extension, Router};
use config::Config;
use sqlx::postgres::PgPoolOptions;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load();

    println!("DATABASE_URL: {}", config.postgres.database_url);

    let pool = PgPoolOptions::new()
        .connect(&config.postgres.database_url.as_str())
        .await?;

    println!("successfully connected to the postgres database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    sqlx::query!(
        "INSERT INTO users (uuid, name, password, created_on, updated_on)
        VALUES ($1, 'admin', 'pass', '2023-01-01', '2023-01-01')
        ON CONFLICT DO NOTHING",
        sqlx::types::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
    )
    .execute(&pool)
    .await?;

    let router = Router::new()
        .merge(routes::recipes::router())
        .merge(routes::users::router())
        .merge(routes::jwt::router())
        .layer(
            ServiceBuilder::new()
                .layer(Extension(pool))
                .layer(Extension(config))
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(5))),
        );

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
