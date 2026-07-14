// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Cryptographic capability tokens for schubert-tsukoshi — a TypeScript mirror
 * of the Schubert Rust crate's `crypto` module.
 *
 * Ed25519-signed single-capability and multi-capability grant tokens, with a
 * wire format **byte-compatible** with the Rust crate. Tokens issued in Rust
 * verify here and vice-versa (see the cross-validation test suite).
 *
 * Requires `@noble/ed25519` and `@noble/hashes` (regular dependencies of this
 * subpath). The core `@industrial-algebra/schubert-tsukoshi` entry remains
 * zero-dependency.
 *
 * @example
 * ```ts
 * import { Issuer, Verifier } from "@industrial-algebra/schubert-tsukoshi/crypto";
 *
 * const issuer = Issuer.fromSeedHex("2a".repeat(32));
 * const grant = issuer.issueGrant("alice", [
 *   { id: "memory:read",  partition: [1] },
 *   { id: "memory:write", partition: [2] },
 * ]);
 *
 * const verifier = new Verifier(issuer.publicKey());
 * verifier.verifyGrant(grant); // throws if invalid
 * verifier.may(grant, [1]);    // true — read implied by the read grant
 * ```
 *
 * @packageDocumentation
 */

export {
  Issuer,
  Verifier,
  partitionsLe,
  singleFromBytes,
  singleToBytes,
  grantFromBytes,
  grantToBytes,
} from "./tokens.js";
export type {
  CapabilityToken,
  GrantCapability,
  GrantToken,
} from "./tokens.js";
export { fromHex, toHex } from "./wire.js";
