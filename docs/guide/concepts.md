# Concepts

Schubert uses **Schubert calculus** — a branch of algebraic geometry — to make
access control decisions. You don't need to be a mathematician to use it, but
understanding the core concepts helps design better policies.

## Capabilities as Geometric Conditions

A **capability** is a Schubert condition — a geometric constraint on a
Grassmannian Gr(k,n). Each capability has a **partition** (like `[1]` or `[2,1]`)
that defines a codimension — how restrictive the condition is.

| Partition | Codimension | Typical Use |
|-----------|-------------|-------------|
| `[1]` | 1 | Read access |
| `[2]` | 2 | Write access |
| `[1,1]` | 2 | Read + audit |
| `[2,1]` | 3 | Manage |
| `[2,2]` | 4 | Admin (point class) |

## The Grassmannian as Policy Space

A Grassmannian Gr(k,n) is the space of all k-dimensional subspaces of
n-dimensional space. Think of it as a **policy space** with dimension k(n−k):

- Gr(2,4) has dimension 4 — room for 4 distinct σ₁ conditions
- Gr(3,6) has dimension 9 — room for complex multi-tenant policies
- Gr(4,8) has dimension 16 — enterprise-scale policy space

## How Access Decisions Work

When you call `acl.check(principal, &["read", "write"])`, Schubert:

1. Verifies the principal holds both capabilities
2. Computes the **Schubert intersection** of the capability conditions
3. Returns a quantitative decision:

| Decision | Meaning |
|----------|---------|
| `Granted { configurations: n }` | Access allowed in exactly n ways |
| `Impossible { conflicting }` | Conditions geometrically incompatible |
| `Denied` | Too many conditions (overconstrained) |
| `Underconstrained { dimension }` | Too few conditions (policy too loose) |

## The Killer Feature: Impossibility Detection

Consider σ₂ (write) and σ₁₁ (read+audit) in Gr(2,4). Each is individually
valid, but together they're **geometrically impossible** — no subspace can
simultaneously satisfy both conditions. The Littlewood-Richardson coefficient
is zero.

A traditional boolean AND would approve. Schubert catches the conflict.

```rust
acl.grant(&principal, "write")?;     // σ₂
acl.grant(&principal, "internal")?;  // σ₁₁

let result = acl.check(&principal, &["write", "internal"])?;
// AccessDecision::Impossible { conflicting: ["write", "internal"] }
```

## Trust and Stability

Schubert models trust as a continuous value from 0.0 to 1.0. The
**wall-crossing engine** determines at what trust levels each capability
becomes unstable:

```rust
let report = analyze_stability(&acl, &principal)?;
// report.phase_diagram — breakpoints where stability changes
// report.walls — individual stability walls
```

Higher-codimension capabilities (AdminLike) are more sensitive to trust
degradation than lower ones (ReadLike). The surreal trust extension replaces
f64 with exact surreal arithmetic for infinitesimal trust resolution.

## Composability

Principals can be composed via shared capabilities (operadic composition):

```rust
let result = compose(&acl, &producer, "output", &consumer, "input")?;
// result.multiplicity — how many configurations survive composition
```

This models service chaining: service A produces output that service B consumes.

## Key Mathematical Properties (Verified)

- σ₁⁴ = 2 in Gr(2,4) — four read-like conditions yield exactly 2 configurations
- σ₂·σ₁₁ = 0 — write + internal-audit is geometrically impossible
- Composition is commutative (grant order doesn't matter)
- Grant-revoke identity (grant then revoke = no change)
