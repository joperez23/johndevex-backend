-- Histórico de tasas de cambio oficiales (BCV) en bolívares venezolanos.
-- La columna `rate` usa NUMERIC (precisión exacta), que en Rust se mapea
-- como `bigdecimal::BigDecimal` gracias al feature "bigdecimal" de sqlx.

CREATE TABLE IF NOT EXISTS finance.exchange_rates (
    id          BIGSERIAL PRIMARY KEY,
    pair        VARCHAR(10)   NOT NULL,                             -- 'USD' | 'EUR'
    price       NUMERIC(20, 8) NOT NULL CHECK (price > 0),
    source      VARCHAR(50)  NOT NULL DEFAULT 'BCV',
    date        DATE NOT NULL DEFAULT now(),
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), -- momento del inserción
    updated_at  TIMESTAMP WITH TIME ZONE                -- momento de actualización
);

-- Consultas típicas: "última tasa de X" y "histórico de X ordenado por fecha"
/*CREATE INDEX IF NOT EXISTS idx_exchange_pair
    ON exchange_rates (pair, created_at DESC);*/

CREATE UNIQUE INDEX IF NOT EXISTS unique_day_pair
    ON finance.exchange_rates (pair, date DESC);

COMMENT ON TABLE finance.exchange_rates IS
    'Histórico de tasas de cambio oficiales (ej. BCV) en bolívares venezolanos';
COMMENT ON COLUMN finance.exchange_rates.price IS
    'Tasa expresada en VES por 1 unidad de la moneda (precisión exacta / BigDecimal)';
