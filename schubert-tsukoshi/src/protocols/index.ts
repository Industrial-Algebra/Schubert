// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Distributed protocols for schubert-tsukoshi.
 *
 * **Status: Phase 2 (planned for v0.4.0 follow-up).** This entry point is
 * reserved for the `GrantCRDT` — a capability-grant set reconciled across
 * replicas using cliffy-tsukoshi's `VectorClock` primitive (the op-based
 * last-writer-wins merge pattern). It is intentionally empty so the package's
 * dual-entry shape matches `@cliffy-ga/tsukoshi` today; the implementation
 * lands in a focused sprint after the v0.4.0 core ships.
 *
 * @packageDocumentation
 */

export {};
