use dotenvy;
use std::env;

pub struct Config {
    pub postgres: PostgresConfig,
}

pub struct PostgresConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
}

impl Config {
    pub fn load() -> Self {
        let _ = dotenvy::dotenv();

        Config {
            postgres: PostgresConfig {
                user: env::var("POSTGRES_USER").expect("missing POSTGRES_USER"),
                password: env::var("POSTGRES_PASSWORD").expect("missing POSTGRES_PASSWORD"),
                host: env::var("POSTGRES_HOST").expect("missing POSTGRES_HOST"),
                port: env::var("POSTGRES_PORT").expect("missing POSTGRES_PORT"),
                database: env::var("POSTGRES_DATABASE").expect("missing POSTGRES_DATABASE"),
            },
        }
    }
}
