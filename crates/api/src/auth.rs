use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use base64::Engine;
use rflow_core::AppError;

use crate::error::ApiError;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub username: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        if !header_value.starts_with("Basic ") {
            return Err(AppError::Unauthorized.into());
        }

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&header_value[6..])
            .map_err(|_| AppError::Unauthorized)?;

        let credentials = String::from_utf8(decoded).map_err(|_| AppError::Unauthorized)?;
        let (username, password) = credentials
            .split_once(':')
            .ok_or(AppError::Unauthorized)?;

        let expected_user = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let expected_pass = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "changeme".into());

        if username != expected_user || password != expected_pass {
            return Err(AppError::Unauthorized.into());
        }

        Ok(AuthUser {
            username: username.to_string(),
        })
    }
}
