// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import {
  Issuer,
  Verifier,
  grantFromBytes,
  grantToBytes,
  partitionsLe,
  singleFromBytes,
  singleToBytes,
} from "./tokens.js";
import { fromHex, toHex } from "./wire.js";

/**
 * Cross-validation vectors, captured verbatim from the Rust crate via
 * `cargo run --example tsukoshi_crypto_vectors --features crypto`.
 *
 * The issuer uses the fixed seed [0x2a; 32]. If the Rust wire format or
 * signing message ever changes, regenerate these with that example.
 */
const SEED_HEX = "2a".repeat(32); // [0x2a; 32] — matches the Rust vector generator
const PUBLIC_KEY_HEX =
  "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61";
const SINGLE_TOKEN_HEX =
  "0005616c696365000b6d656d6f72793a72656164197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d6176f4c9b45241d153e014d2b3a25f95e68b5753a7f335b4e49bcc14c47746f08008bbd8f61efc90604adc82bb9598bab7591f220650ead384ef92fdef9eb25407";
const GRANT_TOKEN_HEX =
  "0003626f620002000b6d656d6f72793a726561640101000c6d656d6f72793a77726974650102197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d6118d91bfa1d6c2807c04ae0e53441f71cc4bbd7705e6fa04d22119426196c2060a9b6ca0535babb341e6e6c90e2d9110aeccef2f86f10bf6ebf33a01dcafd460f";
// A grant with an extra capability injected AFTER signing — Rust emits it but
// it MUST fail signature verification.
const TAMPERED_TOKEN_HEX =
  "00056361726f6c0002000b6d656d6f72793a726561640101000c6d656d6f72793a61646d696e0404040404197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d610685f05e28519b7c858dfaa2efd19c8ad832b75e0cf9ee563cfa6a30f9989eb881f9220b17832af3a4e7f6438c3fb05b8c3275cd59c3eb1fb182cf885c01d502";

describe("crypto — Rust → TS verification (wire format + signature interop)", () => {
  it("derives the same public key from the seed", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    expect(issuer.publicKeyHex()).toBe(PUBLIC_KEY_HEX);
  });

  it("parses and verifies a Rust-issued single-capability token", () => {
    const token = singleFromBytes(fromHex(SINGLE_TOKEN_HEX));
    expect(token.principal).toBe("alice");
    expect(token.capability).toBe("memory:read");
    expect(toHex(token.issuerKey)).toBe(PUBLIC_KEY_HEX);
    const verifier = new Verifier(PUBLIC_KEY_HEX);
    expect(() => verifier.verifySingle(token)).not.toThrow();
  });

  it("parses and verifies a Rust-issued grant token (capabilities canonicalized)", () => {
    const grant = grantFromBytes(fromHex(GRANT_TOKEN_HEX));
    expect(grant.principal).toBe("bob");
    expect(grant.capabilities).toHaveLength(2);
    // Canonical order: read [1] before write [2], despite Rust-side issue order.
    expect(grant.capabilities[0].id).toBe("memory:read");
    expect(grant.capabilities[1].id).toBe("memory:write");
    const verifier = new Verifier(PUBLIC_KEY_HEX);
    expect(() => verifier.verifyGrant(grant)).not.toThrow();
    expect(verifier.may(grant, [1])).toBe(true); // read granted
    expect(verifier.may(grant, [2])).toBe(true); // write granted
    expect(verifier.may(grant, [3])).toBe(false); // not granted
  });

  it("rejects a Rust-emitted tampered grant (extra capability post-sign)", () => {
    const tampered = grantFromBytes(fromHex(TAMPERED_TOKEN_HEX));
    expect(tampered.capabilities).toHaveLength(2); // read + injected admin
    const verifier = new Verifier(PUBLIC_KEY_HEX);
    expect(() => verifier.verifyGrant(tampered)).toThrow(/invalid grant token signature/);
  });
});

describe("crypto — TS → Rust byte-identity (reproduce Rust output exactly)", () => {
  // These are the strongest tests: if TS produces byte-identical tokens to
  // Rust for the same seed + inputs, interop is bidirectionally proven.

  it("issues a single-capability token byte-identical to Rust", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const token = issuer.issue("alice", "memory:read");
    expect(toHex(singleToBytes(token))).toBe(SINGLE_TOKEN_HEX);
  });

  it("issues a grant byte-identical to Rust regardless of input order", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    // Deliberately non-canonical input order (write before read).
    const grant = issuer.issueGrant("bob", [
      { id: "memory:write", partition: [2] },
      { id: "memory:read", partition: [1] },
    ]);
    expect(toHex(grantToBytes(grant))).toBe(GRANT_TOKEN_HEX);
    // Stored capabilities are canonicalized.
    expect(grant.capabilities[0].id).toBe("memory:read");
    expect(grant.capabilities[1].id).toBe("memory:write");
  });
});

describe("crypto — roundtrip and security properties", () => {
  it("single token toBytes/fromBytes roundtrips", () => {
    const issuer = Issuer.generate();
    const token = issuer.issue("carol", "data:delete");
    const roundtrip = singleFromBytes(singleToBytes(token));
    expect(roundtrip.principal).toBe("carol");
    expect(roundtrip.capability).toBe("data:delete");
    new Verifier(issuer.publicKey()).verifySingle(roundtrip);
  });

  it("grant toBytes/fromBytes roundtrips", () => {
    const issuer = Issuer.generate();
    const grant = issuer.issueGrant("dave", [
      { id: "a", partition: [1] },
      { id: "b", partition: [2, 1] },
    ]);
    const roundtrip = grantFromBytes(grantToBytes(grant));
    new Verifier(issuer.publicKey()).verifyGrant(roundtrip);
    expect(roundtrip.capabilities).toHaveLength(2);
  });

  it("rejects a token signed by a different issuer", () => {
    const a = Issuer.generate();
    const b = Issuer.generate();
    const token = a.issue("eve", "secret");
    expect(() => new Verifier(b.publicKey()).verifySingle(token)).toThrow();
  });

  it("partitionsLe: write implies read, admin implies all", () => {
    expect(partitionsLe([1], [2])).toBe(true); // read <= write
    expect(partitionsLe([2], [1])).toBe(false); // write !<= read
    expect(partitionsLe([1], [4, 4, 4, 4])).toBe(true); // admin implies all
    expect(partitionsLe([2, 1], [4, 4, 4, 4])).toBe(true);
  });

  it("may() reflects geometric containment, not just set membership", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const grant = issuer.issueGrant("frank", [
      { id: "memory:write", partition: [2] }, // write only
    ]);
    const verifier = new Verifier(issuer.publicKey());
    verifier.verifyGrant(grant);
    expect(verifier.may(grant, [1])).toBe(true); // read implied by write
    expect(verifier.may(grant, [2])).toBe(true); // write explicit
    expect(verifier.may(grant, [2, 1])).toBe(false); // manage not implied
  });

  it("rejects malformed wire input", () => {
    expect(() => singleFromBytes(fromHex("00"))).toThrow(/truncated/);
    expect(() => singleFromBytes(fromHex(SINGLE_TOKEN_HEX.slice(0, -2)))).toThrow();
    // trailing bytes
    expect(() =>
      singleFromBytes(concat(fromHex(SINGLE_TOKEN_HEX), new Uint8Array([0xff]))),
    ).toThrow(/trailing bytes/);
  });
});

function concat(...parts: Uint8Array[]): Uint8Array {
  let total = 0;
  for (const p of parts) total += p.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const p of parts) {
    out.set(p, off);
    off += p.length;
  }
  return out;
}
