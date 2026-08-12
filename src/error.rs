use ntex::http::StatusCode;
use ntex::web;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("External HTTP API error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Scraping or parsing error: {0}")]
    Parse(String),

    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl web::error::WebResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Http(_) | AppError::Parse(_) => StatusCode::BAD_GATEWAY,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        let status = self.status_code();
        let payload = serde_json::json!({
            "error": self.to_string(),
            "code": status.as_u16(),
        });
        web::HttpResponse::build(status).json(&payload)
    }
}
