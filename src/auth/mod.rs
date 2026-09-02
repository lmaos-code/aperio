use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

mod jwt;

pub use jwt::JwtVerifier;

#[derive(Clone)]
pub struct AuthUser {
    pub sub: String,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct AuthState {
    pub verifier: Option<JwtVerifier>,
    pub resource_url: String,
    pub auth_server_url: String,
}

pub async fn validate_auth(
    State(state): State<AuthState>,
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
        return require_auth_response(&state.resource_url);
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
            require_auth_response(&state.resource_url)
        }
    }
}

fn require_auth_response(resource_url: &str) -> Response {
    let resource_metadata_url = format!(
        "{resource_url}/.well-known/oauth-protected-resource"
    );

    let www_auth = format!(
        r#"Bearer realm="mcp", resource_metadata="{resource_metadata_url}", scope="mcp-read mcp-write""#,
    );

    (
        axum::http::StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, www_auth)],
    )
        .into_response()
}

/// RFC 9728 Protected Resource Metadata
pub async fn protected_resource_metadata(
    axum::extract::State(cfg): axum::extract::State<crate::config::Config>,
) -> impl IntoResponse {
    let resource_url = format!("http://localhost:{}", cfg.svc_port);

    // Strip the .well-known/openid-configuration suffix from issuer_url to get the base issuer
    let issuer_base = cfg
        .issuer_url
        .trim_end_matches('/')
        .trim_end_matches("/.well-known/openid-configuration")
        .to_string();

    axum::Json(serde_json::json!({
        "resource": resource_url,
        "authorization_servers": [issuer_base],
        "bearer_methods_supported": ["header"],
        "scopes_supported": ["mcp:read", "mcp:write"]
    }))
}
