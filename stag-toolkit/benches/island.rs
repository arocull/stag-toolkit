use criterion::{Criterion, criterion_group, criterion_main};
use glam::{Mat4, Vec3};
use stag_toolkit::math::raycast::{Raycast, RaycastParameters};
use stag_toolkit::math::sdf;
use stag_toolkit::mesh::island;
use stag_toolkit::mesh::island::SettingsMesh;
use std::time::Duration;

fn island_builder(c: &mut Criterion) {
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

    c.bench_function("island/bake_bounding_box", |b| {
        b.iter(|| {
            data.bake_bounding_box();
        })
    });

    c.bench_function("island/bake_voxels", |b| {
        b.iter(|| {
            data.clear_voxels();
            data.bake_voxels();
        })
    });

    c.bench_function("island/bake_preview", |b| {
        b.iter(|| {
            data.clear_mesh_preview();
            data.bake_preview();
        })
    });

    // Bakes an island WITHOUT ambient occlusion
    c.bench_function("island/bake_mesh", |b| {
        b.iter(|| {
            data.clear_mesh_baked();
            data.bake_mesh();
        })
    });

    c.bench_function("island/bake_collision", |b| {
        b.iter(|| {
            data.clear_collision();
            data.bake_collision();
        })
    });
}

fn mesh_baking(c: &mut Criterion) {
    let mut data = island::Data::default();
    let shape_list = vec![sdf::Shape::torus(
        Mat4::IDENTITY,
        1.0,
        5.0,
        sdf::ShapeOperation::Union,
    )];
    data.set_shapes(shape_list.clone());

    data.bake_bounding_box();
    data.bake_voxels();
    data.bake_preview();
    let mut preview = data.get_mesh_preview().unwrap().clone();

    let mut raycast_params =
        RaycastParameters::new(Vec3::new(0.0, 10.0, 5.0), Vec3::NEG_Y, f32::INFINITY, false);
    c.bench_function("mesh/raycast", |b| {
        b.iter(|| {
            preview.raycast(raycast_params).unwrap();
        })
    });

    raycast_params.hit_backfaces = true;
    c.bench_function("mesh/raycast (with backfaces)", |b| {
        b.iter(|| {
            preview.raycast(raycast_params).unwrap();
        })
    });

    c.bench_function("mesh/optimize", |b| {
        b.iter(|| {
            let mut cleanup_test = preview.clone();
            cleanup_test.optimize(1e-6);
        })
    });

    // Optimize mesh before performing bakes
    preview.optimize(1e-6);

    c.bench_function("mesh/get_normals_smooth", |b| {
        b.iter(|| {
            preview.get_normals_smooth();
        })
    });

    preview.bake_normals_smooth();

    c.bench_function("mesh/get_ambient_occlusion", |b| {
        b.iter(|| {
            preview.get_ambient_occlusion(8, 10.0, 321);
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
        .measurement_time(Duration::from_secs(90));
    targets=mesh_baking,island_builder
);
criterion_main!(island);
