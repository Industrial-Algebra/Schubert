// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Binary wire-format helpers — a byte-exact mirror of the Schubert Rust
 * crate's `crypto` wire format (`read_u16` / `read_str` / `read_u8` /
 * `read_bytes`). Tokens serialized here are interchangeable with the Rust
 * `CapabilityToken::to_bytes` / `GrantToken::to_bytes` output.
 */

/** Encode a UTF-8 string with a big-endian u16 length prefix. */
export function writeLenString(buf: number[], s: string): void {
  const bytes = stringToUtf8(s);
  const len = bytes.length;
  if (len > 0xffff) throw new Error("schubert-tsukoshi: string too long for u16 length");
  buf.push((len >> 8) & 0xff, len & 0xff);
  for (const b of bytes) buf.push(b);
}

/** Write a big-endian u16. */
export function writeU16(buf: number[], value: number): void {
  if (value > 0xffff) throw new Error("schubert-tsukoshi: u16 overflow");
  buf.push((value >> 8) & 0xff, value & 0xff);
}

/** Write a single u8. */
export function writeU8(buf: number[], value: number): void {
  if (value > 0xff) throw new Error("schubert-tsukoshi: u8 overflow");
  buf.push(value & 0xff);
}

/** Cursor-based reader over a byte array. */
export class Reader {
  private pos = 0;
  constructor(private readonly data: Uint8Array) {}

  u16(): number {
    this.ensure(2);
    const v = (this.data[this.pos] << 8) | this.data[this.pos + 1];
    this.pos += 2;
    return v;
  }

  u8(): number {
    this.ensure(1);
    const v = this.data[this.pos];
    this.pos += 1;
    return v;
  }

  /** Read a length-prefixed UTF-8 string. */
  string(): string {
    const len = this.u16();
    this.ensure(len);
    const slice = this.data.subarray(this.pos, this.pos + len);
    this.pos += len;
    return utf8ToString(slice);
  }

  /** Read `n` raw bytes. */
  bytes(n: number): Uint8Array {
    this.ensure(n);
    const slice = this.data.subarray(this.pos, this.pos + n);
    this.pos += n;
    return slice;
  }

  /** Must be at end-of-input. */
  expectEnd(): void {
    if (this.pos !== this.data.length) {
      throw new Error("schubert-tsukoshi: trailing bytes in token");
    }
  }

  get remaining(): number {
    return this.data.length - this.pos;
  }

  private ensure(n: number): void {
    if (this.pos + n > this.data.length) {
      throw new Error("schubert-tsukoshi: truncated token");
    }
  }
}

/** UTF-8 encode without depending on TextEncoder being present (it is, everywhere). */
function stringToUtf8(s: string): Uint8Array {
  return new TextEncoder().encode(s);
}

function utf8ToString(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

/** Concatenate byte arrays. */
export function concat(...parts: Uint8Array[]): Uint8Array {
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

/** Parse a hex string into a Uint8Array. */
export function fromHex(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) throw new Error("schubert-tsukoshi: odd-length hex");
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(hex.substr(i * 2, 2), 16);
  }
  return out;
}

/** Render a Uint8Array as lowercase hex. */
export function toHex(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += b.toString(16).padStart(2, "0");
  return s;
}

/** Convert a number[] (built by the writers) to a Uint8Array. */
export function toBytes(buf: number[]): Uint8Array {
  return Uint8Array.from(buf);
}
