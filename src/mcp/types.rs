use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AperioError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    #[allow(dead_code)]
    Forbidden(String),

    #[error("not found: {0}")]
    #[allow(dead_code)]
    NotFound(String),

    #[error("bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),

    #[error("conflict: {0}")]
    #[allow(dead_code)]
    Conflict(String),

    #[error("sync error: {0}")]
    #[allow(dead_code)]
    Sync(String),

    #[error("vault error: {0}")]
    #[allow(dead_code)]
    Vault(String),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    message: String,
}

impl IntoResponse for AperioError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Sync(_) => StatusCode::BAD_GATEWAY,
            Self::Vault(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorBody {
            error: status.canonical_reason().unwrap_or("unknown").to_string(),
            message: self.to_string(),
        };

        (status, axum::Json(body)).into_response()
    }
}
