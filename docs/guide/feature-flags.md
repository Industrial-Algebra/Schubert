# Feature Flags

Schubert uses additive feature gates — enabling a feature adds functionality
without breaking existing API.

## Available Features

| Feature | Cargo Flag | What It Enables |
|---------|-----------|----------------|
| Standard library | `std` (default) | HashMap, SystemTime, thread-safe Mutex audit |
| Serialization | `serde` | Serialize/Deserialize on all types, JSON I/O |
| Type-level proofs | `karpal` | `proof` module: Proven, Property, Rewrite, law checks |
| Parallel ops | `parallel` | `check_batch()`, `stability_batch()`, `compose_batch()` via rayon |
| Policy language | `policy` | `policy` module: TOML parsing, validate, roundtrip |
| WebAssembly | `wasm` | `wasm` module: WasmController with JS bindings |
| Cryptographic tokens | `crypto` | `crypto` module: Ed25519 CapabilityToken, Issuer, Verifier |
| Formal verification | `karpal-verify` | `verify` module: SMT/Lean proof obligations, Certified trust boundary |
| Surreal trust | `surreal` | `surreal_trust` module: RationalSurreal + EpsilonPolynomial |
| Holographic memory | `holographic` | `holographic` module: Minuet integration |

## Enabling Features

```toml
[dependencies]
schubert = { version = "0.1.0", features = ["serde", "policy", "crypto"] }
```

## Feature Combinations

All features are designed to compose. Common combinations:

```bash
# Production with crypto tokens and policy loading
cargo build --features serde,policy,crypto

# Research with proofs and verification
cargo build --features karpal,karpal-verify,surreal

# Browser with wasm bindings
cargo build --target wasm32-unknown-unknown --features wasm

# Everything (for development)
cargo build --all-features
```

## `no_std` Support

Disable the `std` feature for `no_std` environments:

```toml
[dependencies]
schubert = { version = "0.1.0", default-features = false }
```

When `std` is disabled:
- `HashMap` → `BTreeMap`
- `InMemoryAudit` is single-threaded
- `now_millis()` returns 0
- `AuditSink` trait is unavailable

## Adding Your Own Feature

Feature gates follow the IA convention:

1. Add to `[features]` in `Cargo.toml`
2. Use `#[cfg(feature = "my-feature")]` on module declarations
3. Use `#[cfg(feature = "my-feature")]` on impl blocks and functions
4. Document in this file and `src/lib.rs`
