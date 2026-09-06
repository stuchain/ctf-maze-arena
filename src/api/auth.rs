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
use std::collections::HashSet;

const BEARER_PREFIX: &str = "Bearer ";

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: Option<String>,
    pub clock_skew_secs: u64,
    pub auth_mode: AuthMode,
    pub issuer: String,
    pub audience: String,
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
    pub iss: String,
    pub aud: String,
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
        match decode_claims(
            token,
            secret,
            config.clock_skew_secs,
            &config.issuer,
            &config.audience,
        ) {
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
    issuer: &str,
    audience: &str,
) -> Result<AuthClaims, jsonwebtoken::errors::Error> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.leeway = clock_skew_secs;
    validation.algorithms = vec![Algorithm::HS256];
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.required_spec_claims =
        HashSet::from_iter(["sub", "exp", "iat", "iss", "aud"].map(str::to_string));
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
    if !token_data.claims.sub.starts_with("github:")
        || token_data.claims.sub.len() > 255
        || token_data
            .claims
            .name
            .as_ref()
            .is_some_and(|name| name.len() > 100)
        || token_data
            .claims
            .avatar_url
            .as_ref()
            .is_some_and(|url| url.len() > 2048 || !url.starts_with("https://"))
    {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        ));
    }
    Ok(token_data.claims)
}

fn is_protected_route(method: &axum::http::Method, path: &str) -> bool {
    (method == axum::http::Method::POST && path == "/api/leaderboard")
        || (path == "/api/profile"
            && matches!(
                *method,
                axum::http::Method::GET | axum::http::Method::DELETE
            ))
        || (method == axum::http::Method::GET && path == "/api/profile/export")
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
            iss: "ctf-maze-web".into(),
            aud: "ctf-maze-api".into(),
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(decode_claims(&token, "wrong-secret", 60, "ctf-maze-web", "ctf-maze-api").is_err());
        assert_eq!(
            decode_claims(&token, "test-secret", 60, "ctf-maze-web", "ctf-maze-api")
                .unwrap()
                .sub,
            "github:1"
        );
        assert!(decode_claims(&token, "test-secret", 60, "wrong", "ctf-maze-api").is_err());
        assert!(decode_claims(&token, "test-secret", 60, "ctf-maze-web", "wrong").is_err());
    }
    #[test]
    fn claims_reject_wrong_algorithm_expiry_and_future_issue_time() {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = AuthClaims {
            sub: "github:1".into(),
            name: None,
            avatar_url: None,
            iat: now,
            exp: now.saturating_sub(120),
            iss: "ctf-maze-web".into(),
            aud: "ctf-maze-api".into(),
        };
        let expired = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(
            decode_claims(&expired, "test-secret", 60, "ctf-maze-web", "ctf-maze-api").is_err()
        );

        let future = AuthClaims {
            iat: now + 61,
            exp: now + 300,
            ..claims.clone()
        };
        let future = encode(
            &Header::new(Algorithm::HS256),
            &future,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(decode_claims(&future, "test-secret", 60, "ctf-maze-web", "ctf-maze-api").is_err());

        let valid = AuthClaims {
            iat: now,
            exp: now + 300,
            ..claims
        };
        let wrong_algorithm = encode(
            &Header::new(Algorithm::HS384),
            &valid,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(decode_claims(
            &wrong_algorithm,
            "test-secret",
            60,
            "ctf-maze-web",
            "ctf-maze-api"
        )
        .is_err());
    }

    #[test]
    fn claims_require_subject_issuer_audience_and_timestamps() {
        let now = chrono::Utc::now().timestamp() as usize;
        let incomplete = serde_json::json!({
            "exp": now + 300, "iat": now, "iss": "ctf-maze-web", "aud": "ctf-maze-api"
        });
        let token = encode(
            &Header::new(Algorithm::HS256),
            &incomplete,
            &EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        assert!(decode_claims(&token, "test-secret", 60, "ctf-maze-web", "ctf-maze-api").is_err());
    }
    #[test]
    fn only_submission_requires_authentication() {
        assert!(is_protected_route(
            &axum::http::Method::POST,
            "/api/leaderboard"
        ));
        assert!(!is_protected_route(&axum::http::Method::POST, "/api/solve"));
        assert!(is_protected_route(&axum::http::Method::GET, "/api/profile"));
        assert!(is_protected_route(
            &axum::http::Method::DELETE,
            "/api/profile"
        ));
    }
}
