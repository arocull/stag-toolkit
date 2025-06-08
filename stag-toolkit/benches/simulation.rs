use criterion::{Criterion, criterion_group, criterion_main};
use std::{collections::HashMap, hint::black_box, time::Duration};

use glam::{Vec3, Vec4, vec4};
use stag_toolkit::simulation::rope::{RopeData, jakobsen_constraint, jakobsen_constraint_single};

fn rope_constraints(c: &mut Criterion) {
    // Test constraint functions by themselves
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

fn rope_simulation(c: &mut Criterion) {
    // Generate a new rope with 100 points
    let mut rope = RopeData::new(10.0, 0.1);
    // Set spring constant and constraint iterations to what we use in Abyss
    rope.spring_constant = 10000.0;
    rope.constraint_iterations = 150;

    // Create an instance-focused binding map, with binds on either end of the rope
    let mut instance_bindings: HashMap<i64, Vec4> = HashMap::new();
    instance_bindings.insert(0, vec4(0.0, 0.0, 0.0, 0.0));
    instance_bindings.insert(1, vec4(0.0, 0.0, -10.0, 1.0));

    let bindings = rope.unique_bind_map(&instance_bindings);
    const DELTA: f64 = 1.0 / 60.0;

    // Benchmark factor<->idx functions
    let rope_bind_index = rope.clone();
    c.bench_function("rope.bind_index", |b| {
        b.iter(|| {
            rope_bind_index.bind_index(black_box(0.37));
        });
    });
    drop(rope_bind_index);

    let rope_bind_factor = rope.clone();
    c.bench_function("rope.bind_factor", |b| {
        b.iter(|| {
            rope_bind_factor.bind_factor(black_box(37));
        });
    });
    drop(rope_bind_factor);

    // Benchmark creating a unique bind-map
    let rope_unique_bind_map = rope.clone();
    c.bench_function("rope.unique_bind_map", |b| {
        b.iter(|| {
            rope_unique_bind_map.unique_bind_map(black_box(&instance_bindings));
        });
    });
    drop(rope_unique_bind_map);

    // Benchmark tension calculations
    let mut rope_tension = rope.clone();
    c.bench_function("rope.tension", |b| {
        b.iter(|| {
            rope_tension.tension(black_box(&bindings));
        });
    });
    drop(rope_tension);

    // Benchmark simulation functions
    let mut rope_tick = rope.clone();
    c.bench_function("rope.step", |b| {
        b.iter(|| {
            rope_tick.step(black_box(DELTA));
        });
    });
    drop(rope_tick);

    let mut rope_constraint = rope.clone();
    c.bench_function("rope.constrain", |b| {
        b.iter(|| {
            rope_constraint.constrain(black_box(&bindings));
        });
    });
    drop(rope_constraint);

    // Benchmark lots of step + constrain iterations
    let mut rope_sim = rope.clone();
    c.bench_function("rope.step + rope.constrain", |b| {
        b.iter(|| {
            rope_sim.step(black_box(DELTA));
            rope_sim.constrain(black_box(&bindings));
        });
    });
    drop(rope_sim);

    // Now, benchmark everything together for a good metric on processing time within Godot
    let mut rope_godot_sim = rope.clone();
    c.bench_function("Godot simulation tick", |b| {
        b.iter(|| {
            let bind_map = rope_godot_sim.unique_bind_map(black_box(&instance_bindings));
            rope_godot_sim.tension(&bind_map);
            rope_godot_sim.step(black_box(DELTA));
            rope_godot_sim.constrain(&bind_map);
        });
    });
    drop(rope_godot_sim);
}

criterion_group!(
    name=rope;
    config=Criterion::default()
        .significance_level(0.03)
        .noise_threshold(0.008)
        .sample_size(250)
        .measurement_time(Duration::from_secs(13));
    targets=rope_constraints, rope_simulation
);
criterion_main!(rope);
