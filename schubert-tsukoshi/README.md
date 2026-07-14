# schubert-tsukoshi

> Pure-TypeScript geometric access control via Schubert calculus — zero runtime dependencies, with impossibility detection in the browser.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`schubert-tsukoshi` is a TypeScript extraction of the [Schubert](https://schubert.industrialalgebra.com) Rust crate's access-control model. Capabilities are **Schubert conditions** on a Grassmannian; access is the **intersection** of those conditions, and the **intersection number** counts exactly how many valid configurations exist. The headline feature — detecting that a policy is *geometrically impossible* (the σ₂·σ₁₁ = 0 case) — works in the browser with no backend.

It follows the `@cliffy-ga/tsukoshi` pattern: pure TypeScript, zero dependencies, works everywhere (browser, Node, Deno, React Native).

## Install

```bash
npm install @industrial-algebra/schubert-tsukoshi
```

> **Publish note:** the `@industrial-algebra` npm org must be created before first publish. The package is build-ready now.

## Quick start

```ts
import { AccessController } from "@industrial-algebra/schubert-tsukoshi";

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

## Distributed access (Phase 2)

The `./protocols` entry point is reserved for `GrantCRDT` — a capability-grant set reconciled across replicas using `@cliffy-ga/tsukoshi`'s `VectorClock` primitive. It lands in a focused sprint after the v0.4.0 core.

## License

Apache-2.0. © Industrial Algebra.
