# Mathematical Foundation

Schubert uses **Schubert calculus** — a branch of algebraic geometry — to make access
control decisions. You don't need to be a mathematician, but understanding the core
concepts helps design better policies.

## The Grassmannian as Policy Space

A **Grassmannian** Gr(k,n) is the space of all k-dimensional subspaces of an
n-dimensional vector space. In Schubert, we use it as the **policy space** —
each point represents a possible access configuration.

The dimension of Gr(k,n) is k(n−k). This is the maximum number of independent
Schubert conditions you can impose before the space collapses:

| Gr(k,n) | Dimension | Max Independent Conditions |
|---|---|---|
| Gr(2,4) | 4 | 4 |
| Gr(3,6) | 9 | 9 |
| Gr(4,8) | 16 | 16 |

## Schubert Conditions

A **Schubert condition** is a geometric constraint defined by a **partition** — a
weakly decreasing sequence of integers like `[1]`, `[2,1]`, or `[2,2]`. Each partition
corresponds to a specific subspace constraint.

The **codimension** of a condition is the sum of the partition entries. Higher
codimension = more restrictive:

| Partition | Codimension | Typical Use |
|---|---|---|
| `[1]` | 1 | Read access |
| `[2]` | 2 | Write access |
| `[1,1]` | 2 | Read + audit |
| `[2,1]` | 3 | Manage |
| `[2,2]` | 4 | Admin (point class) |

## Schubert Intersection

When you check multiple capabilities, Schubert computes their **Schubert intersection**.
The **intersection number** (Littlewood-Richardson coefficient) tells you how many
configurations satisfy all conditions simultaneously:

- **Positive integer**: access is granted with that many configurations
- **Zero**: the conditions are geometrically impossible together (the killer feature)
- **Too many conditions** (> dimension): overconstrained — access denied

## Key Algebraic Identities

> **Note:** These are identities in algebraic geometry (facts about Grassmannians),
> not validated security properties. The security relevance depends on the formal
> mapping from your domain to Schubert conditions. See
> [Distributed Game Sync](../design/distributed-game-sync.md) for the formal mapping.

- σ₁⁴ = 2 in Gr(2,4) — four read-like conditions yield exactly 2 configurations
- σ₂ · σ₁₁ = 0 — write + internal-audit is geometrically impossible
- Composition is commutative — grant order doesn't matter
- Grant-revoke identity — grant then revoke = no net change

## Notation Guide

| Symbol | Meaning |
|---|---|
| σ_λ | Schubert class indexed by partition λ |
| σ₁ | Schubert class for partition [1] (codimension 1) |
| σ₂ | Schubert class for partition [2] (codimension 2) |
| σ₁₁ | Schubert class for partition [1,1] (codimension 2, different direction from σ₂) |
| σ₂₂ | Schubert class for partition [2,2] — the **point class** (codimension 4 in Gr(2,4), the class of a single point) |

The partition [λ₁, λ₂, ...] and the Schubert class σ_λ refer to the same object.
We use partitions when defining capabilities and σ notation when computing intersections.

## External References

- [Grassmannian — Wikipedia](https://en.wikipedia.org/wiki/Grassmannian)
- [Schubert calculus — Wikipedia](https://en.wikipedia.org/wiki/Schubert_calculus)
- [Littlewood-Richardson rule — Wikipedia](https://en.wikipedia.org/wiki/Littlewood%E2%80%93Richardson_rule)
- [Schubert variety — Wolfram MathWorld](https://mathworld.wolfram.com/SchubertVariety.html)
- [Capability-based security — Wikipedia](https://en.wikipedia.org/wiki/Capability-based_security)

### Prior Work in Access Control

- [Fuzzy Multi-Level Security — Cheng et al. (2007)](https://doi.org/10.1109/SP.2007.18) — Continuous trust for MLS (scalar, not geometric)
- [RBAC Models — Sandhu et al. (1996)](https://doi.org/10.1109/2.485845) — Standard role-based model (boolean)
- [ABAC Guide — NIST SP 800-162](https://nvlpubs.nist.gov/nistpubs/specialpublications/NIST.sp.800-162.pdf) — Attribute-based model (boolean)
- [ReBAC — Fong (2011)](https://doi.org/10.1145/1943601.1943605) — Relationship-based access control (graph-based)
- [Proof-Carrying Authentication — Appel & Felten (1999)](https://doi.org/10.1145/319512.319558) — Logical proofs as auth tokens
- [Conflict-Free Replicated Data Types — Shapiro et al. (2011)](https://doi.org/10.1007/978-3-642-24550-3_29) — CRDT foundations
