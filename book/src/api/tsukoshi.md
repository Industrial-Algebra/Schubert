# schubert-tsukoshi (TypeScript)

Schubert's core access-control model is also available as a **pure-TypeScript**
package — [`@industrialalgebra/schubert-tsukoshi`](https://www.npmjs.com/package/@industrialalgebra/schubert-tsukoshi)
— for browser, Node, Deno, and React Native. It ships zero runtime dependencies
and runs the Schubert geometry (including impossibility detection) with no
backend and no WASM.

> The TypeScript package lives in this repository under `schubert-tsukoshi/` and
> has its own README with full API docs. This page is a cross-reference.

## What it provides

| Subpath | What | Dependencies |
|---|---|---|
| `@industrialalgebra/schubert-tsukoshi` | Core: `AccessController`, LR tables, impossibility detection | none (zero-dep) |
| `@industrialalgebra/schubert-tsukoshi/crypto` | Ed25519 `CapabilityToken` / `GrantToken` issue + verify | `@noble/ed25519`, `@noble/hashes` |
| `@industrialalgebra/schubert-tsukoshi/protocols` | `GrantCRDT` — replicated grant set over cliffy-tsukoshi's `VectorClock` | `@cliffy-ga/tsukoshi` |

## The killer feature, in the browser

A principal can hold every required capability and still be denied, because the
conditions are geometrically incompatible:

```ts
import { AccessController } from "@industrialalgebra/schubert-tsukoshi";

const acl = new AccessController("gr24");
acl.registerCapability({ id: "write", partition: [2],   kind: "write" });
acl.registerCapability({ id: "dwide", partition: [1,1], kind: "custom" });

const m = acl.createPrincipal("mallory");
acl.grant(m, "write");
acl.grant(m, "dwide");                          // both held
acl.check(m, ["write", "dwide"]);
// => { kind: "impossible", conflicting: ["write", "dwide"] }
//    σ₂ · σ₁₁ = 0 on Gr(2,4)
```

## Rust ↔ TypeScript interop

The `crypto` subpath uses the **exact same Ed25519 wire format** as the Rust
crate. Tokens issued in Rust verify in TypeScript and vice-versa — proven by a
cross-validation test suite that asserts byte-identical output from a fixed
seed. A Rust-backed service can issue grants that a TypeScript client verifies,
or a TypeScript issuer can mint tokens a Rust verifier accepts.

## Relationship to the Rust crate

- The LR tables are **generated** from the Rust crate's own exact
  `schubert_product` (run `cargo run --example generate_ts_lr_tables`), so the
  TypeScript math is byte-faithful to the Rust math.
- `schubert-tsukoshi` targets the three standard policy spaces: `Gr(2,4)`,
  `Gr(3,6)`, `Gr(4,8)`. Larger spaces require regenerating the tables.
- It does **not** include Karpal verification or surreal trust (those need the
  Rust type system / exact rationals).

See `schubert-tsukoshi/README.md` in this repository for installation, the full
API, and how to add Grassmannians.
