# Schubert

> Replace boolean allow/deny with geometric access control. Quantitative decisions.
> Impossibility detection. Continuous trust.

Schubert is a Rust library that reimagines access control through **Schubert calculus** — a
branch of algebraic geometry. Instead of returning `true` or `false`, Schubert tells you
*how many* valid configurations exist for a given set of capabilities. When conditions are
geometrically impossible to satisfy together, Schubert catches conflicts that traditional
boolean AND checks would silently approve.

## Why Schubert?

Traditional access control gives you a boolean. You either can or you can't. This breaks
down in complex systems:

- **Two capabilities conflict but individually are fine** — a boolean AND approves.
  Schubert detects the geometrical impossibility.
- **Trust degrades over time** — boolean systems can't express partial trust. Schubert
  models continuous trust with wall-crossing analysis.
- **Cross-domain access is guesswork** — can a capability in one domain translate to
  another? Schubert's Schubert intersection answers exactly.
- **Rate limiting is arbitrary** — Schubert scales rate limits by intersection numbers,
  giving higher-trust principals more throughput.

## The Killer Feature: Impossibility Detection

Consider a user with **write** (σ₂) and **internal-audit** (σ₁₁) capabilities in
Gr(2,4). Each capability is individually valid. Together? They're **geometrically
impossible** — no subspace of ℝ⁴ can simultaneously satisfy both conditions.

A traditional RBAC system with boolean AND would approve. Schubert returns
`AccessDecision::Impossible` and tells you exactly which capabilities conflict.

## What Schubert Is Not

- **Not an authentication system** — identity belongs to your OAuth/OIDC provider
- **Not a network service** — Schubert is a library you embed
- **Not a policy server** — no REST API, no gRPC, no wire protocol
- **Not a single Grassmannian** — `MultiController` manages cross-domain access

## The Industrial Algebra Ecosystem

Schubert depends on three sibling projects:

| Crate | Version | Role |
|---|---|---|
| [Amari](https://github.com/Industrial-Algebra/Amari) | 0.23 | Schubert calculus engine — Grassmannians, intersection numbers |
| [Karpal](https://github.com/Industrial-Algebra/Karpal) | 0.5 | Formal verification — type-level proofs, SMT/Lean obligations |
| [Minuet](https://github.com/Industrial-Algebra/Minuet) | 0.3 | Holographic memory — cosine-similarity access patterns |

## License

Schubert is licensed under **Apache-2.0** — a permissive open-source license
with patent grant and attribution requirements. See [LICENSE](https://github.com/Industrial-Algebra/Schubert/blob/main/LICENSE)
for the full text. All contributors must sign the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).

For licensing inquiries: <license@industrialalgebra.com>
