# Schubert — Karpal Verification Integration

**Date:** 2026-05-19
**Status:** Initial integration design — Phase 15C + Phase 17
**Dependencies:** `karpal-proof`, `karpal-proof-derive`, `karpal-verify` (Phase 12), `amari-enumerative`

---

## 1. Architecture

Schubert's access control is built on amari-enumerative's Schubert calculus:
partitions, Grassmannians, Littlewood-Richardson coefficients, intersection
numbers. Every access decision is a Schubert intersection computation. This
is the mathematical core of Industrial Algebra — and it needs verification
at every level.

```
amari-enumerative (Schubert calculus)
    ↓ proves mathematical correctness
karpal-verify (Obligation IR → SMT/Lean)
    ↓ exports proof certificates
karpal-proof (Proven<P,T>, Certified<B,P,T>)
    ↓ carries type-level evidence
schubert::proof (Capability::prove, AccessDecision::prove_finite)
    ↓ integrates into access control API
Schubert user code
```

---

## 2. What's Already Verified

### 2.1 Type-level proofs (karpal-proof)

Schubert's `proof.rs` module already integrates `karpal-proof` v0.3.0 with
the `karpal` feature flag:

| Property | What it proves | Constructor |
|---|---|---|
| `IsValidCapability` | Partition is valid for Gr(k,n) | `Capability::prove(grassmannian)` |
| `IsHeldBy` | Principal holds a capability | runtime law check |
| `IsFiniteAccess` | Access decision is finite | `AccessDecision::prove_finite()` |
| `IsComposableWith` | Two principals are composable | runtime law check |
| `IsAdminLike` | Capability is admin-level | unsafe axiom (trust boundary) |
| `IsWriteLike` | Capability is write-level | derived from IsAdminLike |
| `IsReadLike` | Capability is read-level | derived from IsWriteLike |

### 2.2 Property hierarchy

```text
IsAdminLike ──→ IsWriteLike ──→ IsReadLike ──→ IsValidCapability
```

Using Karpal's `Implies` trait, any capability kind higher in the hierarchy
automatically proves lower ones. An `Proven<IsAdminLike, Capability>` can be
derived as `Proven<IsReadLike, Capability>` through the chain.

### 2.3 Runtime law verification

```rust
// Grant idempotency
law::check_grant_idempotency(&mut acl, &principal_id, "read:data")?;

// Grant-revoke identity
law::check_grant_revoke_identity(&mut acl, &principal_id, "read:data")?;

// Access check idempotency
law::check_access_idempotency(&acl, &principal_id, &["read:data"])?;
```

### 2.4 Rewrite rules

```rust
// Grant order is commutative
let _: Rewrite<GrantSeqAB, GrantSeqBA, ByGrantCommutativity> = Rewrite::witness();

// Grant-then-revoke = identity
let _: Rewrite<GrantThenRevoke, NoOp, ByGrantRevokeIdentity> = Rewrite::witness();

// Access check is symmetric in requirements
let _: Rewrite<CheckAB, CheckBA, ByCheckCommutativity> = Rewrite::witness();
```

---

## 3. Integration Plan — Phase 15C (Schubert Calculus Verification)

### 3.1 Obstacle: Current Trust Boundary

Today, `Capability::prove()` validates the partition at runtime and wraps
the result in `Proven<IsValidCapability, Capability>`:

```rust
pub fn prove(self, grassmannian: (usize, usize)) -> Result<Proven<IsValidCapability, Self>> {
    self.to_schubert_class(grassmannian)?;
    Ok(unsafe { Proven::axiom(self) })  // ← trust-me boundary
}
```

The `unsafe { Proven::axiom(self) }` trusts that `to_schubert_class` correctly
implements the Schubert calculus. But `to_schubert_class` delegates to
`amari-enumerative`, which has no formal verification of its Littlewood-Richardson
computation. The trust boundary is transitive — we trust Schubert's validation,
which trusts amari's math, which is unverified.

**Phase 15C bridges this gap** by generating external proof obligations for
amari-enumerative's operations and importing the results as `Certificate`
values through karpal-verify's trust boundary.

### 3.2 Obligation Bundles

| Bundle name | What it verifies | Backend | Target |
|---|---|---|---|
| `schubert_lr_gr24` | LR coefficients for Gr(2,4) match known values | SMT (Z3) | 6 Schubert classes, exhaustive |
| `schubert_lr_gr36` | LR coefficients for Gr(3,6) match known values | SMT (Z3) | 20 Schubert classes, exhaustive |
| `schubert_lr_gr48` | LR coefficients for Gr(4,8) match known values | SMT (Z3) | ~70 classes, sampled |
| `schubert_partition_validity` | All partitions up to box bound pass validation | SMT (Z3) | Exhaustive for small k,n |
| `schubert_intersection_emptiness` | σ_λ·σ_μ = 0 when codim > dim | Lean 4 | Requires dimension argument |
| `schubert_lr_associativity` | (σ_α·σ_β)·σ_γ = σ_α·(σ_β·σ_γ) | Lean 4 | Requires induction |
| `schubert_intersection_commutativity` | σ_λ·σ_μ = σ_μ·σ_λ | Lean 4 | Structural |
| `schubert_dimension_formula` | dim(σ_λ) = k(n−k) − |λ| | Lean 4 | Formula proof |
| `schubert_wall_crossing` | Stability thresholds match analytics | amari-flynn | Statistical (ε=0.01) |
| `schubert_giambelli` | σ_λ as determinant of special classes | Lean 4 | Schubert polynomial identity |

### 3.3 Integration Code Sketch

```rust
// schubert/src/verify.rs (new module, gated behind karpal-verify feature)

use karpal_verify::{
    AlgebraicSignature, ObligationBundle, Origin, Sort,
    VerificationSession, ArtifactLayout, LeanConfig, SmtConfig, DryRunner,
};

pub fn schubert_lr_obligations() -> ObligationBundle {
    let sig = AlgebraicSignature::new(Sort::Named("SchubertClass"))
        .with_binary("intersect", "lr_product");

    ObligationBundle::new(
        "schubert_lr_consistency",
        Origin::new("amari-enumerative", "LittlewoodRichardson"),
    )
    .with(Obligation::associativity_in(
        "lr_associativity",
        Origin::new("amari-enumerative", "LR rule"),
        &sig,
        "intersect",
    ))
    .with(Obligation::commutativity_in(
        "lr_commutativity",
        Origin::new("amari-enumerative", "LR rule"),
        &sig,
        "intersect",
    ))
}

#[test]
fn verify_schubert_lr_consistency() {
    let bundle = schubert_lr_obligations();
    let report = karpal_verify::verify_bundle(
        &bundle,
        &ArtifactLayout::new("target/karpal-verify/schubert"),
        "SchubertVerify",
        &SmtConfig::default(),
        &LeanConfig::default(),
        &DryRunner,
    ).expect("LR verification session should succeed");

    assert!(
        report.all_passed(),
        "Schubert LR verification failed: {}",
        report.failure_summary()
    );
}
```

### 3.4 Trust Boundary Upgrade

After Phase 15C, `Capability::prove()` crosses TWO trust boundaries:

```rust
pub fn prove(self, grassmannian: (usize, usize))
    -> Result<Certified<LeanCertificate, IsValidCapability, Self>>
{
    // 1. Runtime validation (existing)
    self.to_schubert_class(grassmannian)?;

    // 2. Load externally verified certificate
    let cert = Certificate::load("target/karpal-verify/schubert/lr_consistency.cert")?;

    // 3. Cross the trust boundary explicitly
    Ok(unsafe { Certified::<LeanCertificate, IsValidCapability, Self>::assume(self, cert) })
}
```

The `unsafe` is now auditable: it references a specific certificate file
produced by Lean 4 verification, with a git-tracked golden file. A reviewer
can trace from `prove()` → certificate → Lean theorem → Schubert calculus
specification.

---

## 4. CI Integration

### 4.1 Workflow

```yaml
# .github/workflows/schubert-verify.yml
name: Schubert Verification

on:
  push:
    paths:
      - 'amari/amari-enumerative/**'
      - 'Schubert/**'
  pull_request:

jobs:
  # Tier 1: Runtime property tests (fast, per-PR)
  proptest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test -p schubert --features karpal

  # Tier 2: SMT verification (medium, per-PR)
  smt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get install -y z3
      - run: cargo test -p schubert --features karpal-verify,smt -- --ignored

  # Tier 3: Lean verification (slow, label-gated)
  lean:
    if: contains(github.event.pull_request.labels.*.name, 'run-lean-verify')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: leanprover/lean-action@v1
      - run: cargo test -p schubert --features karpal-verify,lean -- --ignored
```

### 4.2 Golden Files

SMT-LIB2 and Lean 4 export output is checked into the repository:

```
Schubert/
  tests/
    golden/
      schubert_lr_gr24.smt2        # SMT-LIB2 for Gr(2,4) LR
      SchubertVerify.lean           # Lean 4 module for LR theorems
      schubert_certificates.json    # Certificate metadata
```

Golden file regeneration: `cargo test -p schubert --features karpal-verify -- --ignored --test export_golden`

---

## 5. Migration Path

### Phase 1 (now) — karpal-proof-derive integration
- `#[derive(VerifySemigroup)]` on `Principal` and `Capability` types
- Proptest harnesses generated automatically for access control laws
- No new dependencies beyond `karpal-proof-derive`

### Phase 2 (Phase 15C complete) — SMT verification
- `schubert_lr_gr24` SMT-LIB2 export verified against Z3
- `Capability::prove()` upgraded to accept external `Certificate`
- Golden files committed to repository

### Phase 3 (Phase 15C complete) — Lean 4 bridge
- Schubert calculus axioms formalized in Lean 4
- LR associativity and commutativity proven
- `Certified<LeanCertificate, IsValidCapability, Capability>` used in production path

### Phase 4 (Phase 17) — ecosystem-wide
- Schubert verification results referenced by ShaperOS, Minoru, Baedeker
- Karpal certificate registry for cross-crate proof composition

---

## 6. Verification Coverage Matrix

| What | Proptest | SMT | Lean 4 | amari-flynn | Status |
|---|---|---|---|---|---|
| Capability partition validity | ✅ | ✅ | — | — | Phase 1 |
| Access decision idempotency | ✅ | — | ✅ | — | Phase 1 |
| Grant commutativity | ✅ | — | ✅ | — | Phase 1 |
| Grant-revoke identity | ✅ | — | ✅ | — | Phase 1 |
| LR coefficient consistency | — | ✅ | — | — | Phase 2 |
| LR associativity | — | — | ✅ | — | Phase 3 |
| Schubert dimension formula | — | — | ✅ | — | Phase 3 |
| Intersection emptiness | — | ✅ | ✅ | — | Phase 2-3 |
| Wall-crossing stability | — | — | — | ✅ | Phase 3 |
| Giambelli determinant identity | — | — | ✅ | — | Phase 3 |

## 7. References

- [Karpal Roadmap](../../karpal/ROADMAP.md) — Phase 12 (verification backends), Phase 15 (Schubert types), Phase 17 (ecosystem integration)
- [Borsalino verification integration](../../Borsalino/docs/verification-integration.md) — GPU compute verification
- [Schubert Karpal Integration (May 14, 2026)](../../karpal/HANDOFF.md) — Initial karpal-proof integration notes
- [amari-flynn documentation](https://github.com/Industrial-Algebra/amari-flynn) — Statistical verification backend
- Fulton, *Young Tableaux* — Schubert calculus foundations
