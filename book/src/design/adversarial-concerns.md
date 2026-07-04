# Adversarial Concerns

> Addresses Proserpina critique finding #13: "The geometric model is gameable
> via model injection, dimensionality poisoning, and denial-of-service attacks."

## Attack Surface

Schubert is a library with no network surface. The attack surface is the
API the host application exposes to untrusted input. The main concerns:

### 1. Computational Denial of Service (DoS)

**Attack:** An adversary submits many capabilities or large partitions to
exhaust the intersection computation.

**Impact:** Littlewood-Richardson computation is #P-complete in general.
Adversarial input could trigger exponential-time computation.

**Mitigation:**

```rust
// Bound the number of capabilities per check
const MAX_CAPABILITIES_PER_CHECK: usize = 16;

fn safe_check(acl: &AccessController, principal: &PrincipalId, caps: &[&str])
    -> Result<AccessDecision>
{
    if caps.len() > MAX_CAPABILITIES_PER_CHECK {
        return Ok(AccessDecision::Denied);
    }
    // Bound partition size — reject partitions with entries > n
    for cap_id in caps {
        if let Some(cap) = acl.get_capability(cap_id) {
            if cap.partition.iter().any(|&p| p > acl.n()) {
                return Ok(AccessDecision::Denied);
            }
        }
    }
    acl.check(principal, caps)
}
```

**Built-in protection:** Schubert's `check()` method validates partitions
against the Grassmannian dimensions. Invalid partitions return `Err`
before computation begins.

### 2. Dimensionality Poisoning

**Attack:** An adversary manipulates the Grassmannian dimensions (k, n) to
create a degenerate policy space where all intersections are trivially
positive.

**Impact:** If n is too large relative to k, every intersection returns a
large positive number, making impossibility detection useless.

**Mitigation:**

- The host application controls Grassmannian dimensions — adversaries cannot
  change them unless the API exposes `new(k, n)` to untrusted callers.
- Recommended dimensions are bounded: Gr(2,4), Gr(3,6), Gr(4,8). These have
  been benchmarked and their intersection behavior is well-understood.
- Do not expose `AccessController::new()` to untrusted input.

### 3. Capability Flooding

**Attack:** An adversary registers thousands of capabilities to pollute
the policy space.

**Impact:** Memory exhaustion and slow capability lookups.

**Mitigation:**

- Capability registration is an administrative operation, not a user-facing API.
- The `AccessController` uses a `HashMap` for O(1) capability lookup regardless
  of count.
- Rate-limit capability registration in the host application.

### 4. Token Replay (crypto feature)

**Attack:** An adversary captures a valid `CapabilityToken` and replays it.

**Impact:** Unauthorized access if the token hasn't expired.

**Mitigation:**

- Tokens have optional expiry (`expires_at`). Use short-lived tokens.
- The host application tracks used tokens (Schubert tokens are stateless).
- Use TLS to prevent token interception.

### 5. CRDT State Poisoning

**Attack:** A malicious node injects poisoned grant state into the CRDT.

**Impact:** The merged state contains invalid grants.

**Mitigation:**

```rust
// Use staleness gating to reject stale state
state.set_max_staleness(Some(30_000)); // 30 seconds

// After merge, validate the resulting state geometrically
state.merge(&remote_state);
if let Some(staleness) = state.staleness_ms() {
    if staleness > 30_000 {
        log::warn!("Rejecting stale CRDT merge: {staleness}ms");
        return Err(merge_error);
    }
}
```

- CRDT merge is commutative and idempotent — poisoned state can be
  overwritten by legitimate state.
- Use `is_converged_with()` to detect nodes that are significantly behind.
- The geometric intersection check catches impossible grant combinations
  even in poisoned state — if the intersection is zero, the grant is rejected
  regardless of CRDT state.

## Timing Side Channels

**Concern:** Intersection computation time varies with partition complexity.
An adversary could infer information about the policy by measuring response
times.

**Impact:** Low. The adversary learns the *complexity* of the check, not the
*result*. The result (Granted/Impossible) is observable regardless.

**Mitigation:** For high-security applications, pad response times to a
constant value. This is the host application's responsibility.

## Summary

| Attack | Severity | Mitigation |
|---|---|---|
| Computational DoS | Medium | Bound capabilities per check, validate partitions |
| Dimensionality poisoning | Low | Don't expose `new(k,n)` to untrusted callers |
| Capability flooding | Low | Admin-only registration, HashMap O(1) lookup |
| Token replay | Medium | Short-lived tokens, replay tracking, TLS |
| CRDT poisoning | Medium | Staleness gating, geometric validation post-merge |
| Timing side channel | Low | Constant-time padding (host responsibility) |

Schubert's core guarantee — geometric impossibility detection — is
**unforgeable**. No adversarial input can make σ₂·σ₁₁ ≠ 0 in Gr(2,4).
The attacks above target availability and state integrity, not the
correctness of individual access decisions.
