# API Gateway

Pattern for an API gateway using Schubert for authorization.

**Source**: `examples/api_gateway.rs`

## Pattern

The API gateway authenticates (external) and uses Schubert to authorize:

```rust
fn handle_request(
    acl: &AccessController,
    token: &str,
    endpoint: &str,
) -> Result<bool> {
    // 1. Authenticate (external — JWT, OAuth, etc.)
    let principal = authenticate(token)?;

    // 2. Map endpoint to capabilities
    let required = match endpoint {
        "/api/data" => &["read:data"],
        "/api/admin" => &["admin"],
        _ => return Ok(false),
    };

    // 3. Authorize via Schubert
    match acl.check(&principal, required)? {
        AccessDecision::Granted { .. } => Ok(true),
        _ => Ok(false),
    }
}
```

## Key Takeaway

Schubert is a library, not a network service. Embed it in your gateway,
middleware, or sidecar — Schubert handles authorization, your infrastructure
handles authentication and transport.
