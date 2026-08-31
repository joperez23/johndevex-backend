# Finance Rates & Messaging Backend

Backend de alto rendimiento desarrollado en **Rust** utilizando **Ntex v3.12.3** y **SQLx** con **PostgreSQL**. Este servicio se encarga de:
1. Extraer, calcular y almacenar tipos de cambio e indicadores financieros (BCV, Binance P2P, Datos.gov.co y BanRep) con precisión decimal estricta (`bigdecimal::BigDecimal`).
2. Gestionar el envío de mensajes por **WhatsApp** mediante un worker automatizado en segundo plano con control de tasa (30s).
3. Gestionar el despacho de **Correos Electrónicos** vía Gmail SMTP mediante un worker en segundo plano encolado con cadencia configurable (15s por correo) y soporte de listas/lotes.

---

## 🚀 Características Principales

- **Framework Web:** [Ntex v3.12.3](https://ntex.rs/) asíncrono sobre Tokio runtime.
- **Persistencia de Datos:** [SQLx v0.9](https://github.com/launchbadge/sqlx) conectando a PostgreSQL de forma asíncrona.
- **Precisión Financiera:** Uso estricto de `bigdecimal::BigDecimal` (`numeric(20, 8)`) para evitar errores de redondeo de punto flotante.
- **Procesamiento de Tasas:**
  - **Scraping HTML:** Extracción resiliente mediante CSS selectors y regex fallback para el Banco Central de Venezuela (BCV).
  - **Filtrado Inteligente de API:** Filtrado de anuncios "Promoted Ad" en Binance P2P y selección de indicadores tipo "Media" en BanRep.
  - **Normalización Numérica:** Conversión automática de formatos regionales latinos/europeos (p. ej., `3.602,33` $\rightarrow$ `3602.33`).
  - **Estrategia Upsert:** Clave única compuesta por `(pair, date)` que actualiza el valor si ya existe un registro para el día actual.
- **Worker de WhatsApp (Headless Chrome):**
  - Cola en segundo plano con límite de velocidad de 30 segundos por mensaje.
  - Endpoint `/api/whatsapp/qr` para escanear el código QR directamente desde el navegador en servidores headless.
  - Notificaciones automáticas por correo en caso de fallos de entrega.
- **Worker de Cola de Correos (Gmail SMTP):**
  - Despacho desacoplado en segundo plano con intervalo de 15 segundos entre envíos para evitar bloqueos por rate limiting / spam de SMTP.
  - Soporte para encolar envíos individuales (`/api/email/send`) y listas/lotes de correos (`/api/email/send-batch`).
  - Endpoint de métricas y estado de la cola (`/api/email/queue-status`).

---

## 📁 Arquitectura del Proyecto

```text
finance-rates-backend/
├── Cargo.toml               # Dependencias y configuración de Rust
├── .env.example             # Plantilla de variables de entorno
├── init.sql                 # DDL del esquema y tabla PostgreSQL
├── TestBackend.rest         # Colección de peticiones HTTP para pruebas
└── src/
    ├── main.rs              # Punto de entrada, inicio de workers y servidor Ntex
    ├── config.rs            # Carga y validación de variables de entorno
    ├── db.rs                # Configuración del pool de conexiones SQLx
    ├── error.rs             # Manejo centralizado de errores y respuestas HTTP
    ├── models/
    │   ├── mod.rs
    │   ├── rate.rs          # Modelos DTOs y mapeos de BD para tasas
    │   ├── whatsapp.rs      # Modelos de mensajes WhatsApp
    │   └── email.rs         # Modelos de correos (single, batch, status)
    ├── clients/
    │   ├── mod.rs
    │   ├── bcv.rs           # Scraper HTML para USDVES y EURVES
    │   ├── binance.rs       # Cliente REST API para USDTVES (Binance P2P)
    │   ├── datos_gov.rs     # Cliente REST API para USDCOP (Datos.gov.co)
    │   └── banrep.rs        # Cliente REST API para EURCOP (BanRep)
    ├── services/
    │   ├── mod.rs
    │   ├── rate_service.rs  # Lógica de negocio y sincronización de tasas
    │   ├── whatsapp_worker.rs # Worker automatizado de WhatsApp con Headless Chrome
    │   └── email_service.rs # Worker y despachador en cola de emails (15s)
    └── handlers/
        ├── mod.rs
        ├── rate_handler.rs  # Rutas HTTP para tasas de cambio
        ├── whatsapp_handler.rs # Rutas HTTP para WhatsApp y visualización de QR
        └── email_handler.rs # Rutas HTTP para envío individual, lote y estado de emails
```

---

## 📡 Endpoints Disponibles

### Tasas Financieras (`/api/finance`)
- `POST /api/finance/sync/usd-ves` — Sincroniza USD/VES desde BCV
- `POST /api/finance/sync/eur-ves` — Sincroniza EUR/VES desde BCV
- `POST /api/finance/sync/usdt-ves` — Sincroniza USDT/VES desde Binance P2P
- `POST /api/finance/sync/usd-cop` — Sincroniza USD/COP desde Datos.gov.co
- `POST /api/finance/sync/eur-cop` — Sincroniza EUR/COP desde BanRep
- `POST /api/finance/sync/all` — Sincroniza todas las tasas disponibles
- `GET /api/finance/latest?pair=USDVES` — Consulta las tasas más recientes
- `GET /api/finance/history?pair=USDVES&limit=50` — Consulta el histórico de tasas

### Mensajería WhatsApp (`/api/whatsapp`)
- `POST /api/whatsapp/send` — Encola un mensaje para envío por WhatsApp (ritmo de 30s)
- `GET /api/whatsapp/qr` — Muestra el screenshot del código QR en vivo para vincular el dispositivo

### Envío de Correos (`/api/email`)
- `POST /api/email/send` — Encola un correo individual (despachado con ritmo de 15s)
- `POST /api/email/send-batch` — Encola una lista/lote de correos para envío secuencial cada 15s
- `GET /api/email/queue-status` — Consulta la cantidad de correos pendientes y el estado del worker

