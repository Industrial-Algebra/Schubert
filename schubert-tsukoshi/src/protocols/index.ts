// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Distributed protocols for schubert-tsukoshi.
 *
 * Currently provides {@link GrantCRDT} — a capability-grant set replicated
 * across leaderless replicas using `@cliffy-ga/tsukoshi`'s `VectorClock`
 * (state-based LWW-map with add-wins on concurrent ops). This is the
 * trusted-replica counterpart to the `./crypto` proof-carrying tokens.
 *
 * @example
 * ```ts
 * import { GrantCRDT } from "@industrialalgebra/schubert-tsukoshi/protocols";
 *
 * const replica = new GrantCRDT("node-1");
 * replica.grant("alice", { id: "memory:read", partition: [1] });
 * ```
 *
 * @packageDocumentation
 */

export { GrantCRDT } from "./grant-crdt.js";
export type { SerializedGrantCRDT } from "./grant-crdt.js";
