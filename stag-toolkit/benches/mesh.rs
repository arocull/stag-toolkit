use criterion::{Criterion, criterion_group, criterion_main};
use glam::{Mat4, Vec3};
use stag_toolkit::math::raycast::{Raycast, RaycastParameters, RaycastResult};
use stag_toolkit::math::sdf;
use stag_toolkit::mesh::island;
use std::num::NonZero;
use std::time::Duration;

fn mesh_baking(c: &mut Criterion) {
    let mut group = c.benchmark_group("Trimesh");

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
    let mut preview = data
        .get_mesh_preview()
        .expect("There should be a mesh!")
        .clone();
    preview.bake_raycast_planes();

    let mut raycast_params =
        RaycastParameters::new(Vec3::new(0.0, 10.0, 5.0), Vec3::NEG_Y, f32::INFINITY, false);

    let raycast_many = vec![raycast_params; 100];

    group.bench_function("raycast", |b| b.iter(|| preview.raycast(raycast_params)));

    group.bench_function("raycast_many", |b| {
        b.iter(|| -> Vec<Option<RaycastResult>> {
            raycast_many
                .iter()
                .map(|param| preview.raycast(*param))
                .collect()
        })
    });

    raycast_params.hit_backfaces = true;
    group.bench_function("raycast (with backfaces)", |b| {
        b.iter(|| preview.raycast(raycast_params))
    });

    group.bench_function("optimize", |b| b.iter(|| preview.clone().optimize(1e-6)));

    // Optimize mesh before performing bakes
    preview.optimize(1e-6);

    group.bench_function("get_normals_smooth", |b| {
        b.iter(|| preview.get_normals_smooth())
    });

    preview.bake_normals_smooth();

    group.bench_function("get_ambient_occlusion", |b| {
        b.iter(|| {
            preview.get_ambient_occlusion(
                8,
                10.0,
                321,
                NonZero::new(16).expect("This should never fail"),
            )
        })
    });
}

criterion_group!(
    name=mesh;
    config=Criterion::default()
        .significance_level(0.03)
        .noise_threshold(0.008)
        .sample_size(100)
        .warm_up_time(Duration::from_secs(5))
        .measurement_time(Duration::from_secs(90));
    targets=mesh_baking
);
criterion_main!(mesh);
