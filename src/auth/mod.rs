use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::mcp::{AppState, AperioError};

mod jwt;

pub use jwt::JwtVerifier;

#[derive(Clone)]
pub struct AuthUser {
    claims: jwt::Claims,
}

pub async fn validate_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AperioError> {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AperioError::Unauthorized("Bearer Token not found".to_string()))?;

    let claims = state.jwt_verifier.verify(token)?;

    req.extensions_mut().insert(AuthUser { claims });

    Ok(next.run(req).await)
}
