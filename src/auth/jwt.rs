use anyhow::Context;
use jsonwebtoken::jwk::JwkSet;
use tracing::error;

use crate::mcp::AperioError;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct JwtVerifier {
    key_set: JwkSet,
    pub validation: jsonwebtoken::Validation,
}

impl JwtVerifier {
    pub async fn new(cfg: &crate::config::Config) -> anyhow::Result<Self> {
        let jwks = get_jwks(&cfg.issuer_url).await.map_err(|e| {
            error!("Failed to fetch OIDC JWKS: {e}");
            e
        })?;
        Ok(Self {
            key_set: jwks,
            validation: jsonwebtoken::Validation::new_for_family(
                jsonwebtoken::AlgorithmFamily::Rsa,
            ),
        })
    }

    #[allow(clippy::unused_self)]
    pub fn verify(&self, _token: &str) -> Result<Claims, AperioError> {
        Err(AperioError::BadRequest("Not Implemented".to_string()))
    }
}

async fn get_jwks(url: &str) -> Result<JwkSet, anyhow::Error> {
    let req = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    let jwks_uri = req
        .get("jwks_uri")
        .with_context(|| "OIDC JWKS uri not found")?
        .to_string()
        .replace('\"', "");

    let sanitized_url = url::Url::parse(&jwks_uri)
        .context("Could not parse JWKS URL")?;

    reqwest::get(sanitized_url.to_string())
        .await?
        .json::<JwkSet>()
        .await
        .with_context(|| "Could not get JWKS for configured OIDC-Service")
}
