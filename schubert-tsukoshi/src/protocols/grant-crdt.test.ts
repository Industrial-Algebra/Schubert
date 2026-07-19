// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

import { describe, expect, it } from "vitest";
import { GrantCRDT } from "./grant-crdt.js";

const read = { id: "memory:read", partition: [1] };
const write = { id: "memory:write", partition: [2] };
const manage = { id: "memory:manage", partition: [2, 1] };
const admin = { id: "memory:admin", partition: [4, 4, 4, 4] };

/** Two CRDTs are "equal" for convergence checks if their grant sets match. */
function grantSet(c: GrantCRDT, principal: string): Set<string> {
  return new Set(c.capabilitiesOf(principal).map((cap) => cap.id));
}

describe("GrantCRDT — local grant/revoke", () => {
  it("grants and reports capabilities with partition data", () => {
    const c = new GrantCRDT("n1");
    c.grant("alice", read);
    c.grant("alice", write);
    const caps = c.capabilitiesOf("alice");
    expect(caps.map((x) => x.id).sort()).toEqual(["memory:read", "memory:write"]);
    expect(caps.find((x) => x.id === "memory:write")?.partition).toEqual([2]);
  });

  it("revoke removes a capability", () => {
    const c = new GrantCRDT("n1");
    c.grant("alice", read);
    c.revoke("alice", "memory:read");
    expect(c.capabilitiesOf("alice")).toEqual([]);
    expect(c.holds("alice", "memory:read")).toBe(false);
  });

  it("may() reflects geometric containment", () => {
    const c = new GrantCRDT("n1");
    c.grant("alice", write); // [2]
    expect(c.may("alice", [1])).toBe(true); // read implied by write
    expect(c.may("alice", [2])).toBe(true); // write explicit
    expect(c.may("alice", [2, 1])).toBe(false); // manage not implied
  });

  it("admin grant implies everything (max partition)", () => {
    const c = new GrantCRDT("n1");
    c.grant("root", admin);
    expect(c.may("root", [1])).toBe(true);
    expect(c.may("root", [2, 1])).toBe(true);
    expect(c.may("root", [4, 4, 4, 4])).toBe(true);
  });

  it("principals are isolated", () => {
    const c = new GrantCRDT("n1");
    c.grant("alice", read);
    expect(c.capabilitiesOf("bob")).toEqual([]);
    expect(c.holds("bob", "memory:read")).toBe(false);
  });
});

describe("GrantCRDT — merge causality", () => {
  it("a later op on the same node overrides an earlier one (causal order)", () => {
    const c = new GrantCRDT("n1");
    c.grant("alice", read); // n1:1
    c.revoke("alice", "memory:read"); // n1:2 — happens-after the grant
    const replica = new GrantCRDT("n2");
    replica.merge(c);
    expect(replica.holds("alice", "memory:read")).toBe(false);
  });

  it("merge propagates grants to an empty replica", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    a.grant("alice", write);
    const b = new GrantCRDT("n2");
    b.merge(a);
    expect(grantSet(b, "alice")).toEqual(new Set(["memory:read", "memory:write"]));
  });

  it("a subsequent revoke after merge overrides the propagated grant", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read); // n1:1
    const b = new GrantCRDT("n2");
    b.merge(a); // b clock now {n1:1}; alice holds read
    b.revoke("alice", "memory:read"); // n2:1, clock {n1:1,n2:1} happens-after n1:1
    a.merge(b);
    expect(a.holds("alice", "memory:read")).toBe(false);
  });
});

describe("GrantCRDT — concurrent conflict resolution (add-wins)", () => {
  it("concurrent grant + revoke → grant wins (add-wins)", () => {
    // n1 grants read (n1:1); n2 revokes read (n2:1) — concurrent, no sync yet.
    const n1 = new GrantCRDT("n1");
    n1.grant("alice", read);
    const n2 = new GrantCRDT("n2");
    n2.grant("alice", read); // both must start granted for a fair revoke test
    n1.grant("alice", read);
    n2.revoke("alice", "memory:read");

    n1.merge(n2); // n1's grant (n1:1) concurrent with n2's revoke (n2:1)
    expect(n1.holds("alice", "memory:read")).toBe(true); // grant wins
  });

  it("two concurrent grants of different capabilities both survive", () => {
    const n1 = new GrantCRDT("n1");
    n1.grant("alice", read);
    const n2 = new GrantCRDT("n2");
    n2.grant("alice", write);
    n1.merge(n2);
    expect(grantSet(n1, "alice")).toEqual(new Set(["memory:read", "memory:write"]));
  });
});

describe("GrantCRDT — CRDT laws (strong eventual consistency)", () => {
  it("commutative: merge(A,B) == merge(B,A)", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    a.revoke("bob", "memory:read");
    const b = new GrantCRDT("n2");
    b.grant("bob", write);
    b.grant("alice", manage);

    const ab = new GrantCRDT("x");
    ab.merge(a);
    ab.merge(b);

    const ba = new GrantCRDT("x");
    ba.merge(b);
    ba.merge(a);

    expect(grantSet(ab, "alice")).toEqual(grantSet(ba, "alice"));
    expect(grantSet(ab, "bob")).toEqual(grantSet(ba, "bob"));
  });

  it("associative: merge(merge(A,B),C) == merge(A,merge(B,C))", () => {
    const mk = () => {
      const c = new GrantCRDT("src");
      c.grant("alice", read);
      return c;
    };
    const a = mk();
    a.grant("alice", write);
    const b = mk();
    b.revoke("alice", "memory:read");
    const c = mk();
    c.grant("alice", manage);

    const left = new GrantCRDT("x");
    left.merge(a);
    left.merge(b);
    const leftFinal = new GrantCRDT("x");
    leftFinal.merge(left);
    leftFinal.merge(c);

    const right = new GrantCRDT("x");
    right.merge(b);
    right.merge(c);
    const rightFinal = new GrantCRDT("x");
    rightFinal.merge(a);
    rightFinal.merge(right);

    expect(grantSet(leftFinal, "alice")).toEqual(grantSet(rightFinal, "alice"));
  });

  it("idempotent: merge(A,A) == A", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    a.grant("alice", write);
    const before = grantSet(a, "alice");
    a.merge(a);
    expect(grantSet(a, "alice")).toEqual(before);
  });

  it("duplicate message delivery is harmless", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    const b = new GrantCRDT("n2");
    b.merge(a);
    b.merge(a); // duplicate
    b.merge(a); // triplicate
    expect(grantSet(b, "alice")).toEqual(new Set(["memory:read"]));
  });
});

describe("GrantCRDT — serialization", () => {
  it("toJSON / fromJSON roundtrips state and merges identically", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    a.grant("alice", write);
    a.revoke("bob", "memory:read");

    const json = a.toJSON();
    const restored = GrantCRDT.fromJSON(json);

    expect(grantSet(restored, "alice")).toEqual(new Set(["memory:read", "memory:write"]));
    expect(restored.holds("bob", "memory:read")).toBe(false);

    // A fresh replica merges the restored snapshot and converges.
    const b = new GrantCRDT("n2");
    b.merge(restored);
    expect(grantSet(b, "alice")).toEqual(grantSet(a, "alice"));
  });

  it("a snapshot loaded onto a new replica continues to advance correctly", () => {
    const a = new GrantCRDT("n1");
    a.grant("alice", read);
    const snap = a.toJSON();

    // Load onto n2 (different nodeId); subsequent local op must outrank n1's.
    const b = GrantCRDT.fromJSON(snap, "n2");
    b.revoke("alice", "memory:read"); // n2:1 happens-after observed n1:1
    a.merge(b);
    expect(a.holds("alice", "memory:read")).toBe(false);
  });
});

describe("GrantCRDT — multi-principal, multi-capability integration", () => {
  it("replicates a small RBAC policy and answers queries from merged state", () => {
    const hub = new GrantCRDT("hub");
    hub.grant("alice", read);
    hub.grant("alice", write);
    hub.grant("bob", read);
    hub.grant("root", admin);

    const edge = new GrantCRDT("edge");
    edge.merge(hub);

    expect(edge.may("alice", [1])).toBe(true);
    expect(edge.may("alice", [2])).toBe(true);
    expect(edge.may("alice", [2, 1])).toBe(false);
    expect(edge.may("bob", [2])).toBe(false); // bob only has read
    expect(edge.may("root", [4, 4, 4, 4])).toBe(true); // root is admin
  });
});
