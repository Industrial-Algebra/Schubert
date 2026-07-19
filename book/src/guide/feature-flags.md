# Feature Flags

Schubert uses additive feature gates — enabling a feature adds functionality without
breaking existing API.

## Available Features

| Feature | What It Enables |
|---|---|
| `std` (default) | HashMap, SystemTime, thread-safe Mutex audit |
| `serde` | Serialize/Deserialize on all types, JSON I/O |
| `karpal` | `proof` module: Proven, Property, Rewrite, law checks |
| `parallel` | `check_batch()`, `stability_batch()`, `compose_batch()` via rayon |
| `policy` | `policy` module: TOML parsing, validate, roundtrip |
| `wasm` | `wasm` module: WasmController with JS bindings |
| `crypto` | `crypto` module: Ed25519 CapabilityToken, GrantToken, Issuer, Verifier, KeyStore |
| `axum` | `axum` module: AuthPrincipal bearer-token extractor (enables `crypto`) |
| `karpal-verify` | `verify` module: SMT/Lean proof obligations, Certified trust boundary |
| `surreal` | `surreal_trust` module: RationalSurreal + EpsilonPolynomial |
| `holographic` | `holographic` module: Minuet integration |

## Common Combinations

```bash
# Production with crypto tokens and policy loading
cargo build --features serde,policy,crypto

# Web service with bearer-token auth
cargo build --features axum

# Research with proofs and verification
cargo build --features karpal,karpal-verify,surreal

# Browser with wasm bindings
cargo build --target wasm32-unknown-unknown --features wasm

# Everything (for development)
cargo build --all-features
```

All features are designed to compose freely. Enable only what you need —
each feature adds compile time and binary size.

## Adding Your Own Feature

Feature gates follow the IA ecosystem convention:

1. Add to `[features]` in `Cargo.toml`
2. Use `#[cfg(feature = "my-feature")]` on module declarations
3. Use `#[cfg(feature = "my-feature")]` on impl blocks and functions
4. Document in `src/lib.rs`
