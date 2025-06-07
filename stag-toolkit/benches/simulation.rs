use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use glam::Vec3;
use stag_toolkit::simulation::rope::{jakobsen_constraint, jakobsen_constraint_single};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("jakobsen_contraint", |b| {
        b.iter(|| {
            jakobsen_constraint(
                black_box(Vec3::splat(2.0)),
                black_box(Vec3::splat(-2.0)),
                black_box(0.5),
            )
        })
    });

    c.bench_function("jakobsen_contraint_single", |b| {
        b.iter(|| {
            jakobsen_constraint_single(
                black_box(Vec3::splat(2.0)),
                black_box(Vec3::splat(-2.0)),
                black_box(0.5),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
