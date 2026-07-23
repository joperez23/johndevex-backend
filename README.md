# bcv-rates-api

Backend en **[ntex](https://ntex.rs) 3.10.1** + **PostgreSQL**, modularizado y
listo para producción. Incluye un primer endpoint funcional que hace
_web scraping_ de la tasa oficial de cambio del [BCV](https://www.bcv.org.ve/glosario/cambio-oficial)
(USD y EUR en bolívares) y la persiste en base de datos usando `BigDecimal`
para no perder precisión.

## Stack

| Pieza              | Crate                                      |
| ------------------ | ------------------------------------------ |
| Framework web      | `ntex` 3.10.1 (runtime `tokio`)            |
| CORS               | `ntex-cors`                                |
| Base de datos      | `sqlx` (PostgreSQL, `rustls`, sin OpenSSL) |
| Precisión numérica | `bigdecimal`                               |
| HTTP client        | `reqwest` (rustls)                         |
| Parseo HTML        | `scraper`                                  |
| Errores            | `thiserror`                                |
| Config             | variables de entorno + `dotenvy`           |

## Estructura del proyecto

```
src/
├── main.rs                 # arranque: config, DB, servidor, scraping inicial
├── config/                 # lectura y validación de variables de entorno
├── db/                      # pool de conexiones + migraciones embebidas
├── error/                   # AppError único (implementa WebResponseError)
├── models/                  # entidades de dominio (ExchangeRate, Pair)
├── repositories/             # acceso a datos (SQL vía sqlx)
├── services/
│   ├── bcv_scraper.rs        # descarga + parseo del HTML del BCV
│   ├── exchange_rate_service.rs # orquesta scraper + repositorio
│   └── scheduler.rs          # scraping periódico opcional en background
├── handlers/                 # controladores HTTP (ntex `#[web::get/post]`)
├── routes/                   # composición de rutas (App::configure)
└── state.rs                  # estado compartido inyectado con State<T>

migrations/                   # SQL embebido en el binario (sqlx::migrate!)
```

Cada capa tiene una responsabilidad única: `handlers` solo traduce
HTTP ⇄ tipos de Rust, `services` contiene la lógica de negocio,
`repositories` es la única capa que conoce SQL, y `models` son los datos
compartidos entre capas.

## Variables de entorno

Copia `.env.example` a `.env` y ajusta lo necesario. La única variable
**obligatoria** es `DATABASE_URL`; todas las demás tienen un valor por
defecto razonable. Ver `.env.example` para la lista completa y comentada
(servidor, base de datos, scraper del BCV, CORS, logging).

## Cómo correrlo

### Opción A: Docker Compose (recomendado)

```bash
docker compose up --build
```

Esto levanta PostgreSQL + la API. La API queda en `http://localhost:8080` y
ejecuta las migraciones automáticamente al arrancar.

### Opción B: local con Rust instalado

```bash
# 1. Levanta un PostgreSQL (o usa uno existente)
docker run -d --name bcv-postgres -e POSTGRES_USER=bcv -e POSTGRES_PASSWORD=bcv \
  -e POSTGRES_DB=bcv_rates -p 5432:5432 postgres:16-alpine

# 2. Configura el entorno
cp .env.example .env

# 3. Compila y ejecuta (las migraciones corren solas al arrancar)
cargo run
```

> Requiere Rust **1.88** o superior (`rustup update`).

## Endpoints

| Método | Ruta                                    | Descripción                                   |
| ------ | --------------------------------------- | --------------------------------------------- |
| GET    | `/health`                               | Health-check (verifica conexión a la DB)      |
| POST   | `/api/v1/exchange-rates/scrape`         | Scrapea el BCV ahora mismo y guarda USD + EUR |
| GET    | `/api/v1/exchange-rates/latest`         | Última tasa guardada de USD y EUR             |
| GET    | `/api/v1/exchange-rates/{pair}/latest`  | Última tasa de una moneda (`USD` o `EUR`)     |
| GET    | `/api/v1/exchange-rates/{pair}/history` | Histórico, más reciente primero (`?limit=30`) |

Ejemplo:

```bash
curl -X POST http://localhost:8080/api/v1/exchange-rates/scrape

curl http://localhost:8080/api/v1/exchange-rates/latest
# {
#   "usd": { "id": 1, "pair": "USD", "rate": 168.66830000, "source": "BCV", ... },
#   "eur": { "id": 2, "pair": "EUR", "rate": 182.14330000, "source": "BCV", ... }
# }
```

El campo `rate` viaja como número (no como string) pero se calcula y
almacena internamente con **precisión decimal exacta** (`BigDecimal` en
Rust, `NUMERIC(20, 8)` en PostgreSQL) — nunca como `f32`/`f64`.

## Scraping automático (opcional)

Si defines `BCV_SCRAPE_INTERVAL_SECS` (ej. `3600` para cada hora), la API
lanza una tarea en background que repite el scraping periódicamente además
del que se ejecuta una vez al arrancar. Si no la defines, el scraping solo
ocurre cuando llamas explícitamente a `POST /api/v1/exchange-rates/scrape`.

## Notas importantes sobre el scraping del BCV

- El BCV no ofrece una API oficial: se obtiene el HTML público de
  `https://www.bcv.org.ve/glosario/cambio-oficial` y se extraen los valores
  de los contenedores `div#dolar` y `div#euro` (patrón estable usado por la
  comunidad para este sitio). Si el BCV cambia su marcado HTML, el scraper
  devolverá un error 502 (`scraping_error`) claro en vez de fallar en
  silencio; en ese caso solo hay que ajustar los selectores en
  `src/services/bcv_scraper.rs`.
- El certificado TLS del sitio del BCV ha tenido problemas de validación de
  forma recurrente. Por eso, por defecto (`BCV_INSECURE_TLS=true`), el
  cliente HTTP omite la verificación estricta del certificado **únicamente**
  para esa URL fija y conocida — no afecta a ninguna otra conexión de la
  aplicación. Si en tu entorno el certificado es válido, puedes poner esa
  variable en `false`.
- Sé razonable con la frecuencia de scraping (el BCV solo actualiza la tasa
  una vez al día hábil): no hay necesidad de golpear el sitio cada pocos
  segundos.

## Manejo de errores

Todos los errores de la aplicación pasan por un único tipo `AppError`
(`src/error/mod.rs`) que implementa `ntex::web::error::WebResponseError`
(el patrón recomendado en la [documentación de ntex](https://ntex.rs/docs/errors)).
Los handlers simplemente devuelven `Result<HttpResponse, AppError>` y ntex
se encarga de traducirlo a una respuesta JSON consistente:

```json
{ "error": "not_found", "message": "no hay tasas registradas para USD..." }
```

## Tests

```bash
cargo test
```

Incluye tests unitarios de la lógica de parseo del BCV (formato numérico
venezolano y extracción desde HTML de ejemplo) en
`src/services/bcv_scraper.rs`, siguiendo el patrón de testing descrito en
la [documentación de ntex](https://ntex.rs/docs/testing).

## Producción

- Imagen Docker multi-stage sin OpenSSL (todo vía `rustls`), corre como
  usuario sin privilegios.
- Migraciones embebidas en el binario (`sqlx::migrate!`) y ejecutadas
  automáticamente al arrancar.
- Logging estructurado vía `RUST_LOG` (usa `middleware::Logger` de ntex).
- Compresión de respuestas (`middleware::Compress`).
- CORS configurable por variable de entorno.
- Health-check (`/health`) listo para usar en Docker/Kubernetes.
- Para servir HTTPS directamente (sin proxy inverso), se puede habilitar el
  feature `rustls` u `openssl` de `ntex` y usar `HttpServer::bind_rustls` /
  `bind_openssl` — no incluido aquí porque lo habitual es terminar TLS en un
  reverse proxy (nginx, Caddy, Traefik, un load balancer, etc.).

## Extender el proyecto

Para añadir un nuevo recurso (ej. "usuarios"):

1. `models/user.rs` — struct + `sqlx::FromRow`.
2. `repositories/user_repository.rs` — SQL.
3. `services/user_service.rs` — lógica de negocio.
4. `handlers/user_handler.rs` — handlers `#[web::get/post/...]`.
5. Registrar los handlers en `routes/mod.rs`.
6. Añadir la migración SQL correspondiente en `migrations/`.
