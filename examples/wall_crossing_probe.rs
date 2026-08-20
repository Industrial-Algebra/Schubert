// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Wall-crossing composition probe (Roadmap #17, Workstream C of the v0.5.0
//! sprint plan).
//!
//! Runs `analyze_composed_stability` over a family of operadic compositions
//! and reports, for each, whether the composed phase diagram `P_C` is the
//! additive combination of the constituents' retained capabilities — the
//! empirical form of the Kontsevich–Soibelman-type question:
//!
//! > Is `P_C` determined by `P_A` and `P_B`?
//!
//! Where `is_additive` holds, composition is *predictable*. Where it fails,
//! the deviating trust levels (`non_additive_breakpoints`) are emergence
//! signatures — the wall-crossing analogue of a BPS bound state whose spectrum
//! is not the union of its parts.
//!
//! This is the honest empirical instrument: it does not assume the answer,
//! it measures it. Under the current `WallCrossingEngine` the expected
//! outcome is additive behavior (walls are per-capability), and the probe
//! confirms exactly that — establishing the baseline against which any
//! future interaction-aware engine must be judged.
//!
//! Run:
//! ```text
//! cargo run --example wall_crossing_probe
//! ```

use schubert::stability::analyze_composed_stability;
use schubert::{AccessController, Capability, CapabilityKind};

/// A composition case: two principals, a shared interface, and a label.
struct Case<'a> {
    label: &'a str,
    a_caps: &'a [(&'a str, &'a [usize])],
    b_caps: &'a [(&'a str, &'a [usize])],
    interface: &'a str,
}

fn main() -> schubert::Result<()> {
    // Capability ladder on Gr(2,4): distinct codimensions => distinct walls.
    let caps: [(String, Vec<usize>, CapabilityKind); 4] = [
        ("read".into(), vec![1], CapabilityKind::ReadLike),
        ("write".into(), vec![2], CapabilityKind::WriteLike),
        ("manage".into(), vec![2, 1], CapabilityKind::WriteLike),
        ("admin".into(), vec![2, 2], CapabilityKind::AdminLike),
    ];

    // A family of compositions: simple handoff, mixed grants, admin gluing.
    // Each case runs on a fresh controller so principals never collide.
    let cases = [
        Case {
            label: "read -> write handoff",
            a_caps: &[("manage", &[2, 1])],
            b_caps: &[("admin", &[2, 2])],
            interface: "read",
        },
        Case {
            label: "disjoint retained caps",
            a_caps: &[("write", &[2])],
            b_caps: &[("manage", &[2, 1])],
            interface: "read",
        },
        Case {
            label: "overlapping retained caps",
            a_caps: &[("read", &[1]), ("write", &[2])],
            b_caps: &[("read", &[1])],
            interface: "write",
        },
        Case {
            label: "admin gluing",
            a_caps: &[],
            b_caps: &[("read", &[1])],
            interface: "admin",
        },
    ];

    println!("Wall-crossing composition probe — Roadmap #17 (Gr(2,4))");
    println!("{}", "=".repeat(64));

    let mut any_emergent = false;
    for (i, case) in cases.iter().enumerate() {
        // Gr(2,4) — the standard policy space; fresh per case.
        let mut acl = AccessController::new(2, 4)?;
        for (id, partition, kind) in &caps {
            acl.register_capability(Capability::new(
                id.clone(),
                id.clone(),
                partition.clone(),
                *kind,
            ))?;
        }

        let pa = acl.create_principal("probe-a")?;
        let pb = acl.create_principal("probe-b")?;

        // A holds the interface (as output) plus its retained set; B holds
        // the interface (as input) plus its retained set.
        let mut granted_a: Vec<&str> = vec![case.interface];
        for (id, _) in case.a_caps {
            if !granted_a.contains(id) {
                granted_a.push(id);
            }
        }
        for id in granted_a {
            acl.grant(&pa, id)?;
        }
        let mut granted_b: Vec<&str> = vec![case.interface];
        for (id, _) in case.b_caps {
            if !granted_b.contains(id) {
                granted_b.push(id);
            }
        }
        for id in granted_b {
            acl.grant(&pb, id)?;
        }

        let report = analyze_composed_stability(&acl, &pa, case.interface, &pb, case.interface)?;

        let verdict = if report.is_additive {
            "ADDITIVE (P_C = P_A + P_B)"
        } else {
            any_emergent = true;
            "NON-ADDITIVE (emergence!)"
        };
        println!(
            "\n[{}] {}\n  multiplicity: {} | composed caps: {} | {}",
            i + 1,
            case.label,
            report.composition_multiplicity,
            report.phase_diagram_composed.total_capabilities,
            verdict,
        );
        for bp in &report.non_additive_breakpoints {
            println!("    emergence at trust level {:.3}", bp.value());
        }
    }

    println!("\n{}", "-".repeat(64));
    if any_emergent {
        println!("RESULT: non-additivity detected — the composed diagrams are");
        println!("NOT determined by the constituents. Report the breakpoints above.");
    } else {
        println!("RESULT: all compositions additive — under the current");
        println!("WallCrossingEngine, P_C is determined by P_A and P_B (walls are");
        println!("per-capability). This is the baseline; an interaction-aware");
        println!("engine would show deviations here.");
    }

    Ok(())
}
