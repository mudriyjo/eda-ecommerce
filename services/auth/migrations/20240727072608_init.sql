-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS
    "users" (
        id SERIAL NOT NULL PRIMARY KEY,
        uuid UUID NOT NULL DEFAULT (uuid_generate_v4()),
        name VARCHAR(100) NOT NULL,
        phone VARCHAR(12) NOT NULL,
        verified BOOLEAN NOT NULL DEFAULT FALSE,
        code VARCHAR(9) NOT NULL,
        role VARCHAR(50) NOT NULL DEFAULT 'user',
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW()
    );

CREATE INDEX users_phone_idx ON users (phone);