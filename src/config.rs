struct Config {
    postgres: Postgres,
}

struct Postgres {
    user: String,
    password: String,
    host: String,
    port: u16,
    database: String,
}
