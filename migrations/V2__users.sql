CREATE TABLE users (
    id             UUID NOT NULL PRIMARY KEY,
    name           VARCHAR NOT NULL CHECK (length(name) > 0),
    login          VARCHAR NOT NULL CHECK (length(login) > 0),
    password_hash  VARCHAR NOT NULL CHECK (length(password_hash) > 0),
    email          VARCHAR CHECK (length(email) > 0),
    phone          VARCHAR CHECK (length(phone) > 0),
    created_at     TIMESTAMPTZ NOT NULL,
    deleted_at     TIMESTAMPTZ,
    CHECK (email IS NOT NULL OR phone IS NOT NULL)
);

CREATE UNIQUE INDEX idx_users_login ON users (login)
WHERE deleted_at IS NULL;

CREATE TABLE users_lock (
    id  UUID NOT NULL PRIMARY KEY
);

INSERT INTO users (id, name, login, password_hash, email, phone, created_at, deleted_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Administrator',
    'admin',
    'admin',
    'admin@localhost',
    '+70000000000',
    '2020-01-01 00:00:00',
    null
);

INSERT INTO users (id, name, login, password_hash, email, phone, created_at, deleted_at)
SELECT
    uuid_generate_v4(),
    'User ' || i,
    'user' || i,
    'user' || i,
    'user' || i || '@localhost',
    '+7000000000' || i,
    '2020-01-01 00:00:00',
    null
FROM generate_series(1, 100) AS i;