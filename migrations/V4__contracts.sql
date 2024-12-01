CREATE TABLE contracts (
    id                     UUID NOT NULL PRIMARY KEY,
    kind                   INT2 NOT NULL CHECK (kind BETWEEN 1 AND 5),
    name                   VARCHAR NOT NULL CHECK (length(name) > 0),
    description            VARCHAR NOT NULL CHECK (length(description) > 0),
    realty_id              UUID REFERENCES realties ON UPDATE RESTRICT
                                                    ON DELETE RESTRICT,
    employer_id            UUID NOT NULL REFERENCES users ON UPDATE RESTRICT
                                                          ON DELETE RESTRICT,
    landlord_id            UUID REFERENCES users ON UPDATE RESTRICT
                                                 ON DELETE RESTRICT,
    purchaser_id           UUID REFERENCES users ON UPDATE RESTRICT
                                                 ON DELETE RESTRICT,
    price                  NUMERIC,
    price_currency         INT2 CHECK (price_currency BETWEEN 1 AND 3),
    deposit                NUMERIC,
    deposit_currency       INT2 CHECK (deposit_currency BETWEEN 1 AND 3),
    one_time_fee           NUMERIC,
    one_time_fee_currency  INT2 CHECK (one_time_fee_currency BETWEEN 1 AND 3),
    monthly_fee            NUMERIC,
    monthly_fee_currency   INT2 CHECK (monthly_fee_currency BETWEEN 1 AND 3),
    percent_fee            NUMERIC,
    is_placed              BOOLEAN,
    created_at             TIMESTAMPTZ NOT NULL,
    expires_at             TIMESTAMPTZ,
    terminated_at          TIMESTAMPTZ
);
COMMENT ON COLUMN contracts.kind
        IS '1 - rent, 2 - sale, 3 - management for rent, 4 - management for sale, 5 - employment';
COMMENT ON COLUMN contracts.price_currency
        IS '1 - USD, 2 - EUR, 3 - RUB';
COMMENT ON COLUMN contracts.deposit_currency
        IS '1 - USD, 2 - EUR, 3 - RUB';
COMMENT ON COLUMN contracts.one_time_fee_currency
        IS '1 - USD, 2 - EUR, 3 - RUB';
COMMENT ON COLUMN contracts.monthly_fee_currency
        IS '1 - USD, 2 - EUR, 3 - RUB';

CREATE TABLE contracts_lock (
    id  UUID NOT NULL PRIMARY KEY
);

INSERT INTO contracts (id, kind, name, description, realty_id, employer_id, landlord_id, purchaser_id, price, price_currency, deposit, deposit_currency, one_time_fee, one_time_fee_currency, monthly_fee, monthly_fee_currency, percent_fee, is_placed, created_at, expires_at, terminated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    5,
    'Administrator contract',
    'Administrator contract',
    NULL,
    '00000000-0000-0000-0000-000000000001',
    NULL,
    NULL,
    0,
    1,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    NULL,
    '2024-01-01 00:00:00',
    NULL,
    NULL
);