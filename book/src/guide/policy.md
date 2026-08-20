# Policy Language (TOML)

Schubert supports declarative policies via TOML files. Enable with the `policy` feature.

## Policy File Format

```toml
# policy.toml
[grassmannian]
k = 2
n = 4

[capabilities.read]
partition = [1]
kind = "ReadLike"
label = "Read access"

[capabilities.write]
partition = [2]
kind = "WriteLike"
label = "Write access"

[capabilities.admin]
partition = [2, 2]
kind = "AdminLike"
label = "Full administration"

[principals.alice]
grants = ["read", "write"]

[principals.bob]
grants = ["read"]

[principals.admin_user]
grants = ["admin"]
```

## Loading Policies

```rust
use schubert::AccessController;

let toml_str = std::fs::read_to_string("policy.toml")?;
let acl = AccessController::from_policy_toml(&toml_str)?;

// Use the loaded controller
let alice = acl.get_principal("alice")?;
acl.check(&alice, &["read", "write"])?;
```

## Exporting Policies

```rust
let toml_str = acl.to_policy_toml()?;
std::fs::write("exported-policy.toml", toml_str)?;
```

## Validation

Policies are validated on load:

- Grassmannian dimensions must satisfy 0 < k < n
- Partitions must be weakly decreasing
- Capability IDs must be unique
- Principal grants must reference registered capabilities
- CapabilityKind must be a valid variant

Invalid policies return descriptive errors with context.

## Constrained Issuance (v0.5.0)

A policy can also constrain **what grants may be issued** from it — closing
the seam between policy-driven controllers and proof-carrying bearer tokens
(Roadmap #20.3). With `crypto` + `policy` enabled:

```rust
use schubert::crypto::{issue_grant_under_policy, CapabilityIssuer, GrantOptions, GrantPolicy};
use schubert::policy::PolicyConfig;

let policy = GrantPolicy::from_policy(&PolicyConfig::from_toml(toml_str)?)?;
let issuer = CapabilityIssuer::from_seed(seed);

// Entitled issuance: exact (id, partition) match required — signs + verifies.
let grant = issue_grant_under_policy(
    &issuer, &policy, "alice",
    &[("read".into(), vec![1])],
    GrantOptions::with_expiry(session_end),
)?;

// Anything beyond the entitlement is denied with a structured error:
// GrantDeniedByPolicy { principal, capability } — including a stronger
// partition smuggled under an allowed capability id.
```

The check fails closed: principals absent from the policy are entitled to
nothing, and both the capability id **and** its Schubert partition must match
the policy's own definition (the policy is the source of geometric truth —
see [`crypto`](../api/crypto.md) for the token lifecycle this feeds).
