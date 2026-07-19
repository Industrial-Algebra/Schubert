// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import * as ed from "@noble/ed25519";
import { sha512 } from "@noble/hashes/sha2";
import {
  Reader,
  concat,
  fromHex,
  toBytes,
  toHex,
  writeLenString,
  writeU16,
  writeU8,
} from "./wire.js";
import { partitionsLe } from "../partition.js";
export { partitionsLe };

// --- Ed25519 setup (noble needs a SHA-512 implementation plugged in) ---
// This is standard RFC 8032 Ed25519 — byte-compatible with ed25519-dalek.
ed.etc.sha512Sync = (...m: Uint8Array[]) => sha512(ed.etc.concatBytes(...m));

const ISSUER_KEY_LEN = 32;
const SIG_LEN = 64;

/** UTF-8 encode a string to bytes. */
function utf8(s: string): Uint8Array {
  return new TextEncoder().encode(s);
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A capability entry within a grant: an id plus its Schubert partition. */
export interface GrantCapability {
  readonly id: string;
  readonly partition: readonly number[];
}

/** A single-capability Ed25519 token (mirrors Rust `CapabilityToken`). */
export interface CapabilityToken {
  readonly principal: string;
  readonly capability: string;
  readonly issuerKey: Uint8Array;
  readonly signature: Uint8Array;
}

/** A multi-capability Ed25519 grant token (mirrors Rust `GrantToken`). */
export interface GrantToken {
  readonly principal: string;
  readonly capabilities: readonly GrantCapability[];
  readonly issuerKey: Uint8Array;
  readonly signature: Uint8Array;
}

// ---------------------------------------------------------------------------
// Canonical ordering — must match Rust's `sort_by` exactly.
// Rust: `partition.cmp(&partition).then_with(|| id.cmp(id))`, i.e. Vec<usize>
// lexicographic (component-wise, shorter-is-prefix-less) then id bytes.
// ---------------------------------------------------------------------------

function cmpPartition(
  a: readonly number[],
  b: readonly number[],
): number {
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i++) {
    if (a[i] < b[i]) return -1;
    if (a[i] > b[i]) return 1;
  }
  return a.length - b.length;
}

function canonicallySorted(
  caps: readonly GrantCapability[],
): GrantCapability[] {
  return [...caps].sort((a, b) => {
    const c = cmpPartition(a.partition, b.partition);
    return c !== 0 ? c : a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
  });
}

// ---------------------------------------------------------------------------
// Signing-message construction — byte-exact mirror of the Rust crate.
// ---------------------------------------------------------------------------

/** CapabilityToken signing message: principal | 0x00 | capability | 0x00 | issuerKey */
function singleSigningMessage(
  principal: string,
  capability: string,
  issuerKey: Uint8Array,
): Uint8Array {
  return concat(
    utf8(principal),
    new Uint8Array([0]),
    utf8(capability),
    new Uint8Array([0]),
    issuerKey,
  );
}

/**
 * GrantToken signing message:
 *   principal | 0x00 |
 *   (per cap, canonically sorted: id | 0x00 | part_len_u8 | part bytes) |
 *   0x00 | issuerKey
 */
function grantSigningMessage(
  principal: string,
  capabilities: readonly GrantCapability[],
  issuerKey: Uint8Array,
): Uint8Array {
  const parts: Uint8Array[] = [utf8(principal), new Uint8Array([0])];
  for (const cap of canonicallySorted(capabilities)) {
    const idBytes = utf8(cap.id);
    const partBytes = new Uint8Array(cap.partition.length);
    for (let i = 0; i < cap.partition.length; i++) {
      partBytes[i] = cap.partition[i] & 0xff; // matches Rust `part as u8`
    }
    parts.push(idBytes, new Uint8Array([0]), new Uint8Array([cap.partition.length]), partBytes);
  }
  parts.push(new Uint8Array([0]), issuerKey);
  return concat(...parts);
}

// ---------------------------------------------------------------------------
// Wire format — toBytes / fromBytes
// ---------------------------------------------------------------------------

/** Serialize a single-capability token to the binary wire format. */
export function singleToBytes(token: CapabilityToken): Uint8Array {
  const buf: number[] = [];
  writeLenString(buf, token.principal);
  writeLenString(buf, token.capability);
  pushBytes(buf, token.issuerKey);
  pushBytes(buf, token.signature);
  return toBytes(buf);
}

/** Deserialize a single-capability token. Throws if malformed. */
export function singleFromBytes(bytes: Uint8Array): CapabilityToken {
  const r = new Reader(bytes);
  const principal = r.string();
  const capability = r.string();
  const issuerKey = copyBytes(r.bytes(ISSUER_KEY_LEN));
  const signature = copyBytes(r.bytes(SIG_LEN));
  r.expectEnd();
  return { principal, capability, issuerKey, signature };
}

/** Serialize a grant token to the binary wire format. */
export function grantToBytes(token: GrantToken): Uint8Array {
  const buf: number[] = [];
  writeLenString(buf, token.principal);
  writeU16(buf, token.capabilities.length);
  for (const cap of token.capabilities) {
    writeLenString(buf, cap.id);
    writeU8(buf, cap.partition.length);
    for (const p of cap.partition) buf.push(p & 0xff);
  }
  pushBytes(buf, token.issuerKey);
  pushBytes(buf, token.signature);
  return toBytes(buf);
}

/** Deserialize a grant token. Throws if malformed. */
export function grantFromBytes(bytes: Uint8Array): GrantToken {
  const r = new Reader(bytes);
  const principal = r.string();
  const count = r.u16();
  const capabilities: GrantCapability[] = [];
  for (let i = 0; i < count; i++) {
    const id = r.string();
    const plen = r.u8();
    const partition: number[] = [];
    for (let j = 0; j < plen; j++) partition.push(r.u8());
    capabilities.push({ id, partition });
  }
  const issuerKey = copyBytes(r.bytes(ISSUER_KEY_LEN));
  const signature = copyBytes(r.bytes(SIG_LEN));
  r.expectEnd();
  return { principal, capabilities, issuerKey, signature };
}

function pushBytes(buf: number[], bytes: Uint8Array): void {
  for (const b of bytes) buf.push(b);
}

function copyBytes(b: Uint8Array): Uint8Array {
  return b.slice();
}

// ---------------------------------------------------------------------------
// Issuer
// ---------------------------------------------------------------------------

/**
 * An issuer of capability tokens. Holds an Ed25519 signing key derived from a
 * 32-byte seed. The public key is distributed to verifiers.
 */
export class Issuer {
  private constructor(private readonly seed: Uint8Array) {}

  /** Generate an issuer with a cryptographically random seed. */
  static generate(): Issuer {
    const seed = new Uint8Array(32);
    crypto.getRandomValues(seed);
    return new Issuer(seed);
  }

  /** Restore an issuer from a 32-byte seed (the recommended persistence form). */
  static fromSeed(seed: Uint8Array): Issuer {
    if (seed.length !== 32) {
      throw new Error("schubert-tsukoshi: Ed25519 seed must be 32 bytes");
    }
    return new Issuer(seed.slice());
  }

  /** Restore from a hex-encoded seed. */
  static fromSeedHex(hex: string): Issuer {
    return Issuer.fromSeed(fromHex(hex));
  }

  /** The 32-byte public key, for distribution to verifiers. */
  publicKey(): Uint8Array {
    return ed.getPublicKey(this.seed);
  }

  /** The public key as a lowercase hex string (64 chars). */
  publicKeyHex(): string {
    return toHex(this.publicKey());
  }

  /** Issue a single-capability token. */
  issue(principal: string, capability: string): CapabilityToken {
    const issuerKey = this.publicKey();
    const message = singleSigningMessage(principal, capability, issuerKey);
    const signature = ed.sign(message, this.seed);
    return { principal, capability, issuerKey, signature };
  }

  /**
   * Issue a multi-capability grant. Capabilities are canonically sorted before
   * signing, so grant order does not affect the signature.
   */
  issueGrant(principal: string, capabilities: readonly GrantCapability[]): GrantToken {
    const issuerKey = this.publicKey();
    const message = grantSigningMessage(principal, capabilities, issuerKey);
    const signature = ed.sign(message, this.seed);
    // Store capabilities in canonical order (as the Rust token does).
    return {
      principal,
      capabilities: canonicallySorted(capabilities),
      issuerKey,
      signature,
    };
  }
}

// ---------------------------------------------------------------------------
// Verifier
// ---------------------------------------------------------------------------

/**
 * Verifier for capability and grant tokens. Constructed from the issuer's
 * public key.
 */
export class Verifier {
  private readonly publicKey: Uint8Array;
  constructor(publicKey: Uint8Array | string) {
    this.publicKey = typeof publicKey === "string" ? fromHex(publicKey) : publicKey.slice();
    if (this.publicKey.length !== 32) {
      throw new Error("schubert-tsukoshi: public key must be 32 bytes");
    }
  }

  /** Verify a single-capability token's signature. Throws on failure. */
  verifySingle(token: CapabilityToken): void {
    this.requireIssuerKey(token.issuerKey);
    const message = singleSigningMessage(token.principal, token.capability, token.issuerKey);
    if (!ed.verify(token.signature, message, this.publicKey)) {
      throw new Error("schubert-tsukoshi: invalid capability token signature");
    }
  }

  /** Verify a grant token's signature. Throws on failure. */
  verifyGrant(grant: GrantToken): void {
    this.requireIssuerKey(grant.issuerKey);
    const message = grantSigningMessage(grant.principal, grant.capabilities, grant.issuerKey);
    if (!ed.verify(grant.signature, message, this.publicKey)) {
      throw new Error("schubert-tsukoshi: invalid grant token signature");
    }
  }

  /**
   * Geometric containment: does the grant authorize a capability whose
   * partition is `required`? Returns true iff some granted partition λ
   * satisfies `required ≤ λ` component-wise (write implies read;
   * admin = max partition implies all).
   */
  may(grant: GrantToken, required: readonly number[]): boolean {
    return grant.capabilities.some((cap) => partitionsLe(required, cap.partition));
  }

  private requireIssuerKey(tokenKey: Uint8Array): void {
    if (!bytesEqual(tokenKey, this.publicKey)) {
      throw new Error("schubert-tsukoshi: token issuer key does not match verifier");
    }
  }
}

function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}
