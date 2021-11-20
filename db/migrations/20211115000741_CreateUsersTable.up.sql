-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TYPE user_role AS ENUM ('superuser', 'staff', 'premium', 'basic');

CREATE TABLE address (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    line_1 TEXT NOT NULL,
    line_2 TEXT,
    city TEXT NOT NULL,
    state TEXT NOT NULL,
    country TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    geog GEOGRAPHY(Point),
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON address
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE populist_user (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'basic',
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    confirmed_at timestamptz,
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON populist_user
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE user_profile (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    first_name TEXT,
    middle_name TEXT,
    last_name TEXT,
    preferred_name TEXT,
    address_id uuid,
    user_id uuid NOT NULL,
    CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES populist_user(id),
    CONSTRAINT fk_address FOREIGN KEY(address_id) REFERENCES address(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON user_profile
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();