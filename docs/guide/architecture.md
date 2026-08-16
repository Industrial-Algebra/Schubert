# Architecture

Schubert is organized into 18 source modules. Here's how they fit together.

## Module Map

```
schubert/
├── AccessController          # Main entry point — all access checks go through here
│   ├── capability.rs         # Capability, CapabilityId, CapabilityKind
│   ├── principal.rs          # Principal, PrincipalId (wraps amari Namespace)
│   ├── decision.rs           # AccessDecision, ComputationPath, AccessContext
│   └── error.rs              # SchubertError (11 variants)
│
├── Access Control Extensions
│   ├── composition.rs        # Operadic composition (compose principals)
│   ├── stability.rs          # Wall-crossing stability analysis
│   ├── audit.rs              # Pluggable audit sink (AuditSink trait)
│   └── rate_limit.rs         # Token-bucket rate limiting
│
├── Feature-Gated Modules
│   ├── proof.rs (karpal)     # Compile-time verification (Proven, Rewrite)
│   ├── policy.rs (policy)    # TOML policy language
│   ├── crypto.rs (crypto)    # Ed25519 capability tokens
│   ├── verify.rs (karpal-verify)  # SMT/Lean proof obligations
│   ├── surreal_trust.rs (surreal) # Exact surreal trust levels
│   ├── wasm.rs (wasm)        # wasm-bindgen JS bindings
│   ├── holographic.rs (holographic) # Minuet holographic memory
│   └── routing.rs            # Geometric network routing
│
├── Multi-Domain
│   ├── multi.rs              # MultiController (cross-domain access)
│   └── crdt.rs               # Eventually-consistent CRDT grants
│
└── Infrastructure
    ├── phantom.rs            # Re-exports amari phantom types
    └── lib.rs                # Re-exports, feature docs
```

## Data Flow

```
Principal + Capabilities → AccessController::check()
    ↓
SchubertCalculus::multi_intersect()  [amari-enumerative]
    ↓
AccessDecision { Granted{n}, Impossible, Denied, Underconstrained }
    ↓
AuditSink::record()  [optional]
```

## Dependency Graph

```
schubert
├── amari-enumerative v0.23    (Schubert calculus engine)
│   └── amari-core             (multivector, geometric algebra)
├── thiserror                  (error derive macros)
├── [optional] serde + serde_json  (serialization)
├── [optional] karpal-proof v0.5   (type-level proofs)
├── [optional] karpal-verify v0.5  (SMT/Lean verification)
├── [optional] karpal-schubert-types v0.5 (Schubert types for Karpal)
├── [optional] rayon           (parallel batch operations)
├── [optional] toml            (policy language)
├── [optional] ed25519-dalek   (cryptographic tokens)
├── [optional] amari-surreal   (surreal trust)
├── [optional] minuet          (holographic memory)
├── [optional] wasm-bindgen + js-sys (WebAssembly)
```

## Key Design Decisions

1. **Identity is external** — `PrincipalId` is an opaque string. Schubert never authenticates.
2. **No dual storage** — `Principal` wraps amari `Namespace` directly.
3. **Audit is fire-and-forget** — failing sinks never block access decisions.
4. **Feature gates are additive** — never break existing API.
5. **No unsafe code** — pure safe Rust (one exception: `Certified` trust boundary).
6. **Synchronous by default** — async wrappers can be built externally.
