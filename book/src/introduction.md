# Schubert

> Replace boolean allow/deny with geometric access control. Quantitative decisions.
> Impossibility detection. Continuous trust.

Schubert is a Rust library that reimagines access control through **Schubert calculus** — a
branch of algebraic geometry. Instead of returning `true` or `false`, Schubert tells you
*how many* valid configurations exist for a given set of capabilities. When conditions are
geometrically impossible to satisfy together, Schubert catches conflicts that traditional
boolean AND checks would silently approve.

## Why Schubert?

Traditional access control gives you a boolean. You either can or you can't. This breaks
down in complex systems:

- **Two capabilities conflict but individually are fine** — a boolean AND approves.
  Schubert detects the geometrical impossibility.
- **Trust degrades over time** — boolean systems can't express partial trust. Schubert
  models continuous trust with wall-crossing analysis.
- **Cross-domain access is guesswork** — can a capability in one domain translate to
  another? Schubert's Schubert intersection answers exactly.
- **Rate limiting is arbitrary** — Schubert scales rate limits by intersection numbers,
  giving higher-trust principals more throughput.

## The Killer Feature: Impossibility Detection

Consider a user with **write** (σ₂) and **internal-audit** (σ₁₁) capabilities in
Gr(2,4). Each capability is individually valid. Together? They're **geometrically
impossible** — no subspace of ℝ⁴ can simultaneously satisfy both conditions.

A traditional RBAC system with boolean AND would approve. Schubert returns
`AccessDecision::Impossible` and tells you exactly which capabilities conflict.

## Quick Start

```rust
use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = AccessController::new(2, 4)?;
    acl.register_capability(Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike))?;
    acl.register_capability(Capability::new("write", "Write", vec![2], CapabilityKind::WriteLike))?;

    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read")?;
    acl.grant(&alice, "write")?;

    match acl.check(&alice, &["read", "write"])? {
        AccessDecision::Granted { configurations, .. } => {
            println!("Granted: {configurations} valid configurations");
        }
        AccessDecision::Impossible { conflicting } => {
            println!("Impossible: {conflicting:?}");
        }
        _ => println!("Denied or underconstrained"),
    }
    Ok(())
}
```

## What Schubert Is Not

- **Not an authentication system** — identity belongs to your OAuth/OIDC provider
- **Not a network service** — Schubert is a library you embed
- **Not a policy server** — no REST API, no gRPC, no wire protocol
- **Not a single Grassmannian** — `MultiController` manages cross-domain access

## The Industrial Algebra Ecosystem

Schubert depends on three sibling projects:

| Crate | Version | Role |
|---|---|---|
| Crate | Version | Role | Required? |
|---|---|---|---|
| [Amari](https://github.com/Industrial-Algebra/Amari) | 0.23 | Schubert calculus engine — Grassmannians, intersection numbers | **Yes** (core) |
| [Karpal](https://github.com/Industrial-Algebra/Karpal) | 0.6 | Formal verification — type-level proofs, SMT/Lean obligations | Optional (`karpal` / `karpal-verify` feature) |
| [Minuet](https://github.com/Industrial-Algebra/Minuet) | 0.5 | Holographic memory — cosine-similarity access patterns | Optional (`holographic` feature) |

Only Amari is a hard dependency. Karpal and Minuet are opt-in features for
formal verification and holographic access patterns respectively.

## License

Schubert is licensed under **Apache-2.0** — a permissive open-source license
with patent grant and attribution requirements. See [LICENSE](https://github.com/Industrial-Algebra/Schubert/blob/main/LICENSE)
for the full text. All contributors must sign the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).

For licensing inquiries: <license@industrialalgebra.com>
