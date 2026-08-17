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
 * The issuer uses the fixed seed [0x2a; 32]. Grant vectors pin the nonce
 * (0x42 / 0x43 / 0x99) — plain `issue_grant` is randomized by design (v0.5.0,
 * #20.1). If the Rust wire format or signing message ever changes, regenerate
 * these with that example.
 */
const SEED_HEX = "2a".repeat(32); // [0x2a; 32] — matches the Rust vector generator
const PUBLIC_KEY_HEX =
  "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61";
const SINGLE_TOKEN_HEX =
  "0005616c696365000b6d656d6f72793a72656164197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d6176f4c9b45241d153e014d2b3a25f95e68b5753a7f335b4e49bcc14c47746f08008bbd8f61efc90604adc82bb9598bab7591f220650ead384ef92fdef9eb25407";
const GRANT_TOKEN_HEX =
  "0003626f620002000b6d656d6f72793a726561640101000c6d656d6f72793a77726974650102197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61a8b7089c984c932579252655393b45772fca4b30d6f995ed1102775f555d25239fa160d52c5e8ef009576aa22481038dff726efca773a872744c135442567a044242424242424242424242424242424200";
// v0.5.0 (#20.1): erin's grant carries a signed expiry (2_000_000_000) and
// the pinned nonce 0x99 — wire tail: nonce(16) | tag 01 | u64 BE expiry.
const EXPIRY_GRANT_TOKEN_HEX =
  "00046572696e0001000b6d656d6f72793a726561640101197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d619b5f212c7aa3961355ec61f7f67d357df98457a58229ec68fab75b6de7f1cc6a7439275b42e2967f5cfa34aefcfce2d7422db2d0b324bda214884aadaf9fda0199999999999999999999999999999999010000000077359400";
// A grant with an extra capability injected AFTER signing — Rust emits it but
// it MUST fail signature verification.
const TAMPERED_TOKEN_HEX =
  "00056361726f6c0002000b6d656d6f72793a726561640101000c6d656d6f72793a61646d696e0404040404197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d611831eb063a3341d08e8a12476d3f9c3ed9ce8e9eaa839c1c24553bd08a40d3a66ba51e29f99e954ee0a10e316cc7fa09670a4d4db21443145a93e3689e9090034343434343434343434343434343434300";

const EXPIRY_AT = 2_000_000_000;

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
    // v0.5.0 wire fields: pinned nonce, no expiry.
    expect(toHex(grant.nonce)).toBe("42".repeat(16));
    expect(grant.expiresAt).toBeNull();
    const verifier = new Verifier(PUBLIC_KEY_HEX);
    expect(() => verifier.verifyGrant(grant)).not.toThrow();
    expect(verifier.may(grant, [1])).toBe(true); // read granted
    expect(verifier.may(grant, [2])).toBe(true); // write granted
    expect(verifier.may(grant, [3])).toBe(false); // not granted
  });

  it("parses and verifies a Rust-issued expiry grant — and kills it at the boundary", () => {
    const grant = grantFromBytes(fromHex(EXPIRY_GRANT_TOKEN_HEX));
    expect(grant.principal).toBe("erin");
    expect(toHex(grant.nonce)).toBe("99".repeat(16));
    expect(grant.expiresAt).toBe(EXPIRY_AT);
    const verifier = new Verifier(PUBLIC_KEY_HEX);
    // Alive strictly before the boundary, dead AT it (inclusive, matches Rust).
    expect(() => verifier.verifyGrantAt(grant, EXPIRY_AT - 1)).not.toThrow();
    expect(() => verifier.verifyGrantAt(grant, EXPIRY_AT)).toThrow(/grant expired/);
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
  // Rust for the same seed + inputs, interop is bidirectionally proven —
  // including the expiry and nonce paths.

  it("issues a single-capability token byte-identical to Rust", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const token = issuer.issue("alice", "memory:read");
    expect(toHex(singleToBytes(token))).toBe(SINGLE_TOKEN_HEX);
  });

  it("issues a grant byte-identical to Rust regardless of input order", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    // Deliberately non-canonical input order (write before read); pinned nonce.
    const grant = issuer.issueGrant(
      "bob",
      [
        { id: "memory:write", partition: [2] },
        { id: "memory:read", partition: [1] },
      ],
      { nonce: fromHex("42".repeat(16)) },
    );
    expect(toHex(grantToBytes(grant))).toBe(GRANT_TOKEN_HEX);
    // Stored capabilities are canonicalized.
    expect(grant.capabilities[0].id).toBe("memory:read");
    expect(grant.capabilities[1].id).toBe("memory:write");
  });

  it("issues an expiry grant byte-identical to Rust (nonce + expiry path)", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const grant = issuer.issueGrant(
      "erin",
      [{ id: "memory:read", partition: [1] }],
      { nonce: fromHex("99".repeat(16)), expiresAt: EXPIRY_AT },
    );
    expect(toHex(grantToBytes(grant))).toBe(EXPIRY_GRANT_TOKEN_HEX);
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

  it("grant toBytes/fromBytes roundtrips, preserving nonce and expiry", () => {
    const issuer = Issuer.generate();
    const grant = issuer.issueGrant(
      "dave",
      [
        { id: "a", partition: [1] },
        { id: "b", partition: [2, 1] },
      ],
      { expiresAt: EXPIRY_AT },
    );
    const roundtrip = grantFromBytes(grantToBytes(grant));
    new Verifier(issuer.publicKey()).verifyGrantAt(roundtrip, EXPIRY_AT - 1);
    expect(roundtrip.capabilities).toHaveLength(2);
    expect(roundtrip.nonce).toHaveLength(16);
    expect(roundtrip.expiresAt).toBe(EXPIRY_AT);
  });

  it("default issuance is distinct (random nonce) and both verify", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const a = issuer.issueGrant("sam", [{ id: "read", partition: [1] }]);
    const b = issuer.issueGrant("sam", [{ id: "read", partition: [1] }]);
    expect(toHex(a.nonce)).not.toBe(toHex(b.nonce));
    expect(toHex(grantToBytes(a))).not.toBe(toHex(grantToBytes(b)));
    const verifier = new Verifier(issuer.publicKey());
    expect(() => verifier.verifyGrant(a)).not.toThrow();
    expect(() => verifier.verifyGrant(b)).not.toThrow();
  });

  it("injected nonce is deterministic", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const opts = { nonce: fromHex("07".repeat(16)) };
    const a = issuer.issueGrant("sam", [{ id: "read", partition: [1] }], opts);
    const b = issuer.issueGrant("sam", [{ id: "read", partition: [1] }], opts);
    expect(toHex(grantToBytes(a))).toBe(toHex(grantToBytes(b)));
  });

  it("tampering the nonce or stripping the expiry breaks the signature", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const verifier = new Verifier(issuer.publicKey());
    const grant = issuer.issueGrant(
      "tam",
      [{ id: "read", partition: [1] }],
      { nonce: fromHex("11".repeat(16)), expiresAt: EXPIRY_AT },
    );

    // Flip a nonce byte -> signature failure (signature checked before expiry).
    const flipped = { ...grant, nonce: grant.nonce.slice() };
    flipped.nonce[0] ^= 1;
    expect(() => verifier.verifyGrantAt(flipped, 0)).toThrow(/invalid grant token signature/);

    // Strip the expiry -> signature failure.
    const stripped = { ...grant, expiresAt: null };
    expect(() => verifier.verifyGrantAt(stripped, 0)).toThrow();
  });

  it("verifyGrant uses the wall clock against expiry", () => {
    const issuer = Issuer.fromSeedHex(SEED_HEX);
    const verifier = new Verifier(issuer.publicKey());
    const dead = issuer.issueGrant("x", [{ id: "read", partition: [1] }], {
      expiresAt: 1,
    });
    expect(() => verifier.verifyGrant(dead)).toThrow(/grant expired/);
    const alive = issuer.issueGrant("x", [{ id: "read", partition: [1] }], {
      expiresAt: 9_999_999_999,
    });
    expect(() => verifier.verifyGrant(alive)).not.toThrow();
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
    // Invalid expiry tag (2 is neither "none" nor "present").
    const bad = fromHex(EXPIRY_GRANT_TOKEN_HEX);
    bad[bad.length - 9] = 0x02; // tag sits 9 bytes from the end (tag + 8-byte expiry)
    expect(() => grantFromBytes(bad)).toThrow(/invalid expiry tag/);
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
