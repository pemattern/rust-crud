mod config;
mod user;

use axum::{
    routing::{get, post},
    Extension, Router,
};

use sqlx::postgres::PgPoolOptions;
use config::Config;


#[tokio::main]
async fn main() {
    let cfg = Config::load();

    let database_url = format!("postgres://{}:{}@{}:{}/{}",
        cfg.postgres.user,
        cfg.postgres.password,
        cfg.postgres.host,
        cfg.postgres.port,
        cfg.postgres.database,
    );

    println!("{database_url}");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(tokio::time::Duration::from_secs(5))
        .connect(database_url.as_str())
        .await
        .unwrap();

    let app = Router::new()
        .route("/user/:uuid", get(user::get_user))
        .route("/users", get(user::get_all_users))
        .route("/user", post(user::post_user))
        .layer(Extension(pool));

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
