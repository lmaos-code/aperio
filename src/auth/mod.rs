use axum::{
    extract::{Extension, Request},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

mod jwt;

pub use jwt::JwtVerifier;

#[derive(Clone)]
pub struct AuthUser {
    #[allow(dead_code)]
    pub sub: String,
    #[allow(dead_code)]
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct AuthState {
    pub verifier: Option<JwtVerifier>,
}

pub async fn validate_auth(
    Extension(state): Extension<AuthState>,
    Extension(cfg): Extension<crate::config::Config>,
    mut req: Request,
    next: Next,
) -> Response {
    let Some(verifier) = &state.verifier else {
        return next.run(req).await;
    };

    let Some(token) = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return require_auth_response(&cfg, &req);
    };

    match verifier.verify(token) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthUser {
                sub: claims.sub.unwrap_or_default(),
                email: claims.email,
            });
            next.run(req).await
        }
        Err(e) => {
            tracing::warn!("JWT verification failed: {e}");
            tracing::trace!("Token {token}");
            require_auth_response_with_error(&cfg, &req, &e.to_string())
        }
    }
}

fn require_auth_response(cfg: &crate::config::Config, req: &Request) -> Response {
    let resource_url = cfg.public_url.clone().unwrap_or_else(|| {
        let host = req
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");
        format!("http://{host}")
    });

    let resource_metadata_url = format!("{resource_url}/.well-known/oauth-protected-resource");

    let www_auth = format!(
        r#"Bearer realm="{}", resource_metadata="{}""#,
        cfg.mcp_realm, resource_metadata_url,
    );

    let body = serde_json::json!({
        "error": "unauthorized",
        "error_description": "Valid authentication token required. See WWW-Authenticate header for details.",
        "resource_metadata": resource_metadata_url,
    });

    (
        axum::http::StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, www_auth)],
        axum::Json(body),
    )
        .into_response()
}

fn require_auth_response_with_error(
    cfg: &crate::config::Config,
    req: &Request,
    error_detail: &str,
) -> Response {
    let resource_url = cfg.public_url.clone().unwrap_or_else(|| {
        let host = req
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");
        format!("http://{host}")
    });

    let resource_metadata_url = format!("{resource_url}/.well-known/oauth-protected-resource");

    let www_auth = format!(
        r#"Bearer realm="{}", resource_metadata="{}""#,
        cfg.mcp_realm, resource_metadata_url,
    );

    let body = serde_json::json!({
        "error": "unauthorized",
        "error_description": format!("Token verification failed: {error_detail}"),
        "resource_metadata": resource_metadata_url,
    });

    (
        axum::http::StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, www_auth)],
        axum::Json(body),
    )
        .into_response()
}

/// RFC 9728 Protected Resource Metadata
pub async fn protected_resource_metadata(
    axum::extract::Extension(cfg): axum::extract::Extension<crate::config::Config>,
    req: axum::http::request::Parts,
) -> impl IntoResponse {
    let resource_url = cfg.public_url.clone().unwrap_or_else(|| {
        let host = req
            .headers
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("localhost");
        format!("http://{host}")
    });

    let issuer_base = cfg
        .discovery_url
        .trim_end_matches('/')
        .trim_end_matches("/.well-known/openid-configuration")
        .to_string();

    axum::Json(serde_json::json!({
        "resource": resource_url,
        "authorization_servers": [issuer_base],
        "bearer_methods_supported": ["header"],
    }))
}
