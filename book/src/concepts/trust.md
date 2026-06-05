# Trust and Stability

Schubert models trust as a continuous value from 0.0 to 1.0, and analyzes how
capabilities degrade as trust erodes.

## Continuous Trust

Unlike boolean access control (trusted / not trusted), Schubert supports continuous
trust:

```rust
use schubert::AccessContext;

let ctx = AccessContext {
    resource: Some("customer-data".into()),
    time_budget_ms: Some(500),
    required_trust: 0.85, // 85% trust required
};

acl.check_with_context(&principal, &["read:data"], &ctx)?;
```

## Wall-Crossing Stability

The **wall-crossing engine** (`analyze_stability()`) finds the trust levels where
capabilities become unstable:

```rust
use schubert::analyze_stability;

let report = analyze_stability(&acl, &principal)?;

// report.phase_diagram — breakpoints where stability changes
// report.walls — individual stability walls per capability
// report.most_sensitive — which capability degrades first
```

A **phase diagram** shows how many configurations are available at each trust level.
As trust drops, capabilities cross stability walls — higher-codimension (AdminLike)
capabilities cross first.

## Trust Sensitivity by Kind

| CapabilityKind | Degradation Pattern |
|---|---|
| ReadLike | Degrades below ~0.3 trust |
| WriteLike | Degrades below ~0.5 trust |
| AdminLike | Degrades below ~0.7 trust |

This models the security principle that powerful operations require higher trust.

## Surreal Trust Levels

For applications requiring exact arithmetic on infinitesimal trust differences,
enable the `surreal` feature:

```rust
// Requires: features = ["surreal"]
use schubert::surreal_trust::SurrealTrust;

let trust = SurrealTrust::new(rational_surreal_value);
```

The surreal trust module uses Amari's `RationalSurreal` (v0.23) for exact arithmetic
on trust values, including infinitesimal ε and ε². This enables:

- **Exact comparison** of arbitrarily close trust levels
- **Infinitesimal trust recovery** after temporary degradation
- **Provable trust monotonicity** for formal verification

```rust
// Compare infinitesimal trust levels
let a = SurrealTrust::from_epsilon(1); // ε (infinitesimal)
let b = SurrealTrust::from_epsilon(2); // 2ε (twice infinitesimal)
assert!(a < b);

// Exact trust composition
let combined = a.compose_with(b)?;
```

For a deep dive, see [Surreal Trust Levels](../design/surreal-trust.md).
