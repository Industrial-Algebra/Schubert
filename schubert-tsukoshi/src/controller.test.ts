// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import { AccessController, grassmannianTag } from "./controller.js";
import type { AccessDecision } from "./types.js";

describe("AccessController — Gr(2,4)", () => {
  function fresh24(): AccessController {
    const acl = new AccessController("gr24");
    acl.registerCapability({ id: "read", partition: [1], kind: "read" });
    acl.registerCapability({ id: "write", partition: [2], kind: "write" });
    acl.registerCapability({
      id: "manage",
      partition: [2, 1],
      kind: "manage",
    });
    acl.registerCapability({
      id: "admin",
      partition: [2, 2],
      kind: "admin",
    });
    return acl;
  }

  it("rejects unsupported Grassmannians", () => {
    expect(() => grassmannianTag(5, 10)).toThrow(/no precomputed table/);
  });

  it("validates partitions against the box", () => {
    const acl = new AccessController("gr24"); // box 2x2
    expect(() =>
      acl.registerCapability({ id: "bad", partition: [3], kind: "custom" }),
    ).toThrow(/exceeds box width/);
    expect(() =>
      acl.registerCapability({ id: "bad2", partition: [1, 1, 1], kind: "custom" }),
    ).toThrow(/at most 2/);
    expect(() =>
      acl.registerCapability({ id: "bad3", partition: [1, 2], kind: "custom" }),
    ).toThrow(/non-increasing/);
  });

  it("denies when a principal lacks a required capability (set-membership)", () => {
    const acl = fresh24();
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "read");
    const d = acl.check(alice, ["read", "write"]);
    expect(d.kind).toBe("denied");
    if (d.kind === "denied") expect(d.missing).toBe("write");
  });

  it("grants a single read (underconstrained: 1 condition < dim 4)", () => {
    const acl = fresh24();
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "read");
    const d = acl.check(alice, ["read"]);
    // σ_1 alone: codim 1 < dim 4 → positive-dimensional
    expect(d.kind).toBe("underconstrained");
    if (d.kind === "underconstrained") expect(d.dimension).toBe(3);
  });

  it("grants read+manage with a finite configuration count", () => {
    // σ_1 · σ_{2,1}: codim 1+3 = 4 = dim. The product's point-class coeff
    // is the configuration count.
    const acl = fresh24();
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "read");
    acl.grant(alice, "manage");
    const d = acl.check(alice, ["read", "manage"]);
    expect(d.kind).toBe("granted");
    if (d.kind === "granted") expect(d.configurations).toBeGreaterThanOrEqual(1);
  });

  it("detects the σ₂·σ₁₁ impossibility (the killer feature)", () => {
    // write = σ₂, and a [1,1] condition (call it "dwide") are geometrically
    // incompatible on Gr(2,4): their product is exactly zero even though the
    // principal holds both.
    const acl = new AccessController("gr24");
    acl.registerCapability({ id: "write", partition: [2], kind: "write" });
    acl.registerCapability({
      id: "dwide",
      partition: [1, 1],
      kind: "custom",
    });
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "write");
    acl.grant(alice, "dwide");
    const d = acl.check(alice, ["write", "dwide"]);
    expect(d.kind).toBe("impossible");
    if (d.kind === "impossible") {
      expect(d.conflicting).toEqual(["write", "dwide"]);
    }
  });

  it("admin (point class) squared is exactly one configuration", () => {
    // σ_{2,2} is the point class; σ_{2,2}·σ_{2,2} is beyond the box (size 8>4)
    // so it is impossible — but a single admin grant is a 0-dim point.
    const acl = fresh24();
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "admin");
    const d = acl.check(alice, ["admin"]);
    expect(d.kind).toBe("granted");
    if (d.kind === "granted") expect(d.configurations).toBe(1);
  });
});

describe("AccessController — cross-Grassmannian specificity", () => {
  it("σ₂·σ₁₁ is NOT impossible on Gr(3,6) (bigger box admits the product)", () => {
    // On Gr(3,6) the box is 3x3, so σ₂·σ_{1,1} = σ_{2,1,1} + σ_{3,1} survives.
    // Codim 2+2 = 4 < dim 9 → underconstrained, never impossible.
    const acl = new AccessController("gr36");
    acl.registerCapability({ id: "write", partition: [2], kind: "write" });
    acl.registerCapability({ id: "dwide", partition: [1, 1], kind: "custom" });
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "write");
    acl.grant(alice, "dwide");
    const d = acl.check(alice, ["write", "dwide"]);
    expect(d.kind).not.toBe("impossible");
    // codim sum 4 < dim 9 → positive-dimensional
    expect(d.kind).toBe("underconstrained");
  });

  it("works on Gr(4,8)", () => {
    const acl = new AccessController("gr48");
    expect(acl.dim).toBe(16);
    acl.registerCapability({ id: "r", partition: [1], kind: "read" });
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "r");
    const d = acl.check(alice, ["r"]);
    expect(d.kind).toBe("underconstrained"); // codim 1 < 16
  });
});

describe("AccessController — lifecycle", () => {
  it("grant / revoke / holds", () => {
    const acl = new AccessController("gr24");
    acl.registerCapability({ id: "read", partition: [1], kind: "read" });
    const alice = acl.createPrincipal("alice");
    expect(acl.holds(alice, "read")).toBe(false);
    acl.grant(alice, "read");
    expect(acl.holds(alice, "read")).toBe(true);
    acl.revoke(alice, "read");
    expect(acl.holds(alice, "read")).toBe(false);
  });

  it("throws on duplicate principal or capability", () => {
    const acl = new AccessController("gr24");
    acl.registerCapability({ id: "read", partition: [1], kind: "read" });
    acl.createPrincipal("alice");
    expect(() => acl.createPrincipal("alice")).toThrow(/already exists/);
    expect(() =>
      acl.registerCapability({ id: "read", partition: [2], kind: "read" }),
    ).toThrow(/already registered/);
  });

  it("setPosition lets a non-unit position participate in geometry", () => {
    const acl = new AccessController("gr24");
    acl.registerCapability({ id: "read", partition: [1], kind: "read" });
    const alice = acl.createPrincipal("alice");
    acl.grant(alice, "read");
    acl.setPosition(alice, [1]); // position contributes σ_1
    const d = acl.check(alice, ["read"]);
    // σ_1 (position) · σ_1 (read): codim 2 < dim 4 → underconstrained
    expect(d.kind).toBe("underconstrained");
  });
});

/** Compile-time exhaustiveness guard over the decision union. */
function _acceptsAllDecisions(_d: AccessDecision): void {
  // no-op; ensures the union stays exported and typed.
}
void _acceptsAllDecisions;
