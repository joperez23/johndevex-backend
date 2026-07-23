--
-- PostgreSQL database dump
--

\restrict F8bBMIdZ5KIzVO5VNAK3GNkqrFcv3fil8HYDta426FaQreYkhhjaVUfkfEtkf1l

-- Dumped from database version 18.3
-- Dumped by pg_dump version 18.3

-- Started on 2026-07-23 18:45:20 -04

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
-- TOC entry 6 (class 2615 OID 42225)
-- Name: finance; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA finance;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 222 (class 1259 OID 42582)
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
-- TOC entry 4507 (class 0 OID 0)
-- Dependencies: 222
-- Name: TABLE exchange_rates; Type: COMMENT; Schema: finance; Owner: -
--

COMMENT ON TABLE finance.exchange_rates IS 'Histórico de tasas de cambio oficiales (ej. BCV) en bolívares venezolanos';


--
-- TOC entry 4508 (class 0 OID 0)
-- Dependencies: 222
-- Name: COLUMN exchange_rates.price; Type: COMMENT; Schema: finance; Owner: -
--

COMMENT ON COLUMN finance.exchange_rates.price IS 'Tasa expresada en VES por 1 unidad de la moneda (precisión exacta / BigDecimal)';


--
-- TOC entry 221 (class 1259 OID 42581)
-- Name: exchange_rates_id_seq; Type: SEQUENCE; Schema: finance; Owner: -
--

CREATE SEQUENCE finance.exchange_rates_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 4509 (class 0 OID 0)
-- Dependencies: 221
-- Name: exchange_rates_id_seq; Type: SEQUENCE OWNED BY; Schema: finance; Owner: -
--

ALTER SEQUENCE finance.exchange_rates_id_seq OWNED BY finance.exchange_rates.id;


--
-- TOC entry 224 (class 1259 OID 42601)
-- Name: m_pair; Type: TABLE; Schema: finance; Owner: -
--

CREATE TABLE finance.m_pair (
    id smallint NOT NULL,
    pair character varying(10) NOT NULL,
    description character varying(100)
);


--
-- TOC entry 223 (class 1259 OID 42600)
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
-- TOC entry 4510 (class 0 OID 0)
-- Dependencies: 223
-- Name: m_pair_id_seq; Type: SEQUENCE OWNED BY; Schema: finance; Owner: -
--

ALTER SEQUENCE finance.m_pair_id_seq OWNED BY finance.m_pair.id;


--
-- TOC entry 220 (class 1259 OID 42567)
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
-- TOC entry 4337 (class 2604 OID 42585)
-- Name: exchange_rates id; Type: DEFAULT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.exchange_rates ALTER COLUMN id SET DEFAULT nextval('finance.exchange_rates_id_seq'::regclass);


--
-- TOC entry 4341 (class 2604 OID 42604)
-- Name: m_pair id; Type: DEFAULT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.m_pair ALTER COLUMN id SET DEFAULT nextval('finance.m_pair_id_seq'::regclass);


--
-- TOC entry 4499 (class 0 OID 42582)
-- Dependencies: 222
-- Data for Name: exchange_rates; Type: TABLE DATA; Schema: finance; Owner: -
--

COPY finance.exchange_rates (id, pair, price, source, date, created_at, updated_at) FROM stdin;
3	USDVES	737.23210000	BCV	2026-07-22	2026-07-22 10:39:07.985338-04	2026-07-22 16:09:27.695328-04
4	EURVES	840.85744397	BCV	2026-07-22	2026-07-22 10:39:07.985338-04	2026-07-22 16:09:27.759243-04
72	USDVES	742.22920000	BCV	2026-07-23	2026-07-23 18:25:06.593929-04	\N
73	EURVES	844.21891437	BCV	2026-07-23	2026-07-23 18:25:06.593929-04	\N
1	USDVES	737.23210000	BCV	2026-07-21	2026-07-21 12:27:43.739298-04	2026-07-21 18:20:26.958248-04
2	EURVES	840.85744397	BCV	2026-07-21	2026-07-21 12:29:01.672266-04	2026-07-21 18:20:27.015833-04
\.


--
-- TOC entry 4501 (class 0 OID 42601)
-- Dependencies: 224
-- Data for Name: m_pair; Type: TABLE DATA; Schema: finance; Owner: -
--

COPY finance.m_pair (id, pair, description) FROM stdin;
1	USDVES	Valor de dólar en bolívares
2	EURVES	Valor de euro en bolívares
3	USDCOP	Valor de dólar en pesos colombianos
4	USDTVES	Valor de dólar USDT en bolívares
\.


--
-- TOC entry 4497 (class 0 OID 42567)
-- Dependencies: 220
-- Data for Name: _sqlx_migrations; Type: TABLE DATA; Schema: public; Owner: -
--

COPY public._sqlx_migrations (version, description, installed_on, success, checksum, execution_time) FROM stdin;
2026072101	create exchange rates table	2026-07-21 12:27:43.403271-04	t	\\x0669795cee01c1c2bc5356d87c5eb335fd83bc688054642fbb11a4fb9525eb781bc39d2d59744156c1d460dda4ddb825	3366430
\.


--
-- TOC entry 4511 (class 0 OID 0)
-- Dependencies: 221
-- Name: exchange_rates_id_seq; Type: SEQUENCE SET; Schema: finance; Owner: -
--

SELECT pg_catalog.setval('finance.exchange_rates_id_seq', 73, true);


--
-- TOC entry 4512 (class 0 OID 0)
-- Dependencies: 223
-- Name: m_pair_id_seq; Type: SEQUENCE SET; Schema: finance; Owner: -
--

SELECT pg_catalog.setval('finance.m_pair_id_seq', 4, true);


--
-- TOC entry 4346 (class 2606 OID 42597)
-- Name: exchange_rates exchange_rates_pkey; Type: CONSTRAINT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.exchange_rates
    ADD CONSTRAINT exchange_rates_pkey PRIMARY KEY (id);


--
-- TOC entry 4349 (class 2606 OID 42608)
-- Name: m_pair m_pair_pk; Type: CONSTRAINT; Schema: finance; Owner: -
--

ALTER TABLE ONLY finance.m_pair
    ADD CONSTRAINT m_pair_pk PRIMARY KEY (id);


--
-- TOC entry 4344 (class 2606 OID 42580)
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- TOC entry 4347 (class 1259 OID 42598)
-- Name: unique_day_pair; Type: INDEX; Schema: finance; Owner: -
--

CREATE UNIQUE INDEX unique_day_pair ON finance.exchange_rates USING btree (pair, date DESC);


-- Completed on 2026-07-23 18:45:22 -04

--
-- PostgreSQL database dump complete
--

\unrestrict F8bBMIdZ5KIzVO5VNAK3GNkqrFcv3fil8HYDta426FaQreYkhhjaVUfkfEtkf1l

