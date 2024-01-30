CREATE TABLE IF NOT EXISTS users (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    name varchar(24) UNIQUE NOT NULL,
    password varchar(72) NOT NULL,
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL
);

CREATE TABLE IF NOT EXISTS recipes (
    uuid uuid PRIMARY KEY UNIQUE NOT NULL,
    owner uuid REFERENCES users (uuid) NOT NULL,
    title varchar(64) NOT NULL,
    dose_in_grams smallint NOT NULL,
    yield_in_grams smallint NOT NULL,
    duration_in_seconds smallint NOT NULL,
    roast_level varchar(64) NOT NULL,
    grind_setting varchar(64) NOT NULL,
    rating_out_of_ten smallint NOT NULL,
    created_on timestamptz NOT NULL,
    updated_on timestamptz NOT NULL    
);
