use criterion::{Criterion, criterion_group, criterion_main};
use glam::{Mat4, Vec3};
use stag_toolkit::math::sdf;
use stag_toolkit::mesh::island;
use std::time::Duration;

fn island_builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("IslandBuilder");
    let mut data = island::Data::default();

    let shape_list = vec![
        sdf::Shape::sphere(Mat4::IDENTITY, 3.0, sdf::ShapeOperation::Union),
        sdf::Shape::torus(
            Mat4::from_translation(Vec3::Y),
            0.75,
            5.0,
            sdf::ShapeOperation::Union,
        ),
        sdf::Shape::torus(
            Mat4::from_translation(Vec3::NEG_Y),
            0.75,
            5.0,
            sdf::ShapeOperation::Union,
        ),
        sdf::Shape::torus(
            Mat4::from_translation(Vec3::Y * 5.0),
            0.675,
            3.0,
            sdf::ShapeOperation::Union,
        ),
        sdf::Shape::torus(
            Mat4::from_translation(Vec3::NEG_Y * 5.0),
            0.675,
            3.0,
            sdf::ShapeOperation::Union,
        ),
    ];
    data.set_shapes(shape_list.clone());

    group.bench_function("bake_bounding_box", |b| {
        b.iter(|| {
            data.bake_bounding_box();
        })
    });

    group.bench_function("bake_voxels", |b| {
        b.iter(|| {
            data.clear_voxels();
            data.bake_voxels();
        })
    });

    group.bench_function("bake_preview", |b| {
        b.iter(|| {
            data.clear_mesh_preview();
            data.bake_preview();
        })
    });

    // Bakes an island WITHOUT ambient occlusion
    group.bench_function("bake_mesh", |b| {
        b.iter(|| {
            data.clear_mesh_baked();
            data.bake_mesh();
        })
    });

    group.bench_function("bake_collision", |b| {
        b.iter(|| {
            data.clear_collision();
            data.bake_collision();
        })
    });
}

criterion_group!(
    name=island;
    config=Criterion::default()
        .significance_level(0.03)
        .noise_threshold(0.008)
        .sample_size(200)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(120));
    targets=island_builder
);
criterion_main!(island);
