CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    name varchar(24) UNIQUE NOT NULL,
    password varchar(72) NOT NULL,
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL
)