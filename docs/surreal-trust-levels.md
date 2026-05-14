# Surreal Trust Levels — An Expansion

*Expanding Direction 12 from the Schubert ROADMAP*

---

## Current State

### Amari 0.22.0 (Released)
The `amari-surreal` crate provides:
- **`Dyadic`** — exact rationals of the form `m / 2^n`
- **`ShortSurreal`** — validated surreal numbers over dyadics, with game-to-surreal conversion via `amari-cgt`, simplest-number construction for finite cuts, and value-based equality (two surreals are equal if their numeric values match, regardless of birthday)

### Amari 0.23.0 (In Progress — Worktree Active)
A significant extension is underway:

- **`RationalSurreal`** — exact rational scalar field backed by `BigRational`. Bridged from `Dyadic` and `ShortSurreal`. Full arithmetic including division. This is the critical piece: where v0.22 surreals can only represent `m/2^n`, v0.23 surreals can represent **any rational number** as a surreal scalar.

- **`amari-surcomplex`** (new crate) — exact complex numbers `a + bi` over `RationalSurreal`. The motivation is algebraic closure for surreal division: `1/(1 + i/2) = 4/5 - 2i/5` requires non-dyadic coefficients that only `RationalSurreal` can provide. Provides conjugate, norm, reciprocal, and full division.

- **`EpsilonPolynomial` / `EpsilonRational`** (feature-gated) — polynomials and rational functions in a formal positive infinitesimal `ε`, with asymptotic ordering as `ε → 0⁺`. This is **not** nilpotent dual numbers (`ε² ≠ 0`). Instead, `ε²` is a positive infinitesimal smaller than `ε`. The ordering follows the asymptotic hierarchy: terms with smaller exponents dominate.

### Key Design Decisions

The roadmap makes explicit choices that shape what surreal trust levels can become:

1. **`RationalSurreal` is separate from `ShortSurreal`** — keeps the dyadic API clean while providing the broader rational field needed for surcomplex division.

2. **`amari-surcomplex` is a separate crate** — prevents complex APIs from cluttering the surreal core, follows established Amari patterns.

3. **Epsilon is not nilpotent** — `ε²` is a distinct positive infinitesimal, not zero. This is crucial for surreal trust: you get a proper infinitesimal hierarchy, not just a single flat dual number.

---

## What Surreal Trust Levels Enable

### The Trust Level Lattice

Currently, Schubert trust levels are `f64` clamped to `[0.0, 1.0]`. This is a **single dimension** with no internal structure. Surreal trust levels replace this with a **totally ordered field** that contains:

| Layer | Example | Meaning |
|-------|---------|---------|
| Finite real | `0.5` | Standard trust |
| Dyadic rational | `1/2^k` | Finer than floating-point |
| General rational | `3/7` | Exact, no floating-point artifacts |
| Infinitesimal | `ε`, `ε²` | "Trust but barely" — distinct levels within the infinitesimal |
| Mixed | `0.5 + ε` | "Half trust plus a shred" |

### Distinguishing Trust at the Infinitesimal Level

With `f64` trust, two "almost zero" trust levels are indistinguishable — `0.0000001` and `0.00000000001` both round to the same behavior. With surreal trust:

```rust
// Hypothetical API
let t1 = SurrealTrust::infinitesimal(1);      // ε    — barely trust
let t2 = SurrealTrust::infinitesimal(2);      // ε²   — barely trust, but smaller
let t3 = SurrealTrust::finite(0.5) + SurrealTrust::infinitesimal(1);  // 0.5 + ε

// t1 > t2 > 0  (true: ε > ε² > 0)
// t3 > 0.5     (true: adding an infinitesimal makes it slightly larger)
// t3 < 0.5 + ε²? (false: ε > ε², so 0.5+ε > 0.5+ε²)
```

This matters for access control because it lets you express nuanced policies:

- **ε trust**: "I've verified your identity but nothing else — you get exactly one read access to public data"
- **ε² trust**: "I've never seen you before — you can observe that data exists but not read it"
- **0.5 + ε trust**: "You're a known user with some history — slightly more than half trust"

### Wall-Crossing with Infinitesimal Resolution

The wall-crossing engine currently computes stability walls at rational `f64` thresholds. With surreal trust, walls can exist at **every infinitesimal level**:

```rust
// Hypothetical: a capability that becomes unstable at trust ε
// This means it's stable for all finite trust > 0, but unstable at ε
// No f64 representation can express this distinction.

let cap = SurrealCapability::new(
    "read:public_detail",
    EpsilonPolynomial::from_scalar(RationalSurreal::from_ratio(1, 2)) // σ_1/2?
);
```

The phase diagram becomes a **surrealvalued function**: for each trust level `t` in the surreal field, it returns a count of stable capabilities. The breakpoints in this diagram include infinitesimal thresholds that standard floating-point cannot represent.

### Surrealvalued Access Decisions

Currently, `AccessDecision::Granted { configurations: u64 }` returns a natural number count. With surcomplex integration, the enumeration result could carry additional geometric information — the **complex phase** of the configuration count, encoding not just "how many" but also geometric relationships between configurations.

---

## Integration Path

### Phase 1: Surreal Trust (Schubert 0.2.0)
- Depend on `amari-surreal` (with `RationalSurreal`)
- Add `SurrealTrust` type wrapping `RationalSurreal`
- Implement `Ord`, ordering arithmetic
- Update `StabilityCondition` to accept surreal trust
- The wall-crossing engine naturally generalizes — the phase φ(t) formula is analytic in t

### Phase 2: Infinitesimal Policies (Schubert 0.3.0)
- Depend on `amari-surreal` with `experimental-epsilon` feature
- `TrustLevel::infinitesimal(order: i32)` for ε, ε², ...
- `TrustLevel::mixed(finite: RationalSurreal, infinitesimal: i32)` for 0.5+ε, etc.
- Policy-level infinitesimal distinction: ε-capabilities are "fragile" — they're the first to go under any trust degradation

### Phase 3: Surcomplex Configuration Counting (Schubert 0.4.0+)
- Depend on `amari-surcomplex`
- `AccessDecision` carries complex-weighted configurations: `SurcomplexGranted { configurations: RationalSurcomplex }`
- The geometric phase encodes relationships between valid configurations
- Entanglement-style correlations: two capabilities whose intersection multiplies to a complex phase indicate correlated access patterns

### Phase 4: Surreal Geometry (Amari 0.24+ / Future)
- As the surcomplex crate matures, algebraic varieties over the surreal field become computable
- Schubert access control over surreal varieties — configurations are points in a surreal Grassmannian
- Trust levels and access spaces are the **same type** — the surreal numbers unify the value domain and the geometry

---

## Design Constraints Preserved

1. **Exact arithmetic throughout.** No floating-point trust levels. Every decision is rational-surreal-exact.

2. **Backward compatible.** `TrustLevel::new(0.5)` works unchanged. The surreal types are additive, not replacement.

3. **Feature-gated.** Epsilon and surcomplex are behind Cargo features. The core surreal trust works with just `RationalSurreal`.

4. **No runtime cost for basic use.** `f64` trust remains available. Surreal trust is opt-in for policies that need infinitesimal resolution.

---

## Open Questions

1. **Birthday metadata:** Should surreal trust levels carry Conway birthdays (construction complexity) as metadata? A trust level `0.5` constructed on day ω has different structural properties than one constructed on day 2, even though their numeric values are equal.

2. **Surcomplex semantics:** What does it mean for a configuration count to be complex-valued? In quantum mechanics, the phase encodes interference. In access control, it might encode correlated policy effects — two capabilities whose joint grant produces constructive or destructive interference in the access space.

3. **Epsilon convergence:** If ε-policies are "fragile" and ε²-policies are "more fragile", is there a principled limit? What about ε^ω — an infinitesimal that's smaller than any finite power of ε? This is where the epsilon polynomial approach meets the full surreal hierarchy.

4. **Persistence in audit logs:** If trust levels are surreal-valued, do audit logs store the full surreal representation or only the rational approximation? Storing the full representation preserves the policy intent but may be verbose for high-order epsilon terms.

---

*May 2026 — based on Amari v0.22.0 (released) and v0.23.0 worktree (active development)*
