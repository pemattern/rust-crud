mod config;
mod user;
mod jwt;
mod keygen;

use std::net::SocketAddr;
use axum::{
    Extension, Router, routing::get,
};
use sqlx::postgres::PgPoolOptions;
use config::Config;
use tower::ServiceBuilder;

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

    println!("connection URL: {database_url}");

    let pool = PgPoolOptions::new()
        .connect(database_url.as_str())
        .await?;

    println!("successfully connected to the postgres database");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let router = Router::new()
        .merge(user::router())        
        .merge(jwt::router())
        .layer( // TODO: Add the Sevice Builder to JWT. not here
            ServiceBuilder::new()
            .layer(Extension(pool))
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
