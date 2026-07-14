// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import { TABLES, type GrassmannianTable } from "./lr-tables.js";
import type {
  AccessDecision,
  Capability,
  CapabilityKind,
  Partition,
  PrincipalId,
} from "./types.js";

/**
 * A supported Grassmannian Gr(k,n). schubert-tsukoshi ships precomputed
 * Littlewood-Richardson tables for these three policy spaces; larger spaces
 * require regenerating the tables from the Rust crate (see
 * `examples/generate_ts_lr_tables.rs`).
 */
export type SupportedGrassmannian = "gr24" | "gr36" | "gr48";

const TABLE_BY_DIM: ReadonlyMap<string, GrassmannianTable> = new Map(
  Object.entries(TABLES),
);

function tableFor(tag: SupportedGrassmannian): GrassmannianTable {
  const t = TABLE_BY_DIM.get(tag);
  if (!t) throw new Error(`schubert-tsukoshi: unknown Grassmannian "${tag}"`);
  return t;
}

/** Pick the table tag for a given (k, n). */
export function grassmannianTag(
  k: number,
  n: number,
): SupportedGrassmannian {
  const tag = `gr${k}${n}` as SupportedGrassmannian;
  if (!TABLE_BY_DIM.has(tag)) {
    throw new Error(
      `schubert-tsukoshi: Gr(${k},${n}) has no precomputed table. ` +
        `Supported: Gr(2,4), Gr(3,6), Gr(4,8). Regenerate from the Rust crate ` +
        `to add more.`,
    );
  }
  return tag;
}

/** Stable key for a partition, matching the generator's encoding ("" = unit). */
function partKey(p: Partition): string {
  return p.join(",");
}

/** Codimension of a partition = sum of its parts. */
function codimension(p: Partition): number {
  let s = 0;
  for (const x of p) s += x;
  return s;
}

/** A sparse Schubert polynomial: partition-key -> integer coefficient. */
type SchubertPoly = Map<string, number>;

/** Multiply two sparse polynomials using the precomputed LR product table. */
function multiply(
  table: GrassmannianTable,
  a: SchubertPoly,
  b: SchubertPoly,
): SchubertPoly {
  const out: SchubertPoly = new Map();
  for (const [ka, ca] of a) {
    for (const [kb, cb] of b) {
      const terms = table.product[`${ka}|${kb}`];
      if (!terms) continue; // product is 0 for this pair (e.g. σ₂·σ₁₁)
      for (const [nuKey, coeff] of terms) {
        const delta = ca * cb * coeff;
        out.set(nuKey, (out.get(nuKey) ?? 0) + delta);
      }
    }
  }
  return out;
}

/** The unit polynomial σ_() (the whole Grassmannian). */
function unitPoly(): SchubertPoly {
  return new Map([["", 1]]);
}

/** Single-class polynomial σ_λ. */
function classPoly(p: Partition): SchubertPoly {
  return new Map([[partKey(p), 1]]);
}

/**
 * Geometric access controller — the TypeScript mirror of Schubert's
 * `AccessController`.
 *
 * Capabilities are Schubert conditions (partitions); a principal's access is
 * granted when the intersection of the required conditions is non-empty, and
 * the intersection number tells you exactly how many valid configurations
 * exist. Impossibility (σ₂·σ₁₁ = 0) is detected from the precomputed tables.
 *
 * @example
 * ```ts
 * import { AccessController } from "@industrialalgebra/schubert-tsukoshi";
 *
 * const acl = new AccessController("gr24"); // Gr(2,4)
 * acl.registerCapability({ id: "read", partition: [1], kind: "read" });
 * acl.registerCapability({ id: "write", partition: [2], kind: "write" });
 *
 * const alice = acl.createPrincipal("alice");
 * acl.grant(alice, "read");
 * acl.grant(alice, "write");
 *
 * const decision = acl.check(alice, ["read", "write"]);
 * console.log(decision); // { kind: "granted", configurations: 1 }
 * ```
 */
export class AccessController {
  private readonly table: GrassmannianTable;
  private readonly capabilities = new Map<string, Capability>();
  /** principal id -> set of granted capability ids */
  private readonly grants = new Map<PrincipalId, Set<string>>();
  /** principal id -> position partition (default: unit class) */
  private readonly positions = new Map<PrincipalId, Partition>();
  /** The (k, n) of the backing Grassmannian. */
  readonly k: number;
  readonly n: number;
  /** Grassmannian dimension k*(n-k). */
  readonly dim: number;

  constructor(tag: SupportedGrassmannian) {
    this.table = tableFor(tag);
    this.k = this.table.k;
    this.n = this.table.n;
    this.dim = this.table.dim;
  }

  /** Construct from explicit (k, n) dimensions. */
  static forGrassmannian(k: number, n: number): AccessController {
    return new AccessController(grassmannianTag(k, n));
  }

  /** Register a capability (Schubert condition). Throws if the partition is invalid. */
  registerCapability(cap: Omit<Capability, "kind"> & { kind?: CapabilityKind }): void {
    this.validatePartition(cap.partition);
    if (this.capabilities.has(cap.id)) {
      throw new Error(`schubert-tsukoshi: capability "${cap.id}" already registered`);
    }
    this.capabilities.set(cap.id, {
      id: cap.id,
      label: cap.label,
      partition: cap.partition,
      kind: cap.kind ?? "custom",
    });
  }

  /** Create a principal. Throws on duplicate id. */
  createPrincipal(id: PrincipalId): PrincipalId {
    if (this.grants.has(id)) {
      throw new Error(`schubert-tsukoshi: principal "${id}" already exists`);
    }
    this.grants.set(id, new Set());
    this.positions.set(id, []);
    return id;
  }

  /** Grant a capability to a principal. */
  grant(id: PrincipalId, capId: string): void {
    this.requireCapability(capId);
    const set = this.requirePrincipal(id);
    set.add(capId);
  }

  /** Revoke a capability from a principal. */
  revoke(id: PrincipalId, capId: string): void {
    const set = this.grants.get(id);
    set?.delete(capId);
  }

  /** Does a principal currently hold a capability? */
  holds(id: PrincipalId, capId: string): boolean {
    return this.grants.get(id)?.has(capId) ?? false;
  }

  /** Set a principal's geometric position (default is the unit class). */
  setPosition(id: PrincipalId, partition: Partition): void {
    this.validatePartition(partition);
    this.requirePrincipal(id);
    this.positions.set(id, partition);
  }

  /**
   * Check access for a principal against a set of required capabilities.
   *
   * Returns a quantitative {@link AccessDecision}. Set-membership denial
   * (principal lacks a required cap) is decided first; only then does the
   * Schubert geometry run, exactly as in the Rust crate's LR path.
   */
  check(id: PrincipalId, required: readonly string[]): AccessDecision {
    const set = this.requirePrincipal(id);

    // 1. Set-membership: principal must hold every required capability.
    for (const capId of required) {
      if (!set.has(capId)) {
        return { kind: "denied", missing: capId };
      }
    }

    // 2. Resolve required classes (position + required partitions).
    const position = this.positions.get(id) ?? [];
    const requiredPartitions = required.map((capId) => {
      const cap = this.requireCapability(capId);
      return cap.partition;
    });

    // 3. Multiply σ_position · σ_req1 · σ_req2 · ... via the LR table.
    let poly: SchubertPoly = classPoly(position);
    for (const p of requiredPartitions) {
      poly = multiply(this.table, poly, classPoly(p));
    }

    // 4. Classify by codimension sum (faithful to the LR branch).
    const codimSum =
      codimension(position) +
      requiredPartitions.reduce((s, p) => s + codimension(p), 0);

    if (codimSum < this.dim) {
      // Positive-dimensional Schubert variety.
      return { kind: "underconstrained", dimension: this.dim - codimSum };
    }

    // codimSum >= dim: read the point-class coefficient.
    const pointCoeff = poly.get(partKey(this.table.pointClass)) ?? 0;
    if (pointCoeff > 0) {
      return { kind: "granted", configurations: pointCoeff };
    }
    return { kind: "impossible", conflicting: [...required] };
  }

  private requireCapability(capId: string): Capability {
    const cap = this.capabilities.get(capId);
    if (!cap) {
      throw new Error(`schubert-tsukoshi: unknown capability "${capId}"`);
    }
    return cap;
  }

  private requirePrincipal(id: PrincipalId): Set<string> {
    const set = this.grants.get(id);
    if (!set) {
      throw new Error(`schubert-tsukoshi: unknown principal "${id}"`);
    }
    return set;
  }

  private validatePartition(p: Partition): void {
    if (p.length > this.k) {
      throw new Error(
        `schubert-tsukoshi: partition [${p.join(",")}] has ${p.length} parts, ` +
          `but Gr(${this.k},${this.n}) allows at most ${this.k}`,
      );
    }
    for (let i = 0; i < p.length; i++) {
      if (!Number.isInteger(p[i]) || p[i] <= 0) {
        throw new Error(
          `schubert-tsukoshi: partition parts must be positive integers, got [${p.join(",")}]`,
        );
      }
      if (p[i] > this.n - this.k) {
        throw new Error(
          `schubert-tsukoshi: partition part ${p[i]} exceeds box width ${this.n - this.k} ` +
            `for Gr(${this.k},${this.n})`,
        );
      }
      if (i > 0 && p[i] > p[i - 1]) {
        throw new Error(
          `schubert-tsukoshi: partition must be non-increasing, got [${p.join(",")}]`,
        );
      }
    }
  }
}
