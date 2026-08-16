// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import { VectorClock } from "@cliffy-ga/tsukoshi/protocols";
import { partitionsLe } from "../partition.js";
import type { GrantCapability } from "../crypto/tokens.js";

/**
 * A replicated capability-grant set — a CRDT for synchronizing *who has what
 * capability* across leaderless replicas, built on cliffy-tsukoshi's
 * `VectorClock`.
 *
 * # When to use this vs. the crypto layer
 *
 * - **`./crypto` (GrantToken)** — untrusted clients present a signed bearer
 *   token; the verifier checks an Ed25519 signature. Proof-carrying.
 * - **`./protocols` (GrantCRDT)** — trusted replicas synchronize the grant
 *   *set* itself and answer access queries from merged state. No per-request
 *   signature; replicas trust the op origin (or wrap ops in signatures if they
 *   don't). This is the distributed/RBAC-cache mode.
 *
 * The two are complementary: a cluster might use a GrantCRDT to keep every
 * replica's grant state convergent, and issue GrantTokens to clients from
 * that state.
 *
 * # CRDT semantics
 *
 * State-based last-writer-wins map keyed by `(principal, capabilityId)`:
 * - each local op ticks the replica's vector clock;
 * - on merge, per key the entry with the later vector clock wins
 *   (`happensBefore`);
 * - on **concurrent** ops, **grant wins over revoke** (add-wins, OR-Set
 *   flavor), with the originating nodeId as a deterministic tiebreak.
 *
 * This merge is commutative, associative, and idempotent — i.e. a lattice
 * join — so replicas reach strong eventual consistency regardless of merge
 * order or message duplication.
 *
 * # Why not cliffy's `GeometricCRDT`?
 *
 * cliffy-tsukoshi's `GeometricCRDT` models a GA3 multivector state mutated by
 * geometric-product/sandwich transforms (for interpolating positions). That
 * does not map onto a grant/revoke set. This module uses only cliffy's
 * `VectorClock` causal-ordering primitive, which is exactly right for grants.
 *
 * @example
 * ```ts
 * import { GrantCRDT } from "@industrialalgebra/schubert-tsukoshi/protocols";
 *
 * const a = new GrantCRDT("node-a");
 * const b = new GrantCRDT("node-b");
 *
 * a.grant("alice", { id: "memory:read",  partition: [1] });
 * a.grant("alice", { id: "memory:write", partition: [2] });
 *
 * b.merge(a);                       // b receives a's state
 * b.may("alice", [1]);              // true — read implied by read grant
 * b.revoke("alice", "memory:write");
 * b.capabilitiesOf("alice");        // [{ id: "memory:read", partition: [1] }]
 * ```
 */
export class GrantCRDT {
  private clock: VectorClock;

  /**
   * @param nodeId  Stable id of this replica. Used for clock ticks and as the
   *                concurrent-op tiebreak.
   */
  constructor(
    private readonly nodeId: string,
    clock?: VectorClock,
    private readonly entries: Map<string, GrantEntry> = new Map(),
  ) {
    this.clock = clock ?? new VectorClock();
  }

  /** This replica's id. */
  get id(): string {
    return this.nodeId;
  }

  /** The number of tracked grant/revoke entries (debugging/stats). */
  get size(): number {
    return this.entries.size;
  }

  /** Locally grant a capability (with its Schubert partition) to a principal. */
  grant(principal: string, capability: GrantCapability): void {
    this.clock.tick(this.nodeId);
    this.entries.set(keyOf(principal, capability.id), {
      op: "grant",
      capability,
      clock: this.clock.clone(),
      nodeId: this.nodeId,
    });
  }

  /** Locally revoke a capability from a principal. */
  revoke(principal: string, capabilityId: string): void {
    this.clock.tick(this.nodeId);
    this.entries.set(keyOf(principal, capabilityId), {
      op: "revoke",
      clock: this.clock.clone(),
      nodeId: this.nodeId,
    });
  }

  /**
   * All currently-granted capabilities for `principal` (entries whose latest
   * op is a grant). Partition data is preserved for geometric containment.
   */
  capabilitiesOf(principal: string): GrantCapability[] {
    const prefix = `${principal}\u0000`;
    const out: GrantCapability[] = [];
    for (const [k, entry] of this.entries) {
      if (entry.op === "grant" && k.startsWith(prefix) && entry.capability) {
        out.push(entry.capability);
      }
    }
    return out;
  }

  /** Does `principal` currently hold `capabilityId`? */
  holds(principal: string, capabilityId: string): boolean {
    const entry = this.entries.get(keyOf(principal, capabilityId));
    return entry?.op === "grant";
  }

  /**
   * Geometric containment: does `principal` hold a granted capability whose
   * partition dominates `required` component-wise? (Write implies read;
   * admin = max partition implies all.) Uses the shared pure `partitionsLe`.
   */
  may(principal: string, required: readonly number[]): boolean {
    return this.capabilitiesOf(principal).some((cap) =>
      partitionsLe(required, cap.partition),
    );
  }

  /**
   * Merge another replica's state into this one (state-based LWW-map CRDT).
   * Per key, the later-clock entry wins; concurrent ops resolve add-wins
   * (grant beats revoke), with nodeId as a deterministic tiebreak. Then the
   * local clock is advanced to reflect observed events.
   *
   * The argument is consumed but not mutated. The result is equivalent
   * regardless of merge order.
   */
  merge(other: GrantCRDT): void {
    for (const [k, otherEntry] of other.entries) {
      const mine = this.entries.get(k);
      if (!mine) {
        this.entries.set(k, otherEntry);
      } else {
        this.entries.set(k, pickWinner(mine, otherEntry));
      }
    }
    this.clock.update(other.clock);
  }

  /**
   * Snapshot for transport. Round-trips through {@link fromJSON}. The clock
   * is serialized component-wise; entries carry their originating clock and
   * nodeId so merges on the receiver reproduce identical results.
   */
  toJSON(): SerializedGrantCRDT {
    const entries: SerializedEntry[] = [];
    for (const [k, entry] of this.entries) {
      const [principal, capabilityId] = splitKey(k);
      entries.push({
        principal,
        capabilityId,
        op: entry.op,
        capability: entry.capability,
        clock: entry.clock.toJSON(),
        nodeId: entry.nodeId,
      });
    }
    return {
      nodeId: this.nodeId,
      clock: this.clock.toJSON(),
      entries,
    };
  }

  /**
   * Reconstruct a GrantCRDT from a {@link toJSON} snapshot. The `nodeId`
   * argument becomes the reconstructing replica's own id (it may differ from
   * the serialized `nodeId` if a snapshot is loaded onto a new replica — the
   * serialized per-entry nodeIds are what matter for merge tiebreaks).
   */
  static fromJSON(data: SerializedGrantCRDT, nodeId?: string): GrantCRDT {
    const clock = new VectorClock(new Map(Object.entries(data.clock)));
    const entries = new Map<string, GrantEntry>();
    for (const e of data.entries) {
      const entryClock = new VectorClock(new Map(Object.entries(e.clock)));
      entries.set(keyOf(e.principal, e.capabilityId), {
        op: e.op,
        capability: e.capability,
        clock: entryClock,
        nodeId: e.nodeId,
      });
    }
    return new GrantCRDT(nodeId ?? data.nodeId, clock, entries);
  }

  /** Defensive copy of this replica's clock (for inspection/testing). */
  clockClone(): VectorClock {
    return this.clock.clone();
  }
}

/** One entry in the grant map: the latest op for a (principal, capabilityId). */
interface GrantEntry {
  readonly op: "grant" | "revoke";
  /** Present iff `op === "grant"`. */
  readonly capability?: GrantCapability;
  readonly clock: VectorClock;
  readonly nodeId: string;
}

/** Wire shape of a single entry. */
interface SerializedEntry {
  readonly principal: string;
  readonly capabilityId: string;
  readonly op: "grant" | "revoke";
  readonly capability?: GrantCapability;
  readonly clock: Record<string, number>;
  readonly nodeId: string;
}

/** Wire shape of a GrantCRDT snapshot. */
export interface SerializedGrantCRDT {
  readonly nodeId: string;
  readonly clock: Record<string, number>;
  readonly entries: readonly SerializedEntry[];
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/** Map key joining principal and capability id with a NUL separator. */
function keyOf(principal: string, capabilityId: string): string {
  return `${principal}\u0000${capabilityId}`;
}

function splitKey(k: string): [string, string] {
  const i = k.indexOf("\u0000");
  return [k.slice(0, i), k.slice(i + 1)];
}

/**
 * Pick the winning entry under the LWW total order:
 *   1. later vector clock (by `happensBefore`);
 *   2. on concurrency, grant beats revoke (add-wins);
 *   3. on concurrency and same op, smaller nodeId wins (deterministic).
 */
function pickWinner(a: GrantEntry, b: GrantEntry): GrantEntry {
  if (a.clock.happensBefore(b.clock)) return b;
  if (b.clock.happensBefore(a.clock)) return a;
  // Concurrent.
  if (a.op !== b.op) {
    return a.op === "grant" ? a : b; // add-wins
  }
  return a.nodeId <= b.nodeId ? a : b; // deterministic tiebreak
}
