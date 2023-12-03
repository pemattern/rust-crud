use dotenv;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub postgres: PostgresConfig,
    pub jwt: JWTConfig,
}
#[derive(Clone)]
pub struct PostgresConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub database_url: String,
}
#[derive(Clone)]
pub struct JWTConfig {
    pub secret: String,
    pub issuer: String,
}

impl Config {
    pub fn load() -> Self {
        let _ = dotenv::dotenv();

        Config {
            postgres: PostgresConfig {
                user: env::var("POSTGRES_USER").unwrap(),
                password: env::var("POSTGRES_PASSWORD").unwrap(),
                host: env::var("POSTGRES_HOST").unwrap(),
                port: env::var("POSTGRES_PORT").unwrap(),
                database: env::var("POSTGRES_DB").unwrap(),
                database_url: env::var("DATABASE_URL").unwrap(),
            },
            jwt: JWTConfig {
                secret: env::var("JWT_SECRET").unwrap(),
                issuer: env::var("JWT_ISSUER").unwrap(),
            },
        }
    }
}
