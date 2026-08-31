--
-- PostgreSQL database dump
--

\restrict Cn14TBQLtBFYG8iD9HjtjddAMNTdlEooKOBLceMU99Te76C5g5nvl6aMdzeVuX4

-- Dumped from database version 18.4
-- Dumped by pg_dump version 18.4

-- Started on 2026-08-31 16:07:46 -04

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 6 (class 2615 OID 16511)
-- Name: finance; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA finance;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 220 (class 1259 OID 16512)
-- Name: exchange_rates; Type: TABLE; Schema: finance; Owner: -
--

CREATE TABLE finance.exchange_rates (
    id bigint NOT NULL,
    pair character varying(10) NOT NULL,
    price numeric(20,8) NOT NULL,
    source character varying(50) DEFAULT 'BCV'::character varying NOT NULL,
    date date DEFAULT now() NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    CONSTRAINT exchange_rates_price_check CHECK ((price > (0)::numeric))
);


--
-- TOC entry 3432 (class 0 OID 0)
-- Dependencies: 220
-- Name: TABLE exchange_rates; Type: COMMENT; Schema: finance; Owner: -
--

COMMENT ON TABLE finance.exchange_rates IS 'Histórico de tasas de cambio oficiales (ej. BCV) en bolívares venezolanos';


--
-- TOC entry 3433 (class 0 OID 0)
-- Dependencies: 220
-- Name: COLUMN exchange_rates.price; Type: COMMENT; Schema: finance; Owner: -
--

COMMENT ON COLUMN finance.exchange_rates.price IS 'Tasa expresada en VES por 1 unidad de la moneda (precisión exacta / BigDecimal)';


--
-- TOC entry 221 (class 1259 OID 16525)
-- Name: exchange_rates_id_seq; Type: SEQUENCE; Schema: finance; Owner: -
--

CREATE SEQUENCE finance.exchange_rates_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3434 (class 0 OID 0)
-- Dependencies: 221
-- Name: exchange_rates_id_seq; Type: SEQUENCE OWNED BY; Schema: finance; Owner: -
--

ALTER SEQUENCE finance.exchange_rates_id_seq OWNED BY finance.exchange_rates.id;


--
-- TOC entry 222 (class 1259 OID 16526)
-- Name: m_pair; Type: TABLE; Schema: finance; Owner: -
--

CREATE TABLE finance.m_pair (
    id smallint NOT NULL,
    pair character varying(10) NOT NULL,
    description character varying(100)
);


--
-- TOC entry 223 (class 1259 OID 16531)
-- Name: m_pair_id_seq; Type: SEQUENCE; Schema: finance; Owner: -
--

CREATE SEQUENCE finance.m_pair_id_seq
    AS smallint
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3435 (class 0 OID 0)
-- Dependencies: 223
-- Name: m_pair_id_seq; Type: SEQUENCE OWNED BY; Schema: finance; Owner: -
--

ALTER SEQUENCE finance.m_pair_id_seq OWNED BY finance.m_pair.id;


--
-- TOC entry 224 (class 1259 OID 16532)
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


--
-- TOC entry 3261 (class 2604 OID 16544)
-- Name: exchange_rates id; Type: DEFAULT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.exchange_rates ALTER COLUMN id SET DEFAULT nextval('finance.exchange_rates_id_seq'::regclass);


--
-- TOC entry 3265 (class 2604 OID 16545)
-- Name: m_pair id; Type: DEFAULT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.m_pair ALTER COLUMN id SET DEFAULT nextval('finance.m_pair_id_seq'::regclass);


--
-- TOC entry 3422 (class 0 OID 16512)
-- Dependencies: 220
-- Data for Name: exchange_rates; Type: TABLE DATA; Schema: finance; Owner: -
--

COPY finance.exchange_rates (id, pair, price, source, date, created_at, updated_at) FROM stdin;
74	USDVES	766.86030000	BCV	2026-08-12	2026-08-12 18:02:17.732134-04	\N
75	EURVES	885.07948384	BCV	2026-08-12	2026-08-12 18:02:17.906424-04	\N
76	USDTVES	883.10000000	BINANCE_P2P	2026-08-12	2026-08-12 18:02:18.502837-04	\N
77	USDCOP	3123.28000000	DATOS_GOV_CO	2026-08-12	2026-08-12 18:02:19.128761-04	\N
78	EURCOP	3602.33899000	BANREP	2026-08-12	2026-08-12 18:02:19.827213-04	\N
\.


--
-- TOC entry 3424 (class 0 OID 16526)
-- Dependencies: 222
-- Data for Name: m_pair; Type: TABLE DATA; Schema: finance; Owner: -
--

COPY finance.m_pair (id, pair, description) FROM stdin;
1	USDVES	Valor de dólar en bolívares
2	EURVES	Valor de euro en bolívares
3	USDTVES	Valor de dólar USDT en bolívares
4	USDCOP	Valor de dólar en pesos colombianos
5	EURCOP	Valor de euro en pesos colombianos
\.


--
-- TOC entry 3426 (class 0 OID 16532)
-- Dependencies: 224
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
2026072101	create exchange rates table	2026-07-21 12:27:43.403271-04	t	\\x0669795cee01c1c2bc5356d87c5eb335fd83bc688054642fbb11a4fb9525eb781bc39d2d59744156c1d460dda4ddb825	3366430
\.


--
-- TOC entry 3436 (class 0 OID 0)
-- Dependencies: 221
-- Name: exchange_rates_id_seq; Type: SEQUENCE SET; Schema: finance; Owner: -
--

SELECT pg_catalog.setval('finance.exchange_rates_id_seq', 78, true);


--
-- TOC entry 3437 (class 0 OID 0)
-- Dependencies: 223
-- Name: m_pair_id_seq; Type: SEQUENCE SET; Schema: finance; Owner: -
--

SELECT pg_catalog.setval('finance.m_pair_id_seq', 5, true);


--
-- TOC entry 3269 (class 2606 OID 16547)
-- Name: exchange_rates exchange_rates_pkey; Type: CONSTRAINT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.exchange_rates
    ADD CONSTRAINT exchange_rates_pkey PRIMARY KEY (id);


--
-- TOC entry 3272 (class 2606 OID 16549)
-- Name: m_pair m_pair_pk; Type: CONSTRAINT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.m_pair
    ADD CONSTRAINT m_pair_pk PRIMARY KEY (id);


--
-- TOC entry 3274 (class 2606 OID 16551)
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- TOC entry 3270 (class 1259 OID 16552)
-- Name: unique_day_pair; Type: INDEX; Schema: finance; Owner: -
--

CREATE UNIQUE INDEX unique_day_pair ON finance.exchange_rates USING btree (pair, date DESC);


-- Completed on 2026-08-31 16:07:48 -04

--
-- PostgreSQL database dump complete
--

\unrestrict Cn14TBQLtBFYG8iD9HjtjddAMNTdlEooKOBLceMU99Te76C5g5nvl6aMdzeVuX4

