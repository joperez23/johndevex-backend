# Finance Rates Backend

Backend de alto rendimiento desarrollado en **Rust** utilizando **Ntex v3.12.3** y **SQLx** con **PostgreSQL**. Este servicio se encarga de extraer, calcular y almacenar tipos de cambio e indicadores financieros (BCV, Binance P2P, Datos.gov.co y BanRep) con precisión decimal estricta (`bigdecimal::BigDecimal`).

---

## 🚀 Características Principales

- **Framework Web:** [Ntex v3.12.3](https://ntex.rs/) asíncrono y de ultra bajo _overhead_.
- **Persistencia de Datos:** [SQLx v0.8](https://github.com/launchbadge/sqlx) conectando a PostgreSQL de forma asíncrona.
- **Precisión Financiera:** Uso estricto de `bigdecimal::BigDecimal` (`numeric(20, 8)`) para evitar errores de redondeo de punto flotante.
- **Procesamiento de Datos:**
  - **Scraping HTML:** Extracción resiliente mediante CSS selectors y regex fallback para el Banco Central de Venezuela (BCV).
  - **Filtrado Inteligente de API:** Filtrado de anuncios "Promoted Ad" en Binance P2P y selección de indicadores tipo "Media" en BanRep.
  - **Normalización Numérica:** Conversión automática de formatos regionales latinos/europeos (p. ej., `3.602,33` $\rightarrow$ `3602.33`).
- **Estrategia Upsert:** Clave única compuesta por `(pair, date)` que actualiza el valor si ya existe un registro para el día actual.

---

## 📁 Arquitectura del Proyecto

```text
finance-rates-backend/
├── Cargo.toml               # Dependencias y versiones fijadas de Ntex
├── .env.example             # Plantilla de variables de entorno
├── init.sql                 # DDL de la base de datos PostgreSQL
└── src/
    ├── main.rs              # Punto de entrada de la aplicación y estado
    ├── config.rs            # Carga y validación de variables de entorno
    ├── db.rs                # Configuración del pool de conexiones SQLx
    ├── error.rs             # Manejo centralizado de errores y respuestas HTTP
    ├── models/
    │   ├── mod.rs
    │   └── rate.rs          # Modelos DTOs y mapeos de BD
    ├── clients/
    │   ├── mod.rs
    │   ├── bcv.rs           # Scraper HTML para USDVES y EURVES
    │   ├── binance.rs       # Cliente REST API para USDTVES (Binance P2P)
    │   ├── datos_gov.rs     # Cliente REST API para USDCOP (Datos.gov.co)
    │   └── banrep.rs        # Cliente REST API para EURCOP (BanRep)
    ├── services/
    │   ├── mod.rs
    │   └── rate_service.rs  # Lógica de negocio y sincronizaciones
    └── handlers/
        ├── mod.rs
        └── rate_handler.rs  # Rutas y controladores HTTP
```
