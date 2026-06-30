// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Performance benchmarks for Schubert computation paths.
//!
//! Compares all 4 computation engines (Littlewood-Richardson, Localization,
//! Tropical, Matroid) across 3 Grassmannians (Gr(2,4), Gr(3,6), Gr(4,8)).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use schubert::{AccessController, Capability, CapabilityKind, ComputationPath};

fn bench_path(c: &mut Criterion, name: &str, k: usize, n: usize, path: ComputationPath) {
    let mut acl = AccessController::new(k, n).unwrap();

    // Register capabilities
    acl.register_capability(Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike))
        .unwrap();
    acl.register_capability(Capability::new(
        "write", "Write", vec![2], CapabilityKind::WriteLike,
    ))
    .unwrap();
    acl.register_capability(Capability::new(
        "admin", "Admin", vec![2, 1], CapabilityKind::AdminLike,
    ))
    .unwrap();

    let alice = acl.create_principal("alice").unwrap();
    acl.grant(&alice, "read").unwrap();
    acl.grant(&alice, "write").unwrap();

    let label = format!("Gr({k},{n})/{name}");
    c.bench_function(&label, |b| {
        b.iter(|| {
            acl.check_with_path(
                black_box(&alice),
                black_box(&["read", "write"]),
                path,
            )
            .unwrap()
        })
    });
}

fn bench_grassmannian(c: &mut Criterion, k: usize, n: usize) {
    bench_path(
        c, "LittlewoodRichardson", k, n,
        ComputationPath::LittlewoodRichardson,
    );
    bench_path(c, "Localization", k, n, ComputationPath::Localization);
    bench_path(c, "Tropical", k, n, ComputationPath::Tropical);
    bench_path(c, "Matroid", k, n, ComputationPath::Matroid);
}

fn bench_gr24(c: &mut Criterion) {
    bench_grassmannian(c, 2, 4);
}

fn bench_gr36(c: &mut Criterion) {
    bench_grassmannian(c, 3, 6);
}

fn bench_gr48(c: &mut Criterion) {
    bench_grassmannian(c, 4, 8);
}

criterion_group!(benches, bench_gr24, bench_gr36, bench_gr48);
criterion_main!(benches);
