// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Pure partition helpers — zero dependencies, shared by the crypto and
 * protocols subpaths so that `./protocols` does not transitively pull in the
 * Ed25519 (`@noble/ed25519`) dependency of `./crypto`.
 */

/**
 * Component-wise partition comparison: `a ≤ b`, padding the shorter sequence
 * with zeros. Mirrors the Schubert Rust crate's `partitions_le`.
 *
 * Geometric containment on the partition lattice:
 * - `[1] ≤ [2]` → write implies read
 * - `[x] ≤ [4,4,4,4]` for any valid `x` on Gr(4,8) → admin implies all
 */
export function partitionsLe(
  a: readonly number[],
  b: readonly number[],
): boolean {
  const maxLen = Math.max(a.length, b.length);
  for (let i = 0; i < maxLen; i++) {
    const av = a[i] ?? 0;
    const bv = b[i] ?? 0;
    if (av > bv) return false;
  }
  return true;
}
