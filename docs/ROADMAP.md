# Schubert — Directions

**Version:** 0.1.0 — Foundation complete. Speculative directions below.

---

## Current State

Schubert provides a practical access control library built on Schubert calculus. It is embeddable, synchronous, and depends only on `amari-enumerative`. The core API — `AccessController`, `Capability`, `Principal`, `AccessDecision`, operadic composition, stability analysis — is implemented and tested.

What follows are speculative directions. Some are near-term and practical. Others are research-grade and require mathematical or engineering advances. All are genuine possibilities opened by the geometric foundation.

---

## Near-Term (Practical)

### 1. Computation Path Selection

**Current:** The auto-router always uses Littlewood-Richardson, which is exponential in the number of classes. The `check_with_path` method accepts a path preference but doesn't route.

**Direction:** Expose amari's 4 computation paths — LR (exact), equivariant localization (Atiyah-Bott fixed-point), tropical intersection, matroid independence — with automatic selection based on problem characteristics:

- Small Grassmannians, few classes → LR
- Large Grassmannians, many classes → localization
- Degenerate intersections → CSM correction
- Polynomial-time shortcut → matroid independence check

This would make Schubert practical for enterprise-scale Grassmannians (Gr(8,16), Gr(16,32)) where naive LR is intractable.

### 2. Serialization and Persistence

**Current:** In-memory only. Principals and capabilities are transient.

**Direction:** Add `serde` support behind a feature flag. Serialize `AccessController` state to JSON, MessagePack, or a custom binary format. Enable:

- Save/restore access control state across restarts
- Export policy configurations for auditing
- Transmit capability grants between services

### 3. Policy Language

**Current:** Capabilities are defined programmatically in Rust. No declarative specification.

**Direction:** A simple policy language (TOML, YAML, or a custom DSL) for defining capabilities, principals, and grants:

```toml
[grassmannian]
k = 2
n = 4

[capabilities.read_data]
partition = [1]
kind = "ReadLike"
label = "Read data"

[principals.alice]
grants = ["read_data", "write_data"]
```

Parsed at startup. Validated against the Grassmannian. This enables policy-as-code with geometric guarantees.

### 4. WebAssembly Target

**Current:** Depends on `amari-enumerative`, which is not wasm-ready on all paths.

**Direction:** A `wasm` feature that compiles to WebAssembly. Embed Schubert access control in browser-based applications, edge functions, and serverless environments. Enables client-side policy validation with the same geometric guarantees.

### 5. Context-Aware Decisions

**Current:** `check()` only considers capabilities. No notion of resource, time, or environment.

**Direction:** Add an optional `AccessContext` to checks:

```rust
pub struct AccessContext {
    pub resource: Option<String>,
    pub time: Option<u64>,
    pub metadata: HashMap<String, String>,
}
```

The context feeds into the stability engine — certain capabilities may be conditionally stable based on environmental factors.

---

## Medium-Term (Research-Adjacent)

### 6. Multi-Grassmannian Controllers

**Current:** One `AccessController` = one Grassmannian Gr(k,n). Cross-domain access requires multiple controllers.

**Direction:** A `MultiController` that manages multiple Grassmannians with cross-domain capability translation. A principal in Gr(2,4) accessing a resource in Gr(3,6) requires a morphism between Grassmannians — the Schubert calculus of flag varieties provides this.

### 7. Proof-Carrying Capabilities

**Current:** Capabilities are granted by the controller. No cryptographic verification.

**Direction:** Capabilities as cryptographic tokens. A principal presents a signed capability, the controller verifies the signature and checks the Schubert intersection. This enables distributed access control where the capability issuer and the verifier are separate services, unified by the geometric computation.

### 8. Temporal Access Control

**Current:** Capabilities are permanent until revoked.

**Direction:** Timed capabilities with automatic expiry. The stability engine naturally models this — a capability with expiry time T has a trust level that decays to zero at T, crossing walls at predictable intervals. The phase diagram becomes a calendar.

### 9. Quantitative Rate Limiting

**Current:** Access is binary-geometric. No notion of rate.

**Direction:** Blend Schubert intersection with token-bucket rate limiting. The intersection number determines the bucket capacity — access with 2 configurations gets 2× the rate of access with 1. The geometry of access maps to the geometry of throughput.

---

## Far-Term (Speculative)

### 10. Schubert Routing

Access decisions as routing rules. A network where route advertisement = capability grants and forwarding = Schubert intersection. The number of valid routes between source and destination is the intersection number. Congestion is codimension excess. This is the networking model explored in the ShaperOS transport layer, extracted as a standalone protocol.

### 11. Access Control for Holographic Memory

Integration with Minuet-style holographic memory systems. Capabilities are binding vectors in a holographic reduced representation. Access is granted when the query vector's similarity to the capability vector exceeds the trust threshold. The wall-crossing engine determines which memories are accessible at each trust level.

### 12. Surreal Trust Levels

When `amari-surreal` supports generalized surreal numbers beyond the dyadic layer, trust levels could be surreal-valued — enabling infinite descending chains of trust degradation. A capability that becomes unstable at trust level ε (infinitesimal) remains stable at all finite trust levels but can be distinguished from one that becomes unstable at ε².

### 13. Constitutional Verification

When formal verification tooling (karpal-proof, karpal-verify) matures, capability partitions and access decisions become machine-checkable. A principal cannot be granted capabilities whose Schubert intersection exceeds specified bounds. The access controller's correctness is proved against the Schubert calculus axioms.

### 14. Distributed Access Control with CRDTs

Operadic composition over a distributed system using conflict-free replicated data types. Principals hold vector clocks. Capability grants merge via geometric CRDT operations. The intersection number is computed from eventually-consistent state. This requires the Cliffy protocols geometric CRDT layer.

---

## Design Principles (Preserved Across All Directions)

1. **No network service.** Schubert remains a library. Deployment patterns (daemon, sidecar, plugin) are the caller's choice.

2. **Identity is external.** Schubert never authenticates. It authorizes against identities provided by the caller.

3. **Mathematics is the authority.** Every access decision has a geometric basis. No heuristic rules, no pattern matching, no ML models.

4. **Synchronous by default.** The core API is synchronous. Async wrappers can be built externally.

5. **Embeddable.** A single dependency. Compiles anywhere Rust compiles. Suitable for kernels, embedded systems, browsers.

---

*Schubert 0.1.0 — May 2026*
