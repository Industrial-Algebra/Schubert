// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

/**
 * schubert-tsukoshi — pure-TypeScript geometric access control.
 *
 * A zero-dependency extraction of the Schubert Rust crate's access-control
 * model. Capabilities are Schubert conditions on a Grassmannian; access is the
 * intersection of those conditions, and the intersection number counts valid
 * configurations. The σ₂·σ₁₁ = 0 impossibility case works in the browser.
 *
 * @example
 * ```ts
 * import { AccessController } from "@industrialalgebra/schubert-tsukoshi";
 *
 * const acl = new AccessController("gr24");
 * ```
 *
 * @packageDocumentation
 */

export { AccessController, grassmannianTag } from "./controller.js";
export type { SupportedGrassmannian } from "./controller.js";
export type {
  AccessDecision,
  Capability,
  CapabilityKind,
  DeniedAccess,
  GrantedAccess,
  ImpossibleAccess,
  Partition,
  PrincipalId,
  UnderconstrainedAccess,
} from "./types.js";
export { isGranted, isImpossible } from "./types.js";
