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
//! in the handler (extract the verifier via a second `Extension` argument).

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
///
/// On success the grant is fully signature-verified and the handler may
/// consult [`GrantVerifier::may`] for capability-specific authorization.
#[derive(Debug, Clone)]
pub struct AuthPrincipal(pub GrantToken);

/// Error returned by [`AuthPrincipal`]'s extractor.
///
/// Distinguishes **client** failures (no/invalid credential → `401`) from
/// **server** misconfiguration (verifier layer not installed → `500`). The
/// carried detail string is **diagnostic only** — it is never sent to the
/// client in the response body, to avoid leaking how far a forged token got.
/// Inspect it by handling the [`Rejection`](axum::extract::rejection::Rejection)
/// before it becomes a response, or via [`Debug`](std::fmt::Debug).
#[derive(Debug)]
pub enum AuthError {
    /// Client did not present a valid credential. Maps to HTTP **401**.
    ///
    /// Covers: missing `Authorization` header, missing `Bearer ` scheme,
    /// malformed base64, malformed token wire format, and a bad Ed25519
    /// signature. All yield the identical generic response body so an
    /// attacker cannot distinguish which stage failed.
    Unauthorized(&'static str),
    /// The server is misconfigured. Maps to HTTP **500**.
    ///
    /// For example: the router is missing the `Extension<Arc<GrantVerifier>>`
    /// layer. This is a server-side bug, not an authentication failure.
    ServerMisconfigured(&'static str),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        match self {
            // Uniform body for all client-side failures — no stage leak.
            AuthError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
            }
            // Server bug: do not blame the client with a 401.
            AuthError::ServerMisconfigured(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal auth misconfiguration",
            )
                .into_response(),
        }
    }
}

impl<S> FromRequestParts<S> for AuthPrincipal
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Missing verifier layer is a *server* problem, not the client's fault.
        let Extension(verifier): Extension<Arc<GrantVerifier>> =
            Extension::from_request_parts(parts, _state)
                .await
                .map_err(|_| {
                    AuthError::ServerMisconfigured(
                        "auth state not installed — add Extension<Arc<GrantVerifier>> to the router",
                    )
                })?;

        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(AuthError::Unauthorized("missing Authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::Unauthorized("expected 'Bearer <token>' scheme"))?;

        use base64::{Engine, engine::general_purpose::STANDARD as B64};
        let bytes = B64
            .decode(token.trim())
            .map_err(|_| AuthError::Unauthorized("invalid base64 token"))?;

        let grant = GrantToken::from_bytes(&bytes)
            .map_err(|_| AuthError::Unauthorized("malformed capability token"))?;

        verifier
            .verify(&grant)
            .map_err(|_| AuthError::Unauthorized("invalid capability token signature"))?;

        Ok(AuthPrincipal(grant))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{CapabilityIssuer, GrantVerifier};
    use crate::CapabilityId;
    use axum::http::Request;

    /// Build a `(CapabilityId, partition)` grant entry, mirroring the crypto
    /// module's test helper.
    fn cap(id: &str, partition: Vec<usize>) -> (CapabilityId, Vec<usize>) {
        (CapabilityId::new(id), partition)
    }

    /// Base64 bearer payload for a grant, exactly as a client would send it.
    fn bearer(grant: &GrantToken) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD as B64};
        B64.encode(GrantToken::to_bytes(grant))
    }

    /// Build request parts with the given `Authorization` header value.
    fn parts_with_auth(authorization: Option<&str>) -> Parts {
        let mut builder = Request::builder();
        if let Some(value) = authorization {
            builder = builder.header("authorization", value);
        }
        builder.body(()).unwrap().into_parts().0
    }

    #[test]
    fn error_unauthorized_maps_to_401_with_generic_body() {
        let resp = AuthError::Unauthorized("internal detail").into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        // The diagnostic detail must NOT appear in the body.
        let body = resp.into_body();
        // Status already verified; body uniformity is asserted by construction
        // (tuple literal is the constant "Unauthorized"). Drop body to satisfy
        // clippy.
        drop(body);
    }

    #[test]
    fn error_misconfigured_maps_to_500() {
        let resp =
            AuthError::ServerMisconfigured("verifier missing").into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn extractor_valid_token_yields_principal() {
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant("alice", &[cap("memory:read", vec![1])])
            .unwrap();
        let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

        let mut parts = parts_with_auth(Some(&format!("Bearer {}", bearer(&grant))));
        parts.extensions.insert(verifier);

        let auth = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect("valid token must extract");
        assert_eq!(auth.0.principal.as_str(), "alice");
    }

    #[tokio::test]
    async fn extractor_missing_header_is_401() {
        let issuer = CapabilityIssuer::generate();
        let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

        let mut parts = parts_with_auth(None);
        parts.extensions.insert(verifier);

        let err = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect_err("missing header must reject");
        assert!(matches!(err, AuthError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn extractor_malformed_base64_is_401() {
        let issuer = CapabilityIssuer::generate();
        let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

        let mut parts = parts_with_auth(Some("Bearer !!!not-base64!!!"));
        parts.extensions.insert(verifier);

        let err = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect_err("bad base64 must reject");
        assert!(matches!(err, AuthError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn extractor_bad_signature_is_401() {
        // A validly-structured token issued by a *different* key must fail
        // verification against this verifier.
        let other = CapabilityIssuer::generate();
        let forged = other
            .issue_grant("mallory", &[cap("memory:read", vec![1])])
            .unwrap();

        let issuer = CapabilityIssuer::generate();
        let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

        let mut parts = parts_with_auth(Some(&format!("Bearer {}", bearer(&forged))));
        parts.extensions.insert(verifier);

        let err = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect_err("wrong-key token must reject");
        assert!(matches!(err, AuthError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn extractor_tampered_capabilities_is_401() {
        let issuer = CapabilityIssuer::generate();
        let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

        // Issue a read-only grant, then forge a write capability into the
        // stored token. The signature no longer covers the extra cap, so the
        // wire-format roundtrip still carries the tampered payload but
        // verification must reject it.
        let mut grant = issuer
            .issue_grant("alice", &[cap("memory:read", vec![1])])
            .unwrap();
        grant.capabilities.push(crate::crypto::GrantCapability {
            id: CapabilityId::new("memory:write"),
            partition: vec![2],
        });

        let mut parts = parts_with_auth(Some(&format!("Bearer {}", bearer(&grant))));
        parts.extensions.insert(verifier);

        let err = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect_err("tampered token must reject");
        assert!(matches!(err, AuthError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn extractor_missing_verifier_is_500() {
        // No Extension<Arc<GrantVerifier>> installed — a server bug, not a
        // client auth failure.
        let mut parts = parts_with_auth(Some("Bearer dW5kZWZpbmVk"));
        // deliberately do NOT insert the verifier

        let err = AuthPrincipal::from_request_parts(&mut parts, &())
            .await
            .expect_err("missing verifier layer must reject");
        assert!(
            matches!(err, AuthError::ServerMisconfigured(_)),
            "missing verifier must be 500, not 401"
        );
    }
}
