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

    println!("{database_url}");

    let pool = PgPoolOptions::new()
        //.max_connections(5)
        //.idle_timeout(tokio::time::Duration::from_secs(5))
        .connect(database_url.as_str())
        .await?;

    println!("got here");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/user/:uuid", get(user::get_user))
        .route("/users", get(user::get_all_users))
        .route("/user", post(user::post_user))
        .layer(Extension(pool));

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3000)))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
