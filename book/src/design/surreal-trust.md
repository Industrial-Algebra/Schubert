# Surreal Trust Levels

Exact arithmetic on trust values using Amari's `RationalSurreal`. Enable the `surreal`
feature.

## Motivation

Floating-point trust (f64) has rounding errors that accumulate in chained trust
operations. Surreal numbers provide exact arithmetic with infinitesimal resolution.

## RationalSurreal

A surreal number represented as a rational with infinitesimal extensions:

```rust
use schubert::surreal_trust::SurrealTrust;

// Standard trust values
let full = SurrealTrust::new(RationalSurreal::from_f64(1.0));
let half = SurrealTrust::new(RationalSurreal::from_f64(0.5));

// Exact comparison
assert!(half < full);
```

## EpsilonPolynomial

Infinitesimal trust resolution using ε (epsilon) and its powers:

```rust
use schubert::surreal_trust::EpsilonPolynomial;

let eps = EpsilonPolynomial::epsilon();
let two_eps = EpsilonPolynomial::epsilon() * 2;
let eps_sq = EpsilonPolynomial::epsilon_squared();

// ε² < ε < 1 (infinitesimal ordering)
assert!(eps_sq < eps);
assert!(eps < 1.0);

// Compare infinitesimal trust levels
assert!(SurrealTrust::from_epsilon(1) < SurrealTrust::from_epsilon(2));
```

## Use Cases

- **Exact trust comparison**: Arbitrarily close trust levels are distinct
- **Infinitesimal recovery**: Gradual trust restoration in ε increments
- **Provable monotonicity**: Trust never spontaneously increases
- **Compositional trust**: Exact trust arithmetic across service chains

## Comparison

`EpsilonPolynomial` lacks `PartialOrd` — use `compare_infinitesimal()`:

```rust
use schubert::surreal_trust::compare_infinitesimal;

let a = EpsilonPolynomial::epsilon_squared();
let b = EpsilonPolynomial::epsilon();

assert_eq!(compare_infinitesimal(&a, &b), std::cmp::Ordering::Less);
```

Comparison first checks valuations (degree of smallest ε term), then compares
coefficients.
