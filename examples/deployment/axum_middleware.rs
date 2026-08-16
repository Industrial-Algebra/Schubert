// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Axum middleware using Schubert for authorization.
//!
//! Demonstrates integrating Schubert as authorization middleware in a web
//! service. Authentication is external (OAuth/JWT) — Schubert handles
//! authorization decisions.

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use schubert::{AccessController, Capability, CapabilityKind};
use std::sync::Arc;

/// In-memory shared access controller.
struct AppState {
    acl: Arc<AccessController>,
}

/// Axum middleware that checks Schubert access before allowing the request.
async fn schubert_auth(
    req: Request,
    next: Next,
) -> Response {
    // 1. Extract authenticated principal from request (JWT, session, etc.)
    let principal_id = req
        .headers()
        .get("X-Principal-Id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");

    // 2. Map endpoint to required capabilities
    let path = req.uri().path();
    let required = match path {
        "/api/data" => &["read:data"][..],
        "/api/admin" => &["admin"][..],
        _ => return next.run(req).await,
    };

    // 3. Extract shared ACL from app state
    let state = req
        .extensions()
        .get::<Arc<AccessController>>()
        .cloned();

    let Some(acl) = state else {
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "ACL not configured").into_response();
    };

    // 4. Authorize via Schubert
    let principal = schubert::PrincipalId::new(principal_id);
    match acl.check(&principal, required) {
        Ok(schubert::AccessDecision::Granted { .. }) => {
            next.run(req).await
        }
        Ok(schubert::AccessDecision::Impossible { conflicting }) => {
            tracing::warn!(
                "Geometrically impossible access: {:?} for {}",
                conflicting, principal_id
            );
            (axum::http::StatusCode::FORBIDDEN, "Access geometrically impossible").into_response()
        }
        _ => {
            (axum::http::StatusCode::FORBIDDEN, "Access denied").into_response()
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize Schubert
    let mut acl = AccessController::new(2, 4).unwrap();
    acl.register_capability(Capability::new(
        "read:data", "Read data", vec![1], CapabilityKind::ReadLike,
    )).unwrap();
    acl.register_capability(Capability::new(
        "write:data", "Write data", vec![2], CapabilityKind::WriteLike,
    )).unwrap();
    acl.register_capability(Capability::new(
        "admin", "Admin", vec![2, 1], CapabilityKind::AdminLike,
    )).unwrap();

    let alice = acl.create_principal("alice").unwrap();
    acl.grant(&alice, "read:data").unwrap();
    acl.grant(&alice, "write:data").unwrap();

    let shared_acl = Arc::new(acl);

    // Build router with Schubert middleware
    let app = Router::new()
        .route("/api/data", get(|| async { "data here" }))
        .route("/api/admin", get(|| async { "admin here" }))
        .layer(axum::middleware::from_fn(schubert_auth))
        .with_state(shared_acl);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Schubert-protected API running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_controller_is_send_sync() {
        // Verify AccessController can be shared across threads
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AccessController>();
    }
}
