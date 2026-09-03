use super::error::ApiError;
use super::request_id::request_id;
use axum::{
    extract::State,
    http::{HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: Option<String>,
    pub clock_skew_secs: u64,
    pub auth_mode: AuthMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Anonymous,
    Jwt,
    OptionalJwt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

pub async fn jwt_claims_middleware(
    State(config): State<JwtConfig>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if config.auth_mode == AuthMode::Anonymous {
        return next.run(req).await;
    }
    let request_id = request_id(&req);
    let protected = is_protected_route(req.method(), req.uri().path());
    let token = match extract_bearer_token(req.headers().get(axum::http::header::AUTHORIZATION)) {
        Ok(token) => token,
        Err(()) => {
            return ApiError::new(
                StatusCode::UNAUTHORIZED,
                "invalid_authorization",
                "Authorization must use a valid Bearer token.",
                request_id,
            )
            .into_response()
        }
    };
    if protected && token.is_none() {
        return ApiError::new(
            StatusCode::UNAUTHORIZED,
            "authentication_required",
            "Sign in with GitHub to continue.",
            request_id,
        )
        .into_response();
    }
    if let Some(token) = token {
        let Some(secret) = config.secret.as_deref() else {
            return ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "authentication_unavailable",
                "Authentication is temporarily unavailable.",
                request_id,
            )
            .into_response();
        };
        match decode_claims(token, secret, config.clock_skew_secs) {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
            }
            Err(_) => {
                return ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "invalid_token",
                    "The authentication token is invalid or expired.",
                    request_id,
                )
                .into_response()
            }
        }
    }
    next.run(req).await
}

fn extract_bearer_token(value: Option<&HeaderValue>) -> Result<Option<&str>, ()> {
    let Some(value) = value else {
        return Ok(None);
    };
    let raw = value.to_str().map_err(|_| ())?;
    let token = raw.strip_prefix(BEARER_PREFIX).ok_or(())?.trim();
    if token.is_empty() {
        Err(())
    } else {
        Ok(Some(token))
    }
}

fn decode_claims(
    token: &str,
    secret: &str,
    clock_skew_secs: u64,
) -> Result<AuthClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = clock_skew_secs;
    let token_data = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    let now = chrono::Utc::now().timestamp().max(0) as usize;
    if token_data.claims.iat > now + clock_skew_secs as usize {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }
    Ok(token_data.claims)
}

fn is_protected_route(method: &axum::http::Method, path: &str) -> bool {
    method == axum::http::Method::POST && path == "/api/leaderboard"
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    #[test]
    fn bearer_tokens_are_strict() {
        assert_eq!(
            extract_bearer_token(Some(&HeaderValue::from_static("Bearer token123"))).unwrap(),
            Some("token123")
        );
        assert!(extract_bearer_token(Some(&HeaderValue::from_static("Basic abc"))).is_err());
    }
    #[test]
    fn claims_validate_signature() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = AuthClaims {
            sub: "github:1".into(),
            name: Some("tester".into()),
            avatar_url: None,
            iat: now,
            exp: now + 300,
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(decode_claims(&token, "wrong-secret", 60).is_err());
        assert_eq!(
            decode_claims(&token, "test-secret", 60).unwrap().sub,
            "github:1"
        );
    }
    #[test]
    fn only_submission_requires_authentication() {
        assert!(is_protected_route(
            &axum::http::Method::POST,
            "/api/leaderboard"
        ));
        assert!(!is_protected_route(&axum::http::Method::POST, "/api/solve"));
    }
}
