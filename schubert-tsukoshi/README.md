# schubert-tsukoshi

> Pure-TypeScript geometric access control via Schubert calculus — zero runtime dependencies, with impossibility detection in the browser.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`schubert-tsukoshi` is a TypeScript extraction of the [Schubert](https://schubert.industrialalgebra.com) Rust crate's access-control model. Capabilities are **Schubert conditions** on a Grassmannian; access is the **intersection** of those conditions, and the **intersection number** counts exactly how many valid configurations exist. The headline feature — detecting that a policy is *geometrically impossible* (the σ₂·σ₁₁ = 0 case) — works in the browser with no backend.

It follows the `@cliffy-ga/tsukoshi` pattern: pure TypeScript, zero dependencies, works everywhere (browser, Node, Deno, React Native).

## Install

```bash
npm install @industrialalgebra/schubert-tsukoshi
```

> **Publish:** the `@industrialalgebra` npm org exists; this package is publish-ready (run `npm publish` from `schubert-tsukoshi/`).

## Quick start

```ts
import { AccessController } from "@industrialalgebra/schubert-tsukoshi";

const acl = new AccessController("gr24"); // Gr(2,4): a 4-dimensional policy space

acl.registerCapability({ id: "read",   partition: [1],   kind: "read"  });
acl.registerCapability({ id: "write",  partition: [2],   kind: "write" });
acl.registerCapability({ id: "manage", partition: [2,1], kind: "manage" });

const alice = acl.createPrincipal("alice");
acl.grant(alice, "read");
acl.grant(alice, "manage");

const decision = acl.check(alice, ["read", "manage"]);
// => { kind: "granted", configurations: 1 }
```

## The killer feature: impossibility detection

A principal can hold **every required capability** and still be denied, because the conditions are geometrically incompatible. This catches policy conflicts that pass a naive set-membership check:

```ts
const acl = new AccessController("gr24");
acl.registerCapability({ id: "write", partition: [2],   kind: "write" });
acl.registerCapability({ id: "dwide", partition: [1,1], kind: "custom" });

const mallory = acl.createPrincipal("mallory");
acl.grant(mallory, "write");
acl.grant(mallory, "dwide");          // set-membership: ✓ both held

acl.check(mallory, ["write", "dwide"]);
// => { kind: "impossible", conflicting: ["write", "dwide"] }
//    σ₂ · σ₁₁ = 0 on Gr(2,4) — the policy is geometrically void.
```

Notably, σ₂·σ₁₁ is **not** impossible on the larger Gr(3,6) — the bigger box admits the product. Same code, different Grassmannian, different verdict.

## Decisions

`check()` returns a discriminated union:

| `kind`            | Meaning                                                                 |
| ----------------- | ----------------------------------------------------------------------- |
| `"granted"`       | Access permitted; `configurations` = Schubert intersection number.      |
| `"impossible"`    | All caps held, but geometrically incompatible (`conflicting` ids).      |
| `"underconstrained"` | Conditions don't pin a finite config set (`dimension` > 0).         |
| `"denied"`        | Principal lacks a required capability (`missing` id) — set-membership.  |

## Capability tokens (`./crypto`)

Ed25519-signed capability and grant tokens — a TypeScript mirror of the Rust
`crypto` module, with a wire format **byte-compatible** with the Rust crate.
Tokens issued in Rust verify here and vice-versa.

> The `./crypto` subpath depends on `@noble/ed25519` and `@noble/hashes`
> (audited, standard). The core `.` entry remains zero-dependency.

```ts
import {
  Issuer,
  Verifier,
  grantToBytes,
  grantFromBytes,
} from "@industrialalgebra/schubert-tsukoshi/crypto";

// Persist the issuer by its 32-byte seed (store securely, e.g. 0600 file).
const issuer = Issuer.fromSeedHex(process.env.ISSUER_SEED!);
const grant = issuer.issueGrant("alice", [
  { id: "memory:read",  partition: [1] },
  { id: "memory:write", partition: [2] },
]);

// Verifiers only need the public key.
const verifier = new Verifier(issuer.publicKey());
verifier.verifyGrant(grant);           // throws if signature invalid
verifier.may(grant, [1]);              // true — geometric containment

// Wire format roundtrips and is Rust-compatible:
const bytes = grantToBytes(grant);
grantFromBytes(bytes);
```

**Interop guarantee:** the cross-validation test suite
(`src/crypto/tokens.test.ts`) issues tokens from a fixed seed in both Rust and
TypeScript and asserts byte-identical output. Regenerate the Rust vectors with
`cargo run --example tsukoshi_crypto_vectors --features crypto`.

## Supported Grassmannians

Precomputed tables ship for three policy spaces:

| Tag    | Gr(k,n)  | Dimension | Use case                |
| ------ | -------- | --------- | ----------------------- |
| `gr24` | Gr(2,4)  | 4         | Standard RBAC           |
| `gr36` | Gr(3,6)  | 9         | Complex multi-tenant    |
| `gr48` | Gr(4,8)  | 16        | Enterprise policy space |

To add more, regenerate the tables from the Rust crate:

```bash
cargo run --example generate_ts_lr_tables > schubert-tsukoshi/src/lr-tables.ts
```

The generator uses `amari-enumerative`'s exact `schubert_product`, so the TypeScript tables are byte-faithful to the Rust math — no transcription risk.

## How it works

Schubert classes are multiplied via precomputed Littlewood-Richardson coefficient tables. A check folds the position and required classes into a single Schubert polynomial, then reads the coefficient of the point class σ_{(n−k)^k}:

- coefficient `> 0` → **granted** (that many configurations)
- coefficient `== 0`, codimension sum `== dim` → **impossible**
- codimension sum `< dim` → **underconstrained**

This mirrors the Littlewood-Richardson branch of Schubert's `AccessController::check`.

## Distributed grants (`./protocols`)

A replicated capability-grant set — `GrantCRDT` — for synchronizing *who has
what capability* across leaderless replicas, built on `@cliffy-ga/tsukoshi`'s
`VectorClock`. This is the **trusted-replica** counterpart to the
proof-carrying tokens in `./crypto`:

- `./crypto` (`GrantToken`) — untrusted clients present a signed bearer token.
- `./protocols` (`GrantCRDT`) — trusted replicas converge on a shared grant set
  and answer access queries from merged state.

```ts
import { GrantCRDT } from "@industrialalgebra/schubert-tsukoshi/protocols";

const hub = new GrantCRDT("hub");
hub.grant("alice", { id: "memory:read",  partition: [1] });
hub.grant("alice", { id: "memory:write", partition: [2] });

const edge = new GrantCRDT("edge");
edge.merge(hub);                  // edge converges to hub's grant set
edge.may("alice", [1]);           // true — geometric containment over merged state
edge.revoke("alice", "memory:write");

// Snapshot/restore for transport:
const snap = edge.toJSON();
const restored = GrantCRDT.fromJSON(snap, "edge-2");
```

**Semantics:** state-based last-writer-wins map keyed by `(principal,
capability)`. The later vector clock wins; on concurrent ops **grant wins over
revoke** (add-wins), with nodeId as a deterministic tiebreak. Merge is
commutative, associative, and idempotent, so replicas reach strong eventual
consistency regardless of delivery order.

> The `./protocols` subpath depends on `@cliffy-ga/tsukoshi` (for `VectorClock`)
> but **not** on `@noble/ed25519` — its `may()` uses the shared, dependency-free
> `partitionsLe`. (`@cliffy-ga/tsukoshi` will migrate to the
> `@industrialalgebra` scope during its refactor.)

## License

Apache-2.0. © Industrial Algebra.
