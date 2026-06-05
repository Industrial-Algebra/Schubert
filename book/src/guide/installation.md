# Installing and Configuring

## Cargo.toml

```toml
[dependencies]
schubert = "0.1"
```

## Feature Flags

Enable additional features as needed:

```toml
[dependencies]
schubert = { version = "0.1", features = ["serde", "policy", "crypto"] }
```

| Feature | Enables |
|---|---|
| `std` (default) | HashMap, SystemTime, thread-safe audit |
| `serde` | Serialization on all types, JSON I/O |
| `karpal` | Type-level proofs (Proven, Rewrite) |
| `parallel` | Batch operations via rayon |
| `policy` | TOML policy language |
| `wasm` | WebAssembly JS bindings |
| `crypto` | Ed25519 capability tokens |
| `karpal-verify` | Formal verification (SMT/Lean) |
| `surreal` | Exact surreal trust arithmetic |
| `holographic` | Minuet holographic memory |

All features compose — enable any combination.

## `no_std` Support

```toml
[dependencies]
schubert = { version = "0.1", default-features = false }
```

When `std` is disabled: `HashMap` → `BTreeMap`, `InMemoryAudit` is single-threaded,
`AuditSink` trait is unavailable.

## Production Configuration

```toml
[dependencies]
schubert = { version = "0.1", features = [
    "serde",    # Serialization for policy persistence
    "policy",   # TOML policy files
    "crypto",   # Signed capability tokens
    "parallel", # Batch operations
] }
```

For high-assurance systems, add `karpal`, `karpal-verify`, and `surreal`.
