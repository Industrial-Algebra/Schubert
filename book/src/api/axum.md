# Axum Integration

Bearer-token authentication for Axum web services, built on the
[`crypto`](./crypto.md) grant tokens. Enable the `axum` feature (which also
enables `crypto`):

```toml
[dependencies]
schubert = { version = "0.4", features = ["axum"] }
```

The [`AuthPrincipal`](#the-authprincipal-extractor) extractor validates a
Schubert `GrantToken` from the `Authorization: Bearer <token>` header and yields
the verified grant to your handler. The token is base64-encoded using
[`GrantToken::to_bytes`](./crypto.md#wire-format).

## Minimal Example

```rust
use axum::{Extension, Router, routing::get};
use std::sync::Arc;
use schubert::axum::AuthPrincipal;
use schubert::crypto::{CapabilityIssuer, GrantVerifier};

let issuer = CapabilityIssuer::from_seed(/* persisted seed */);
let verifier = Arc::new(GrantVerifier::new(issuer.public_key()));

let app = Router::new()
    .route("/data", get(read_handler))
    .layer(Extension(verifier));  // <- verifier shared with every handler

async fn read_handler(auth: AuthPrincipal) -> String {
    format!("hello {}", auth.0.principal)
}
```

## The `AuthPrincipal` Extractor

`AuthPrincipal(pub GrantToken)` implements `FromRequestParts`. On success the
inner `GrantToken` is **fully signature-verified** — handlers can trust its
`principal` and `capabilities` fields.

The extractor pulls the shared verifier from an `Extension<Arc<GrantVerifier>>`
layer, so you must install it on the router (as above). Handlers may also
extract the verifier themselves to call `may()` for capability-specific
authorization:

```rust
use axum::Extension;
use std::sync::Arc;
use schubert::axum::AuthPrincipal;
use schubert::crypto::GrantVerifier;
use axum::http::StatusCode;

async fn write_handler(
    auth: AuthPrincipal,
    Extension(verifier): Extension<Arc<GrantVerifier>>,
) -> Result<String, (StatusCode, &'static str)> {
    // Geometric containment: does this grant authorize a write ([2])?
    if !verifier.may(&auth.0, &[2]) {
        return Err((StatusCode::FORBIDDEN, "write not granted"));
    }
    Ok(format!("writing as {}", auth.0.principal))
}
```

## Error Responses: 401 vs 500

Rejections are typed, not lumped into a single 401:

| Error variant | HTTP | Meaning |
|---|---|---|
| `AuthError::Unauthorized(_)` | **401** | Client problem: missing/malformed header, bad base64, invalid signature. |
| `AuthError::ServerMisconfigured(_)` | **500** | Server problem: the `Extension<Arc<GrantVerifier>>` layer is missing. |

A missing verifier layer is a server bug, not an authentication failure — so it
returns `500`, not `401`.

**No information leak:** all `Unauthorized` causes (missing header, malformed
token, bad signature) yield the *identical* generic `Unauthorized` response
body. The diagnostic detail is kept on the error value (visible via `Debug` if
you handle the rejection before it becomes a response), but is never sent to the
client — an attacker cannot tell how far a forged token got.

```rust
// Handle rejections yourself (optional) to log the diagnostic detail:
match AuthPrincipal::from_request_parts(&mut parts, &state).await {
    Ok(auth) => { /* ... */ }
    Err(schubert::axum::AuthError::Unauthorized(detail)) => {
        tracing::warn!("auth failed: {detail}");  // logged, not sent to client
        // return a uniform 401
    }
    Err(schubert::axum::AuthError::ServerMisconfigured(detail)) => {
        tracing::error!("misconfiguration: {detail}");  // return 500
    }
}
```

## Token Lifecycle

Tokens are issued server-side with the [`crypto`](./crypto.md) module and handed
to clients (e.g. on login). The client sends them back on each request:

```text
Authorization: Bearer <base64(GrantToken::to_bytes(grant))>
```

The extractor decodes, parses, and verifies in one step; the handler receives a
ready-to-use `AuthPrincipal`. See [`crypto`](./crypto.md) for issuing grants,
key persistence (`KeyStore`), and the geometric-containment `may()` check.

### Expiry is enforced by the extractor (v0.5.0)

The extractor calls `GrantVerifier::verify`, which checks the signed
`expires_at` against the wall clock — **an expired grant is a 401**, the same
client-failure class as a bad signature, with no extra wiring. Issue
session-scoped or time-boxed grants with
[`issue_grant_with_expiry`](./crypto.md#grant-expiry--nonce-v050):

```rust
// Session grant: dies at the boundary — extractor rejects it as 401 after.
let session = issuer.issue_grant_with_expiry(
    "alice", &[(cap_id, partition)], session_end_unix)?;
```

There is no renewal mutation: rotation is re-issue (the issuance nonce makes
the fresh bearer distinct from the old one even from the same seed). Routine
deprovisioning uses expiry; incident response stays with the consumer-side
revocation list (ADR-0001: they compose as OR).
