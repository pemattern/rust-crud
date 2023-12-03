CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    name varchar(24) UNIQUE NOT NULL,
    password varchar(72) NOT NULL,
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL
);

CREATE TABLE IF NOT EXISTS todos (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    owner uuid REFERENCES users (uuid) NOT NULL,
    title varchar(64) NOT NULL,
    content varchar(1024),
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL    
);
