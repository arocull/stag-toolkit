use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;

use stag_toolkit::mesh::island;

fn island_builder(c: &mut Criterion) {
    let group = c.benchmark_group("island");

    let data = island::Data::new(
        island::SettingsVoxels::default(),
        island::SettingsMesh::default(),
        island::SettingsCollision::default(),
    );

    // data.set_shapes()

    // group.bench_function("island_builder", |b| {
    //     b.iter(|| {
    //
    //     })
    // })
}

criterion_group!(
    name=rope;
    config=Criterion::default()
        .significance_level(0.03)
        .noise_threshold(0.008)
        .sample_size(250)
        .measurement_time(Duration::from_secs(13));
    targets=island_builder
);
criterion_main!(rope);
