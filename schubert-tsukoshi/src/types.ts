// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * Core types for schubert-tsukoshi — pure-TypeScript geometric access control.
 *
 * These mirror the public API of the Schubert Rust crate, but the math is done
 * via precomputed Littlewood-Richardson lookup tables (see {@link ../lr-tables.ts}),
 * with no runtime Rust/WASM dependency.
 */

/** A Schubert partition: a non-increasing sequence of positive integers. */
export type Partition = readonly number[];

/**
 * The kind of a capability. As in the Rust crate, the kind is informational
 * for the TS check — the {@link Capability.partition} drives all geometry.
 */
export type CapabilityKind =
  | "read"
  | "write"
  | "admin"
  | "manage"
  | "composite"
  | "custom";

/** A registered capability: a named Schubert condition. */
export interface Capability {
  /** Stable identifier (e.g. `"memory:read"`). */
  readonly id: string;
  /** Human-readable label. */
  readonly label?: string;
  /** Schubert partition defining the condition. */
  readonly partition: Partition;
  /** Informational kind. */
  readonly kind: CapabilityKind;
}

/** A principal's stable identifier. Opaque string. */
export type PrincipalId = string;

/**
 * The quantitative result of an access check — the central Schubert idea:
 * access is not boolean but geometric.
 *
 * - {@link GrantedAccess}: access permitted, with the number of valid
 *   configurations (Schubert intersection number).
 * - {@link ImpossibleAccess}: the principal holds every required capability,
 *   yet they are **geometrically incompatible** (the σ₂·σ₁₁ = 0 case). This is
 *   Schubert's killer feature: it catches policy conflicts that pass a naive
 *   set-membership check.
 * - {@link UnderconstrainedAccess}: the required conditions do not pin down a
 *   finite set of configurations (positive-dimensional Schubert variety).
 * - {@link DeniedAccess}: the principal does not hold a required capability
 *   (ordinary set-membership denial, decided before geometry).
 */
export type AccessDecision =
  | GrantedAccess
  | ImpossibleAccess
  | UnderconstrainedAccess
  | DeniedAccess;

export interface GrantedAccess {
  readonly kind: "granted";
  /** Number of valid access configurations (Schubert intersection number). */
  readonly configurations: number;
}

export interface ImpossibleAccess {
  readonly kind: "impossible";
  /** The required capability ids that are mutually incompatible. */
  readonly conflicting: readonly string[];
}

export interface UnderconstrainedAccess {
  readonly kind: "underconstrained";
  /** Positive dimension of the Schubert variety (dim - codimSum). */
  readonly dimension: number;
}

export interface DeniedAccess {
  readonly kind: "denied";
  /** The required capability id the principal does not hold. */
  readonly missing: string;
}

/** Narrow a decision to its granted case. */
export function isGranted(d: AccessDecision): d is GrantedAccess {
  return d.kind === "granted";
}

/** Narrow a decision to its impossible case. */
export function isImpossible(d: AccessDecision): d is ImpossibleAccess {
  return d.kind === "impossible";
}
