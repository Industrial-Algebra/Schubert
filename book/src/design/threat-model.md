# Threat Model

> Addresses Proserpina critique finding #4: "No threat model, adversary model,
> or security properties are specified."

## Assumed Adversary Capabilities

Schubert is a **library**, not a network service. The threat model assumes:

### What the Adversary CAN Do

| Capability | Mitigation |
|---|---|
| **Observe access decisions** | Schubert is embedded; the host application controls logging via `AuditSink` |
| **Attempt impossible capability combinations** | Schubert returns `AccessDecision::Impossible` — the geometric intersection is zero regardless of adversary input |
| **Submit many capabilities to exhaust computation** | See [Adversarial Concerns](adversarial-concerns.md) — bounded intersection depth, input validation |
| **Forge capability tokens** (if `crypto` feature enabled) | Ed25519 signatures — forging requires the issuer's private key |

### What the Adversary CANNOT Do

| Capability | Why |
|---|---|
| **Bypass the geometric intersection** | The Littlewood-Richardson coefficient is a mathematical fact. No input can make σ₂·σ₁₁ ≠ 0 in Gr(2,4). |
| **Corrupt the computation silently** | The result is deterministic given the same Grassmannian and conditions. |
| **Forge a `Certified<T>` value** (if `karpal-verify` feature enabled) | The `Proven` type requires a valid proof obligation. |

### What Schubert Does NOT Protect Against

| Threat | Owner |
|---|---|
| **Authentication** | External (OAuth, JWT, mTLS). Schubert receives `PrincipalId` from the caller. |
| **Transport security** | External (TLS, WireGuard). Schubert has no network surface. |
| **Key management** | External. The `CapabilityIssuer` generates keys; storage and rotation are the caller's responsibility. |
| **Replay attacks on tokens** | External. Schubert tokens are stateless. The caller tracks used tokens. |
| **Side-channel timing** | Partial. Intersection computation time varies with Grassmannian size. See adversarial concerns. |

## Security Properties

### Property 1: Geometric Impossibility Is Unforgeable

**Claim:** If `check(principal, &[cap_a, cap_b])` returns `Impossible`, then no
input can make it return `Granted` for the same Grassmannian and conditions.

**Justification:** The Littlewood-Richardson coefficient is a topological
invariant of the Grassmannian. It is determined entirely by the partitions
and the Grassmannian dimensions. No runtime input, adversary action, or
state mutation can change a coefficient from 0 to positive.

### Property 2: Grant Commutativity

**Claim:** Granting capabilities in any order produces the same access decision.

**Justification:** Schubert intersection is commutative — σ_a · σ_b = σ_b · σ_a.
The order of grants does not affect the geometric result.

**Limitation:** This is a *semantic* property about the set of held
capabilities, not a *temporal* property. Audit logs must still record the
order of grants for forensics. See [Audit Trail](#audit-trail) below.

### Property 3: Grant-Revoke Identity

**Claim:** Granting then revoking a capability produces the same state as
never granting it (assuming no prior grant).

**Justification:** The grant adds the Schubert condition; the revoke removes
it. The net geometric effect is identity.

**Limitation:** If the capability was already held, revoke produces a
different state. Temporal audit logs are still needed.

## Audit Trail

> Addresses critique finding #21: "Commutativity of grant order eliminates
> temporal auditing."

The geometric commutativity of grants (σ_a · σ_b = σ_b · σ_a) means the
*final access decision* is order-independent. But **temporal ordering
matters for audit and forensics**:

- **Who granted what, when?** — The `AuditSink` records every grant/revoke
  with timestamps.
- **Replay for investigation** — Audit records allow reconstructing the
  sequence of grants leading to an access event.
- **Regulatory compliance** — GDPR, HIPAA, SOC 2 require temporal audit
  trails regardless of semantic commutativity.

**Design principle:** Schubert's geometry determines *what* access is
possible. The audit trail determines *how it came to be*. These are
orthogonal concerns — commutativity in the former does not eliminate the
need for the latter.

## Non-Identifiability

> Addresses critique finding #20: "Positive intersection numbers do not
> identify which users have access."

The intersection number counts *how many configurations* satisfy the
conditions. It does not enumerate *which principals* hold the capabilities.

**This is by design:**

- The intersection number is a property of the **Schubert conditions**
  (the policy), not the **principal set** (the population).
- To check whether a *specific* principal has access, call `check(principal, ...)`.
- The intersection number tells you the policy's *capacity* — how many
  valid ways exist to satisfy it — not the *occupancy*.

**Analogy:** A building has a capacity of 100 (intersection number). To check
whether Alice is inside, you look at the roster (`check(alice, ...)`). The
capacity doesn't tell you who's inside; the roster does.
