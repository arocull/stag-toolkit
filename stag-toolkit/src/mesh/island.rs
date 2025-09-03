use crate::math::bounding_box::BoundingBox;
use crate::math::noise::{Perlin1D, Perlin3D};
use crate::math::sdf::{Shape, ShapeOperation, sample_shape_list, shape_list_bounds};
use crate::math::volumetric::VolumeData;
use crate::mesh::nets::mesh_from_nets;
use crate::mesh::trimesh::{TriangleMesh, TriangleOperations};
use crate::utils;
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape, surface_nets};
use glam::{FloatExt, Mat4, Quat, Vec2, Vec3, Vec4};
use ndshape::ConstShape3u32;
use rayon::prelude::*;
use stag_toolkit_codegen::{ExposeSettings, settings_resource_from};
#[cfg(feature = "godot")]
use {crate::math::types::ToVector3, godot::prelude::*};

const VOLUME_MAX_CELLS: usize = 48;
const VOLUME_MAX_CELLS_TRIM: usize = 44;
type IslandChunkSize = ConstShape3u32<48, 48, 48>; // Same size as VolumeMaxCells

/// Settings for voxel generation.
#[derive(Copy, Clone, PartialEq, ExposeSettings)]
#[settings_resource_from(IslandBuilderSettingsVoxels, Resource)]
pub struct SettingsVoxels {
    /// Number of voxels to pad on each side of the [IslandBuilder] volume.
    #[setting(default = 3, min = 0.0, max = 6.0, soft_max)]
    pub voxel_padding: u32,
    /// Width/height/depth of a voxel. This is the approximate resolution of the resulting [IslandBuilder] mesh.
    #[setting(default=Vec3::splat(0.275), min=0.05, max=1.0, incr=0.001, soft_max, unit="m")]
    pub voxel_size: Vec3,

    /// Frequency of noise directly added to the SDF sampling value, in local space.
    #[setting(default=Vec3::splat(1.0),min=0.0,max=10.0,incr=0.001,soft_max)]
    pub sampling_density_noise_frequency: Vec3,
    /// Amplitude of noise directly added to the SDF sampling value.
    #[setting(
        default = 0.1,
        min = 0.0,
        max = 1.5,
        incr = 0.001,
        soft_max,
        unit = "m"
    )]
    pub sampling_density_noise_amplitude: f64,

    /// Frequency of noise directly added to the SDF sampling position.
    #[setting(default=Vec3::splat(0.3),min=0.0,max=1.0,incr=0.001,soft_max)]
    pub sampling_offset_noise_frequency: Vec3,
    #[setting(default=Vec3::splat(0.2),min=0.0,max=1.0,incr=0.001,soft_max,unit="m")]
    /// Amplitude of noise directly added to the SDF sampling position.
    pub sampling_offset_noise_amplitude: Vec3,

    /// Rounding distance to apply to edges of Signed Distance Field primitives.
    #[setting(default = 1.6, min = 0.0, max = 2.0, soft_max, unit = "m")]
    pub sdf_edge_radius: f32,
    /// Number of smoothing iterations to apply to voxels immediately after sampling Signed Distance Fields.
    #[setting(default = 4, min = 0.0, max = 20.0, soft_max)]
    pub sdf_smooth_iterations: u32,
    /// Radius of voxels to include in each smoothing pass applied immediately after sampling Signed Distance Fields.
    #[setting(default = 3, min = 0.0, max = 5.0, soft_max)]
    pub sdf_smooth_radius_voxels: u32,
    /// Weighting of each smoothing pass applied immediately after sampling Signed Distance Fields.
    #[setting(default = 0.95, min = 0.0, max = 1.0)]
    pub sdf_smooth_weight: f32,

    /// Frequency scale for striation noise on each local axis.
    #[setting(default = Vec3::new(0.2,1.0,0.2), min = 0.0, max = 10.0, incr = 0.001, soft_max)]
    pub striation_frequency: Vec3,
    /// Amplitude of striation noise on each local axis.
    #[setting(
        default = 0.15,
        min = 0.0,
        max = 1.0,
        incr = 0.001,
        soft_max,
        unit = "m"
    )]
    pub striation_amplitude: f64,

    /// Number of voxels per worker group.
    /// This is a performance setting and will not affect the output result.
    #[setting(default=IslandChunkSize::USIZE as u32,min=1.0)]
    pub worker_group_size: u32,
}

/// Settings for mesh generation.
#[derive(Copy, Clone, PartialEq, ExposeSettings)]
#[settings_resource_from(IslandBuilderSettingsMesh, Resource)]
pub struct SettingsMesh {
    /// Distance threshold for vertices to be merged for the visual mesh.
    #[setting(
        default = 0.04,
        min = 0.0,
        max = 1.0,
        incr = 0.001,
        soft_max,
        unit = "m"
    )]
    pub vertex_merge_distance: f32,

    /// Whether to bake Ambient Occlusion to the Red channel.
    /// The Red channel defaults to 1.0 if Ambient Occlusion is not baked.
    #[setting(default = false)]
    pub ao_enabled: bool,
    /// Sampling radius for Ambient Occlusion.
    #[setting(
        default = 8.0,
        min = 0.01,
        max = 50.0,
        incr = 0.01,
        soft_max,
        unit = "m"
    )]
    pub ao_radius: f32,
    /// Weighting value for linearly blending a base value of 1.0 with the baked Ambient Occlusion.
    #[setting(default = 0.8, min = 0.0, max = 1.0, incr = 0.001)]
    pub ao_strength: f32,
    /// Number of ambient occlusion samples to perform.
    #[setting(default = 32, min = 1.0, max = 256.0, incr = 1.0)]
    pub ao_samples: u32,

    /// Minimum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[setting(default=-0.2,min=-1.0,max=1.0,incr=0.001)]
    pub mask_dirt_minimum: f32,
    /// Maximum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[setting(default=0.8,min=-1.0,max=1.0,incr=0.001)]
    pub mask_dirt_maximum: f32,
    /// Arbitrary exponent to apply to the dirt gradient.
    #[setting(default = 1.0, min = -5.0, max = 5.0, incr = 0.001, soft_max)]
    pub mask_dirt_exponent: f32,

    /// Minimum dot value for adding sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[setting(default=0.65,min=-1.0,max=1.0,incr=0.001)]
    pub mask_sand_minimum: f32,
    /// Maximum dot value for baking sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[setting(default=1.0,min=-1.0,max=1.0,incr=0.001)]
    pub mask_sand_maximum: f32,
    /// Arbitrary exponent to apply to the sand gradient.
    #[setting(default=2.6,min=-5.0,max=5.0,incr=0.001,soft_max)]
    pub mask_sand_exponent: f32,

    /// XYZ frequency scale when sampling perlin noise for baking into the Alpha channel.
    #[setting(default=Vec3::new(0.75,0.33,0.75),min=0.0,max=2.0,incr=0.001,soft_max)]
    pub mask_perlin_frequency: Vec3,
}

/// Settings for collision generation.
#[derive(Copy, Clone, PartialEq, ExposeSettings)]
#[settings_resource_from(IslandBuilderSettingsCollision, Resource)]
pub struct SettingsCollision {
    /// Distance threshold for vertices to be merged for the collision hull.
    #[setting(
        default = 0.15,
        min = 0.0,
        max = 1.0,
        incr = 0.001,
        soft_max,
        unit = "m"
    )]
    pub vertex_merge_distance: f32,
    /// Angular threshold for decimating triangles used in physics collisions. In degrees.
    /// If zero, mesh decimation will not occur.
    #[setting(
        default = 2.0,
        min = 0.0,
        max = 179.9,
        incr = 0.001,
        soft_max,
        unit = "degrees"
    )]
    pub decimation_angle: f32,
    /// Maximum number of iterations for performing collision mesh decimation.
    /// The mesh will automatically stop decimating if nothing changes after an iteration.
    #[setting(default = 100, min = 0.0, max = 500.0, incr = 1.0, soft_max)]
    pub decimation_iterations: u32,

    /// Stops the decimation if this many triangles or less were removed during the last decimation step.
    /// This makes collision much faster to generate as it avoids chaining many steps with little gain,
    /// at the cost of some determinism and a (very *very* slightly) more optimized mesh.
    ///
    /// Example: scanning a 5000-triangle mesh only to remove 1 edge is a lot of wasted computation time.
    #[setting(default = 8, min = 0.0, max = 24.0, incr = 1.0, soft_max)]
    pub decimation_dropout: u32,
}

/// Tweakable settings for a specific [IslandBuilder].
#[derive(Copy, Clone, PartialEq, ExposeSettings)]
#[settings_resource_from(IslandBuilderSettingsTweaks, Resource)]
pub struct SettingsTweaks {
    /// Seed for noise parameters.
    #[setting(default = 0, min = 0.0, max = 4294967295.0)]
    pub seed: u32,

    pub w_sampling_density: f64,
    pub w_sampling_offset: f64,
    pub w_striation: f64,
    pub w_mask: f64,
}

#[derive(Default)]
pub struct Data {
    settings_voxels: SettingsVoxels,
    settings_mesh: SettingsMesh,
    settings_collision: SettingsCollision,
    tweaks: SettingsTweaks,

    noise_sdf_density: Perlin1D,
    noise_sdf_sampling: Perlin3D,
    noise_striation: Perlin1D,
    noise_mask: Perlin1D,

    shapes: Vec<Shape>,

    bounds: BoundingBox,
    voxels: Option<VolumeData<f32>>,
    mesh_preview: Option<TriangleMesh>,
    mesh_baked: Option<TriangleMesh>,
    hulls: Vec<TriangleMesh>,

    /// Approximate volume of the Island.
    volume: f32,
}

impl Data {
    /// Creates a new data set for building from.
    pub fn new(
        settings_voxels: SettingsVoxels,
        settings_mesh: SettingsMesh,
        settings_collision: SettingsCollision,
        settings_tweaks: SettingsTweaks,
    ) -> Self {
        Self {
            settings_voxels,
            settings_mesh,
            settings_collision,
            tweaks: settings_tweaks,
            noise_sdf_density: Perlin1D::default(),
            noise_sdf_sampling: Perlin3D::default(),
            noise_striation: Perlin1D::default(),
            noise_mask: Perlin1D::default(),
            shapes: vec![],
            bounds: BoundingBox::default(),
            voxels: None,
            mesh_preview: None,
            mesh_baked: None,
            hulls: vec![],
            volume: 0.0,
        }
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn get_bounds(&self) -> BoundingBox {
        self.bounds
    }

    pub fn get_shapes(&self) -> &Vec<Shape> {
        &self.shapes
    }

    pub fn get_mesh_preview(&self) -> Option<&TriangleMesh> {
        self.mesh_preview.as_ref()
    }

    pub fn get_mesh_baked(&self) -> Option<&TriangleMesh> {
        self.mesh_baked.as_ref()
    }

    pub fn get_hulls(&self) -> &Vec<TriangleMesh> {
        self.hulls.as_ref()
    }

    /// Clears all generated data.
    pub fn dirty_voxels(&mut self) {
        self.voxels = None;
        self.mesh_preview = None;
        self.volume = 0.0;
        self.dirty_mesh();
        self.dirty_collision();
        self.bake_bounding_box();
    }

    /// Clears generated mesh data.
    pub fn dirty_mesh(&mut self) {
        self.mesh_baked = None;
    }

    /// Clears generated collision data.
    pub fn dirty_collision(&mut self) {
        self.hulls.clear();
    }

    /// Updates the settings, dirtying the data if changed.
    /// Returns true if changed.
    pub fn set_voxel_settings(&mut self, settings: SettingsVoxels) -> bool {
        if self.settings_voxels != settings {
            self.settings_voxels = settings;
            self.dirty_voxels();

            let frequency = self.settings_voxels.sampling_density_noise_frequency;
            self.noise_sdf_density.frequency = [
                frequency.x as f64,
                frequency.y as f64,
                frequency.z as f64,
                self.tweaks.w_sampling_density,
            ];
            self.noise_sdf_density.amplitude =
                self.settings_voxels.sampling_density_noise_amplitude;

            let frequency = self.settings_voxels.sampling_offset_noise_frequency;
            self.noise_sdf_sampling.frequency = [
                frequency.x as f64,
                frequency.y as f64,
                frequency.z as f64,
                self.tweaks.w_sampling_offset,
            ];
            let amplitude = self.settings_voxels.sampling_offset_noise_amplitude;
            self.noise_sdf_sampling.amplitude =
                [amplitude.x as f64, amplitude.y as f64, amplitude.z as f64];

            let frequency = self.settings_voxels.striation_frequency;
            self.noise_striation.frequency = [
                frequency.x as f64,
                frequency.y as f64,
                frequency.z as f64,
                self.tweaks.w_striation,
            ];
            self.noise_striation.amplitude = self.settings_voxels.striation_amplitude;
            return true;
        }

        false
    }

    /// Updates the settings, dirtying the data if changed.
    /// Returns true if changed.
    pub fn set_mesh_settings(&mut self, settings: SettingsMesh) -> bool {
        if self.settings_mesh != settings {
            self.settings_mesh = settings;
            self.dirty_mesh();

            let frequency = self.settings_mesh.mask_perlin_frequency;
            self.noise_mask.frequency = [
                frequency.x as f64,
                frequency.y as f64,
                frequency.z as f64,
                self.tweaks.w_mask,
            ];

            return true;
        }
        false
    }

    /// Updates the settings, dirtying the data if changed.
    /// Returns true if changed.
    pub fn set_collision_settings(&mut self, settings: SettingsCollision) -> bool {
        if self.settings_collision != settings {
            self.settings_collision = settings;
            self.dirty_collision();

            return true;
        }
        false
    }

    pub fn set_tweaks(&mut self, settings: SettingsTweaks) -> bool {
        if self.tweaks != settings {
            self.tweaks = settings;
            self.dirty_voxels();

            // update noise seeds
            self.noise_sdf_sampling.set_seed(settings.seed);
            self.noise_sdf_sampling.set_seed(settings.seed + 3);
            self.noise_striation.set_seed(settings.seed + 6);
            self.noise_mask.set_seed(settings.seed + 9);

            return true;
        }
        false
    }

    /// Updates the shape list, dirtying the data if changed.
    pub fn set_shapes(&mut self, shapes: Vec<Shape>) -> bool {
        if self.shapes != shapes {
            self.shapes = shapes;
            self.dirty_voxels();
            return true;
        }
        false
    }

    /// Unsets the voxel bake without dirtying.
    pub fn clear_voxels(&mut self) {
        self.voxels = None;
    }

    /// Unsets the mesh preview without dirtying.
    pub fn clear_mesh_preview(&mut self) {
        self.mesh_preview = None;
    }

    /// Unsets the baked mesh without dirtying.
    pub fn clear_mesh_baked(&mut self) {
        self.mesh_baked = None;
    }

    /// Unsets the collision without dirtying.
    pub fn clear_collision(&mut self) {
        self.hulls.clear();
    }

    /// Automatically computes the axis-aligned bounding box for the Island.
    pub fn bake_bounding_box(&mut self) {
        let padding_size: Vec3 =
            self.settings_voxels.voxel_size * self.settings_voxels.voxel_padding as f32;

        let margin = (self.settings_voxels.striation_amplitude
            + self.settings_voxels.sampling_density_noise_amplitude) as f32
            + self
                .settings_voxels
                .sampling_offset_noise_amplitude
                .max_element();

        let bounds = shape_list_bounds(&self.shapes)
            .expand_margin(margin * 2.0)
            .expand_vector(padding_size.abs() * 2.0);

        self.bounds = bounds;
    }

    fn get_dimensions(&self) -> [usize; 3] {
        let approx_cells = self.bounds.size() / self.settings_voxels.voxel_size;
        [
            approx_cells.x.ceil() as usize,
            approx_cells.y.ceil() as usize,
            approx_cells.z.ceil() as usize,
        ]
    }

    /// Returns a voxel grid and an empty grid and a transform matrix.
    #[doc(hidden)]
    pub fn bake_voxels_init(&self) -> (VolumeData<f32>, Mat4) {
        (
            VolumeData::new(1.0f32, self.get_dimensions()),
            Mat4::from_scale_rotation_translation(
                self.settings_voxels.voxel_size,
                Quat::IDENTITY,
                self.bounds.minimum,
            ),
        )
    }

    /// Bakes the voxel data if able.
    pub fn bake_voxels(&mut self) {
        // Voxels already baked or no shapes to work from
        if self.voxels.is_some() || self.shapes.is_empty() {
            return;
        }

        let (mut voxels, transform) = self.bake_voxels_init();

        let mut voxel_workers = voxels.to_workers(
            utils::worker_count(voxels.get_buffer_size(), 16usize).get(),
            false,
        );

        // Sample island SDF in chunks
        let noise_density = &self.noise_sdf_density;
        let noise_sampling = &self.noise_sdf_sampling;
        voxels.data = voxel_workers
            .par_iter_mut()
            .flat_map(|worker| -> Vec<f32> {
                for i in 0..worker.range_width {
                    let [x, y, z] = voxels.delinearize(i + worker.range_min);

                    let mut sample_pos =
                        transform.transform_point3(Vec3::new(x as f32, y as f32, z as f32));
                    sample_pos += noise_sampling.sample(Vec4::from((
                        sample_pos,
                        self.tweaks.w_sampling_offset as f32,
                    )));

                    let sample = sample_shape_list(
                        &self.shapes,
                        sample_pos,
                        self.settings_voxels.sdf_edge_radius,
                    );
                    let add_in = noise_density.sample(Vec4::from((
                        sample_pos,
                        self.tweaks.w_sampling_density as f32,
                    )));

                    worker.data[i] = sample + add_in as f32;
                }

                worker.data.clone()
            })
            .collect();

        if self.settings_voxels.sdf_smooth_iterations > 0 {
            // Perform smoothing blurs, swapping between current and a buffer.
            // DON'T recreate the buffer each time, because it guzzles performance.
            let blur_buffer = VolumeData::new(1.0, self.get_dimensions());

            voxels.blur(
                self.settings_voxels.sdf_smooth_iterations,
                self.settings_voxels.sdf_smooth_radius_voxels as usize,
                self.settings_voxels.sdf_smooth_weight,
                1,
                1.0,
                blur_buffer,
                voxel_workers,
            );
        }

        voxels.noise_add(
            &self.noise_striation,
            transform,
            self.tweaks.w_striation as f32,
        );

        voxels.set_padding(self.settings_voxels.voxel_padding as usize, 10.0);

        self.voxels = Some(voxels);
    }

    /// Bakes a preview mesh if able.
    pub fn bake_preview(&mut self) {
        if self.mesh_preview.is_some() {
            return;
        }

        if let Some(voxels) = &self.voxels {
            let dim = voxels.get_dimensions();

            let grids_x = (dim[0] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;
            let grids_y = (dim[1] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;
            let grids_z = (dim[2] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;

            let grid_count = grids_x * grids_y * grids_z;
            let grid_strides = [1, grids_x, grids_x * grids_y];

            fn linearize_nets(strides: [usize; 3], x: usize, y: usize, z: usize) -> usize {
                x + strides[1].wrapping_mul(y) + strides[2].wrapping_mul(z)
            }

            // Then, allocate our grids
            let mut grids: Vec<[f32; IslandChunkSize::USIZE]> =
                vec![[1.0f32; IslandChunkSize::USIZE]; grid_count];
            let mut grid_offset: Vec<Vec3> = vec![Vec3::ZERO; grid_count];

            let volume_per_voxel = self.settings_voxels.voxel_size.x
                * self.settings_voxels.voxel_size.y
                * self.settings_voxels.voxel_size.z;
            let mut volume: f32 = 0.0;

            // Fill our constant-size grids with voxel data for surface nets
            for x in 0..grids_x {
                for y in 0..grids_y {
                    for z in 0..grids_z {
                        let grid_idx = linearize_nets(grid_strides, x, y, z);
                        let offset = Vec3::new(
                            (x * (VOLUME_MAX_CELLS - 2)) as f32,
                            (y * (VOLUME_MAX_CELLS - 2)) as f32,
                            (z * (VOLUME_MAX_CELLS - 2)) as f32,
                        ) * self.settings_voxels.voxel_size
                            + self.bounds.minimum;
                        grid_offset[grid_idx] = offset;

                        for i in 0usize..IslandChunkSize::USIZE {
                            // Local XYZ coordinate of Surface Nets volume
                            let coord = IslandChunkSize::delinearize(i as u32);
                            // Global index of Voxel Grid
                            let voxels_idx = voxels.linearize(
                                x * (VOLUME_MAX_CELLS - 2) + coord[0] as usize,
                                y * (VOLUME_MAX_CELLS - 2) + coord[1] as usize,
                                z * (VOLUME_MAX_CELLS - 2) + coord[2] as usize,
                            );

                            let sample = voxels.get_linear(voxels_idx);
                            grids[grid_idx][i] = -sample;

                            if sample < 0.0 {
                                volume += volume_per_voxel;
                            }
                        }
                    }
                }
            }

            // Perform Surface Nets algorithm on all grids in parallel, storing corresponding mesh
            let voxel_size = self.settings_voxels.voxel_size;
            let meshes: Vec<Option<TriangleMesh>> = grids
                .par_iter_mut()
                .enumerate()
                .map(|(idx, grid)| -> Option<TriangleMesh> {
                    let mut buffer = SurfaceNetsBuffer::default();
                    surface_nets(
                        grid,
                        &IslandChunkSize {},
                        [0; 3],
                        [(VOLUME_MAX_CELLS - 1) as u32; 3],
                        &mut buffer,
                    );

                    mesh_from_nets(buffer, voxel_size, grid_offset[idx])
                })
                .collect();

            // Now, join all meshes together
            let mut mesh_final = TriangleMesh::default();

            for mesh in meshes.iter().flatten() {
                mesh_final.join(mesh);
            }

            self.volume = volume;
            self.mesh_preview = Some(mesh_final);
        }
    }

    pub fn bake_mesh(&mut self) {
        if self.mesh_baked.is_some() {
            return;
        }

        self.bake_preview();
        if let Some(mut mesh) = self.mesh_preview.clone() {
            mesh.optimize(self.settings_mesh.vertex_merge_distance);
            mesh.bake_normals_smooth();

            let thread_count = utils::thread_count(16);

            // bake ambient occlusion
            let ao = if self.settings_mesh.ao_enabled {
                mesh.get_ambient_occlusion(
                    self.settings_mesh.ao_samples as usize,
                    self.settings_mesh.ao_radius,
                    self.noise_mask.seed(),
                    thread_count,
                )
            } else {
                vec![]
            };

            let mut colors: Vec<Vec4> = Vec::with_capacity(mesh.count_vertices());
            let mut uv1: Vec<Vec2> = Vec::with_capacity(mesh.count_vertices());
            let mut uv2: Vec<Vec2> = Vec::with_capacity(mesh.count_vertices());

            for (idx, position) in mesh.positions.iter().enumerate() {
                let normal = mesh.normals[idx];

                uv1.push(Vec2::new(position.x + position.z, position.y));
                uv2.push(Vec2::new(position.x, position.z));

                let dot = normal.dot(Vec3::Y);
                let mask_dirt = dot
                    .remap(
                        self.settings_mesh.mask_dirt_minimum,
                        self.settings_mesh.mask_dirt_maximum,
                        0.0,
                        1.0,
                    )
                    .powf(self.settings_mesh.mask_dirt_exponent)
                    .clamp(0.0, 1.0);
                let mask_sand = dot
                    .remap(
                        self.settings_mesh.mask_sand_minimum,
                        self.settings_mesh.mask_sand_maximum,
                        0.0,
                        1.0,
                    )
                    .powf(self.settings_mesh.mask_sand_exponent)
                    .clamp(0.0, 1.0);

                let noise = self
                    .noise_mask
                    .sample(Vec4::from((*position, self.tweaks.w_striation as f32)));

                let mut occlusion: f32 = 1.0;
                if self.settings_mesh.ao_enabled {
                    occlusion = glam::FloatExt::lerp(1.0, ao[idx], self.settings_mesh.ao_strength);
                }

                colors.push(Vec4::new(occlusion, mask_dirt, mask_sand, noise as f32));
            }

            mesh.colors = colors;
            mesh.uv1 = Some(uv1);
            mesh.uv2 = Some(uv2);
            self.mesh_baked = Some(mesh);
        }
    }

    pub fn bake_collision(&mut self) {
        if !self.hulls.is_empty() {
            return;
        }

        if let Some(mut mesh) = self.mesh_preview.clone() {
            // Get a list of all union shapes
            let mut shapes = self.shapes.clone();
            shapes.retain(|shape| shape.operation == ShapeOperation::Union);

            if shapes.is_empty() {
                return;
            }

            // Join mesh and merge by distance before splitting into shapes,
            // to help with edge decimation and prevent vertex merging causing issues on corners
            mesh.optimize(self.settings_collision.vertex_merge_distance);

            let mut hulls: Vec<TriangleMesh> = Vec::with_capacity(shapes.len());
            let tri_prealloc = mesh.triangles.len(); // At most, we can hold this many triangles

            // Generate each triangle mesh with our original mesh positions
            for _ in shapes.iter() {
                let trimesh = TriangleMesh::new(
                    Vec::with_capacity(tri_prealloc),
                    mesh.positions.clone(),
                    None,
                    None,
                );

                hulls.push(trimesh);
            }

            // Assign each triangle to the nearest collision hull
            for tri in mesh.triangles.iter() {
                let mut min_dist = f32::INFINITY;
                let mut min_shape_idx = 0;

                // Fetch centerpoint of triangle to use for comparison
                let center = tri.centerpoint(&mesh.positions);

                for (shape_idx, shape) in shapes.iter().enumerate() {
                    // TODO: somehow take Intersection CSG into account when sampling shapes,
                    // so collision shapes that are cut off via intersections,
                    // do not include shapes added after said intersection.

                    let d = shape.sample(center, self.settings_voxels.sdf_edge_radius);
                    if d < min_dist {
                        min_dist = d;
                        min_shape_idx = shape_idx;
                    }
                }

                hulls[min_shape_idx].triangles.push(*tri);
            }

            // Optimize collision meshes in parallel
            hulls.par_iter_mut().for_each(|mesh| {
                if self.settings_collision.decimation_angle > 0.0 {
                    mesh.decimate_planar(
                        self.settings_collision.decimation_angle.to_radians(),
                        self.settings_collision.decimation_iterations,
                        self.settings_collision.decimation_dropout,
                    );
                }

                // Optimize the mesh again after decimation,
                // but don't worry about merging loose vertices
                mesh.optimize(0.0);
            });

            // Remove hulls with an insignificant triangle count
            hulls.retain(|hull| hull.triangles.len() >= 6);

            self.hulls = hulls;
        }
    }
}
