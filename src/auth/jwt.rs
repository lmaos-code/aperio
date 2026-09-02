use anyhow::Context;
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, Algorithm, DecodingKey, Validation};
use tracing::error;

use crate::mcp::AperioError;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Claims {
    pub sub: Option<String>,
    pub exp: usize,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct JwtVerifier {
    key_set: JwkSet,
    validation: Validation,
}

impl JwtVerifier {
    pub async fn new(cfg: &crate::config::Config) -> anyhow::Result<Self> {
        let jwks = get_jwks(&cfg.issuer_url).await.map_err(|e| {
            error!("Failed to fetch OIDC JWKS: {e}");
            e
        })?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        Ok(Self {
            key_set: jwks,
            validation,
        })
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AperioError> {
        let header =
            decode_header(token).map_err(|e| AperioError::Unauthorized(format!("Invalid token header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AperioError::Unauthorized("Token missing kid header".to_string()))?;

        let jwk = self
            .key_set
            .keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(&kid))
            .ok_or_else(|| AperioError::Unauthorized(format!("No matching key for kid: {kid}")))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|e| AperioError::Unauthorized(format!("Failed to create decoding key: {e}")))?;

        let token_data = decode::<Claims>(token, &decoding_key, &self.validation)
            .map_err(|e| AperioError::Unauthorized(format!("Token validation failed: {e}")))?;

        Ok(token_data.claims)
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

    let sanitized_url = url::Url::parse(&jwks_uri).context("Could not parse JWKS URL")?;

    reqwest::get(sanitized_url.to_string())
        .await?
        .json::<JwkSet>()
        .await
        .with_context(|| "Could not get JWKS for configured OIDC-Service")
}
