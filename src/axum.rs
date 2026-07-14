// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Axum integration for Schubert capability tokens.
//!
//! Provides an extractor that authenticates requests via Schubert
//! proof-carrying capability tokens in the `Authorization: Bearer <token>`
//! header. The token is base64-encoded using the wire format from
//! [`GrantToken::to_bytes`](crate::crypto::GrantToken::to_bytes).
//!
//! # Example
//!
//! ```ignore
//! use axum::{Extension, Router, routing::get};
//! use std::sync::Arc;
//! use schubert::axum::AuthPrincipal;
//! use schubert::crypto::{CapabilityIssuer, GrantVerifier};
//!
//! let issuer = CapabilityIssuer::generate();
//! let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));
//!
//! let app = Router::new()
//!     .route("/data", get(read_handler))
//!     .layer(Extension(verifier));
//!
//! async fn read_handler(auth: AuthPrincipal) -> String {
//!     format!("hello {}", auth.0.principal)
//! }
//! ```
//!
//! For capability-specific authorization, call [`GrantVerifier::may`] directly
//! in the handler or combine with a custom middleware.

use crate::crypto::{GrantToken, GrantVerifier};
use axum::{
    Extension,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use std::sync::Arc;

/// Axum extractor that validates a Schubert capability token from the
/// `Authorization: Bearer <token>` header and yields the verified grant.
///
/// The extractor fetches the shared [`GrantVerifier`] from axum's
/// [`Extension`] layer. The router must be configured with:
///
/// ```ignore
/// .layer(Extension(Arc::new(grant_verifier)))
/// ```
#[derive(Debug, Clone)]
pub struct AuthPrincipal(pub GrantToken);

/// Error returned when authentication fails; maps to HTTP 401.
#[derive(Debug)]
pub struct AuthError(pub &'static str);

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNAUTHORIZED, self.0).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthPrincipal
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Extension(verifier): Extension<Arc<GrantVerifier>> =
            Extension::from_request_parts(parts, _state)
                .await
                .map_err(|_| AuthError("auth state not installed — add Extension<Arc<GrantVerifier>> to your router"))?;

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError("missing Authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AuthError("expected 'Bearer <token>'"))?;

        // Decode from base64 bearer format
        use base64::{Engine, engine::general_purpose::STANDARD as B64};
        let bytes = B64
            .decode(token.trim())
            .map_err(|_| AuthError("invalid base64 token"))?;

        let grant =
            GrantToken::from_bytes(&bytes).map_err(|_| AuthError("malformed capability token"))?;

        verifier
            .verify(&grant)
            .map_err(|_| AuthError("invalid capability token"))?;

        Ok(AuthPrincipal(grant))
    }
}
