use anyhow::Context;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use tracing::error;

use crate::mcp::AperioError;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Claims {
    pub sub: Option<String>,
    pub exp: usize,
    pub iss: Option<String>,
    pub aud: Option<serde_json::Value>,
    pub email: Option<String>,
}

#[derive(Clone)]
pub struct JwtVerifier {
    key_set: JwkSet,
    validation: Validation,
    required_claims: Vec<String>,
}

impl JwtVerifier {
    pub async fn new(cfg: &crate::config::Config) -> anyhow::Result<Self> {
        let jwks = get_jwks(&cfg.discovery_url).await.map_err(|e| {
            error!("Failed to fetch OIDC JWKS: {e}");
            e
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.leeway = 30;
        validation.validate_aud = false;

        if let Some(ref audience) = cfg.required_audience {
            validation.validate_aud = true;
            tracing::trace!("Setting Audience to {audience}");
            validation.set_audience(&[audience.as_str()]);
        }

        if let Some(ref issuer) = cfg.required_issuer {
            tracing::trace!("Setting issuer to {issuer}");
            validation.set_issuer(&[issuer.as_str()]);
        }

        Ok(Self {
            key_set: jwks,
            validation,
            required_claims: cfg.required_claims.clone(),
        })
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AperioError> {
        let header = decode_header(token)
            .map_err(|e| AperioError::Unauthorized(format!("Invalid token header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AperioError::Unauthorized("Token missing kid header".to_string()))?;

        let jwk = self
            .key_set
            .keys
            .iter()
            .find(|k| k.common.key_id.as_deref() == Some(&kid))
            .ok_or_else(|| AperioError::Unauthorized(format!("No matching key for kid: {kid}")))?;

        let decoding_key = DecodingKey::from_jwk(jwk).map_err(|e| {
            AperioError::Unauthorized(format!("Failed to create decoding key: {e}"))
        })?;

        let token_data = decode::<Claims>(token, &decoding_key, &self.validation)
            .map_err(|e| AperioError::Unauthorized(format!("Token validation failed: {e}")))?;

        let claims = token_data.claims;

        for required_claim in &self.required_claims {
            if !has_claim(&claims, required_claim) {
                return Err(AperioError::Unauthorized(format!(
                    "Missing required claim: {required_claim}"
                )));
            }
        }

        Ok(claims)
    }
}

fn has_claim(claims: &Claims, claim_name: &str) -> bool {
    match claim_name {
        "sub" => claims.sub.is_some(),
        "email" => claims.email.is_some(),
        "aud" => claims.aud.is_some(),
        "iss" => claims.iss.is_some(),
        "exp" => true,
        _ => false,
    }
}

async fn get_jwks(url: &str) -> Result<JwkSet, anyhow::Error> {
    let req = reqwest::get(url).await?.json::<serde_json::Value>().await?;

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
