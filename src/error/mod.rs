//! Tipo de error único para toda la aplicación.
//!
//! Sigue el patrón recomendado en la documentación de ntex
//! (https://ntex.rs/docs/errors): un enum que implementa `Display` + `Debug`
//! y el trait `ntex::web::error::WebResponseError`, de forma que cualquier
//! handler pueda simplemente devolver `Result<T, AppError>` y ntex se encarga
//! de traducirlo a una respuesta HTTP con el código de estado adecuado.

use ntex::http::StatusCode;
use ntex::web::error::WebResponseError;
use ntex::web::{HttpRequest, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("error de base de datos: {0}")]
    Database(#[from] sqlx::Error),

    #[error("error al scrapear el BCV: {0}")]
    Scraping(String),

    #[error("recurso no encontrado: {0}")]
    NotFound(String),

    #[error("solicitud inválida: {0}")]
    BadRequest(String),

    #[error("error interno: {0}")]
    Internal(String),
}

/// Cuerpo JSON uniforme para todas las respuestas de error.
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database_error",
            AppError::Scraping(_) => "scraping_error",
            AppError::NotFound(_) => "not_found",
            AppError::BadRequest(_) => "bad_request",
            AppError::Internal(_) => "internal_error",
        }
    }
}

impl WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Scraping(_) => StatusCode::BAD_GATEWAY,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _req: &HttpRequest) -> HttpResponse {
        // Los errores 5xx se registran como advertencia/error en el log;
        // los 4xx normalmente no requieren tanto ruido.
        if self.status_code().is_server_error() {
            log::error!("{self}");
        } else {
            log::warn!("{self}");
        }

        HttpResponse::build(self.status_code()).json(&ErrorBody {
            error: self.error_code(),
            message: self.to_string(),
        })
    }
}
