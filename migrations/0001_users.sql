CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    name text NOT NULL,
    password text NOT NULL,
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL
)