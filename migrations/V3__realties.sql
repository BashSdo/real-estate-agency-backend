CREATE TABLE realties (
    id             UUID NOT NULL PRIMARY KEY,
    hash           UUID NOT NULL,
    address        VARCHAR NOT NULL,
    country        VARCHAR NOT NULL,
    state          VARCHAR,
    city           VARCHAR NOT NULL,
    street         VARCHAR NOT NULL,
    zip_code       VARCHAR,
    building_name  VARCHAR NOT NULL,
    num_floors     INT4 NOT NULL,
    floor          INT4,
    apartment_num  VARCHAR,
    room_num       VARCHAR,
    created_at     TIMESTAMPTZ NOT NULL,
    UNIQUE (hash)
);

CREATE TABLE realties_lock (
    id  UUID NOT NULL PRIMARY KEY
);

CREATE TABLE realties_creation_lock (
    hash  UUID NOT NULL PRIMARY KEY
);


INSERT INTO realties (id, hash,
                      address, country, state, city, street, zip_code, building_name, num_floors, floor,
                      apartment_num, room_num,
                      created_at)
SELECT
    uuid_generate_v4(),
    uuid_generate_v4(),
    'address' || i,
    'country' || i,
    'state' || i,
    'city' || i,
    'street' || i,
    'zip_code' || i,
    'building_name' || i,
    i,
    i,
    'apartment_num' || i,
    'room_num' || i,
    '2020-01-01 00:00:00'
FROM generate_series(1, 100) AS i;