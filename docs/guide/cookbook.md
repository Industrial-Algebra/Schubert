# Integration Cookbook

Recipes for integrating Schubert with common identity and infrastructure systems.

## OAuth 2.0 / OpenID Connect

Map OAuth scopes to Schubert capabilities:

```rust
fn oauth_scopes_to_capabilities(scopes: &[&str]) -> Vec<Capability> {
    scopes.iter().map(|scope| {
        match *scope {
            "read" => Capability::new("read", "Read", vec![1], ReadLike),
            "write" => Capability::new("write", "Write", vec![2], WriteLike),
            "admin" => Capability::new("admin", "Admin", vec![2, 2], AdminLike),
            s => Capability::new(s, s, vec![1], Custom),
        }
    }).collect()
}

// Map JWT subject to PrincipalId
let principal = PrincipalId::new(jwt_claims.sub);
acl.grant(&principal, "read")?;
```

## JWT-Based Authentication

```rust
fn verify_and_check(
    acl: &AccessController,
    token: &str,
    required: &[&str],
) -> Result<AccessDecision> {
    let claims = verify_jwt(token)?;           // your JWT library
    let principal = PrincipalId::new(claims.sub);
    acl.check(&principal, required)
}
```

## Database Row-Level Security

```rust
// Tenant-scoped capabilities
acl.register_capability(Capability::new(
    "read:tenant_a", "Read tenant A", vec![1], ReadLike,
))?;

// Multi-tenant principal
acl.grant(&principal, "read:tenant_a")?;
acl.grant(&principal, "read:tenant_b")?;

// Check cross-tenant — geometrically impossible combinations detected
let result = acl.check(&principal, &[
    "read:tenant_a", "read:tenant_b", "read:tenant_c",
])?;
```

## Kubernetes RBAC

```rust
// Map K8s roles to Schubert partitions
let roles = [
    ("viewer",  vec![1]),      // σ₁
    ("editor",  vec![1, 2]),   // σ₁ + σ₂
    ("operator", vec![1, 2, 1]), // σ₁ + σ₂₁
    ("admin",   vec![2, 2]),   // σ₂₂ (point class)
];

for (role, partition) in roles {
    acl.register_capability(Capability::new(
        &format!("role:{role}"), role, partition, AdminLike,
    ))?;
}
```

## Policy-as-Code (TOML)

```toml
# policy.toml
[grassmannian]
k = 2
n = 4

[capabilities.read]
partition = [1]
kind = "ReadLike"
label = "Read"

[principals.alice]
grants = ["read"]
```

```rust
let acl = AccessController::from_policy_toml(
    &std::fs::read_to_string("policy.toml")?
)?;
```

## Audit Integration

```rust
struct DatabaseAudit {
    pool: PgPool,
}

impl AuditSink for DatabaseAudit {
    fn record(&self, record: &DecisionRecord) -> schubert::Result<()> {
        // Async DB write via tokio::task::spawn_blocking
        Ok(())
    }
}

acl.set_audit_sink(Box::new(DatabaseAudit { pool }));
// Every check() call now records to the database
```

## Rate Limiting

```rust
let mut rl = RateLimiter::new(10.0, 1.0);
rl.configure_from_decision("alice", &granted_decision)?;

// Per-request rate check
if rl.try_consume("alice").is_err() {
    return Err("rate limit exceeded");
}
```

## Cryptographic Tokens

```rust
// Issuer
let issuer = CapabilityIssuer::generate();
let token = issuer.issue("alice", "read:data")?;

// Verifier (separate service)
let verifier = CapabilityVerifier::new(issuer.public_key());
verifier.verify(&token)?;
let (principal, capability) = verifier.verify_and_extract(&token)?;
acl.grant(principal, capability.as_str())?;
```

## Temporal Access (Time-Limited Grants)

```rust
let cap = Capability::new("temp", "Temporary", vec![1], ReadLike)
    .with_expiry(now + 3_600_000); // 1 hour

acl.register_capability(cap)?;
acl.grant(&principal, "temp")?;

// Later:
acl.check_temporal(&principal, &["temp"], now)?;       // OK
acl.check_temporal(&principal, &["temp"], later)?;      // Denied
```

## Multi-Grassmannian Cross-Domain

```rust
let mut mc = MultiController::new();
let rbac = mc.add_domain(2, 4)?;     // RBAC domain
let tenant = mc.add_domain(3, 6)?;   // Multi-tenant domain

mc.create_principal("alice", &rbac)?;
mc.grant_in_domain(&alice, "read", &rbac)?;

// Check if RBAC capability works in the tenant domain:
mc.check_cross_domain(&alice, &["read"], &rbac, &tenant)?;
```
