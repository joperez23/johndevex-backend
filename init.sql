CREATE SCHEMA IF NOT EXISTS finance;

CREATE TABLE IF NOT EXISTS finance.exchange_rates (
    id bigserial NOT NULL,
    pair varchar(10) NOT NULL,
    price numeric(20, 8) NOT NULL,
    "source" varchar(50) DEFAULT 'BCV'::character varying NOT NULL,
    "date" date DEFAULT now() NOT NULL,
    created_at timestamptz DEFAULT now() NOT NULL,
    updated_at timestamptz NULL,
    CONSTRAINT exchange_rates_pkey PRIMARY KEY (id),
    CONSTRAINT exchange_rates_price_check CHECK ((price > (0)::numeric))
);

CREATE UNIQUE INDEX IF NOT EXISTS unique_day_pair ON finance.exchange_rates USING btree (pair, date DESC);
