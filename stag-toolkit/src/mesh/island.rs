use crate::math::bounding_box::BoundingBox;
use crate::math::sdf::{Shape, sample_shape_list, shape_list_bounds};
use crate::math::volumetric::VolumeData;
use crate::mesh::nets::mesh_from_nets;
use crate::mesh::trimesh::TriangleMesh;
use fast_surface_nets::{SurfaceNetsBuffer, ndshape::ConstShape, surface_nets};
use glam::{Mat4, Quat, Vec3};
use ndshape::ConstShape3u32;
use rayon::prelude::*;
use std::mem::swap;

const VOLUME_MAX_CELLS: u32 = 48;
const VOLUME_MAX_CELLS_TRIM: u32 = 44;
type IslandChunkSize = ConstShape3u32<VOLUME_MAX_CELLS, VOLUME_MAX_CELLS, VOLUME_MAX_CELLS>;

/// Settings for voxel generation.
#[derive(Copy, Clone, PartialEq)]
pub struct SettingsVoxels {
    /// Number of voxels to pad on each side of the [IslandBuilder] volume.
    pub voxel_padding: u32,
    /// Width/height/depth of a voxel. This is the approximate resolution of the resulting [IslandBuilder] mesh.
    pub voxel_size: Vec3,

    /// Rounding distance to apply to edges of Signed Distance Field primitives.
    pub sdf_edge_radius: f32,
    /// Number of smoothing iterations to apply to voxels immediately after sampling Signed Distance Fields.
    pub sdf_smooth_iterations: u32,
    /// Radius of voxels to include in each smoothing pass applied immediately after sampling Signed Distance Fields.
    pub sdf_smooth_radius_voxels: u32,
    /// Weighting of each smoothing pass applied immediately after sampling Signed Distance Fields.
    pub sdf_smooth_weight: f32,

    /// Frequency scale for striation noise on local X and Z axii.
    pub striation_scale_xz: f32,
    /// Frequency scale for striation noise on local Y axis.
    pub striation_scale_y: f32,
    /// Amplitude of striation noise on local X and Z axii.
    pub striation_amplitude_xz: f32,
    /// Amplitude of striation noise on local Y axis.
    pub striation_amplitude_y: f32,

    /// Number of voxels per worker group.
    /// This is a performance setting and will not affect the output result.
    pub worker_group_size: u32,
}

/// Settings for mesh generation.
#[derive(Copy, Clone, PartialEq)]
pub struct SettingsMesh {
    /// Distance threshold for vertices to be merged for the visual mesh.
    pub mesh_vertex_merge_distance: f32,

    /// Whether to bake Ambient Occlusion to the Red channel.
    /// The Red channel defaults to 1.0 if Ambient Occlusion is not baked.
    pub ao_enabled: bool,
    /// Weighting value for linearly blending a base value of 1.0 with the baked Ambient Occlusion.
    pub ao_strength: f32,

    /// Minimum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_dirt_minimum: f32,
    /// Maximum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_dirt_maximum: f32,
    /// Arbitrary exponent to apply to the dirt gradient.
    pub mask_dirt_exponent: f32,

    /// Minimum dot value for adding sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_sand_minimum: f32,
    /// Maximum dot value for baking sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_sand_maximum: f32,
    /// Arbitrary exponent to apply to the sand gradient.
    pub mask_sand_exponent: f32,

    /// X frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_x: f32,
    /// Y frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_y: f32,
    /// Z frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_z: f32,
}

/// Settings for collision generation.
#[derive(Copy, Clone, PartialEq)]
pub struct SettingsCollision {
    /// Whether to merge collision vertices on non-manifold edges.
    pub collision_merge_nonmanifold_edges: bool,
    /// Whether to perform collision decimation on non-manifold edges.
    pub collision_decimate_nonmanifold_edges: bool,
    /// Distance threshold for vertices to be merged for the collision hull.
    pub collision_vertex_merge_distance: f32,
    /// Angular threshold for decimating triangles used in physics collisions. In degrees.
    /// If zero, mesh decimation will not occur.
    pub collision_decimation_angle: f32,
    /// Maximum number of iterations for performing collision mesh decimation.
    /// The mesh will automatically stop decimating if nothing changes after an iteration.
    pub collision_decimation_iterations: u32,
}

pub struct Data {
    settings_voxels: SettingsVoxels,
    settings_mesh: SettingsMesh,
    settings_collision: SettingsCollision,

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
    ) -> Self {
        Self {
            settings_voxels,
            settings_mesh,
            settings_collision,
            shapes: vec![],
            bounds: BoundingBox::default(),
            voxels: None,
            mesh_preview: None,
            mesh_baked: None,
            hulls: vec![],
            volume: 0.0,
        }
    }

    /// Clears all generated data.
    pub fn dirty_voxels(&mut self) {
        self.voxels = None;
        self.mesh_preview = None;
        self.volume = 0.0;
        self.dirty_mesh();
        self.dirty_collision();
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
    pub fn set_voxel_settings(&mut self, settings: SettingsVoxels) {
        if self.settings_voxels != settings {
            self.settings_voxels = settings;
            self.dirty_voxels();
        }
    }

    /// Updates the settings, dirtying the data if changed.
    pub fn set_mesh_settings(&mut self, settings: SettingsMesh) {
        if self.settings_mesh != settings {
            self.settings_mesh = settings;
            self.dirty_mesh();
        }
    }

    /// Updates the settings, dirtying the data if changed.
    pub fn set_collision_settings(&mut self, settings: SettingsCollision) {
        if self.settings_collision != settings {
            self.settings_collision = settings;
            self.dirty_collision();
        }
    }

    /// Updates the shape list, dirtying the data if changed.
    pub fn set_shapes(&mut self, shapes: Vec<Shape>) {
        if self.shapes != shapes {
            self.shapes = shapes;
            self.dirty_voxels();
        }
    }

    /// Bakes the voxel data if able.
    pub fn bake_voxels(&mut self) {
        // Voxels already baked or no shapes to work from
        if self.voxels.is_some() || self.shapes.is_empty() {
            return;
        }

        let striation_amplitude = Vec3::new(
            self.settings_voxels.striation_amplitude_xz,
            self.settings_voxels.striation_amplitude_y,
            self.settings_voxels.striation_amplitude_xz,
        )
        .abs();
        let padding_size: Vec3 =
            self.settings_voxels.voxel_size * self.settings_voxels.voxel_padding as f32;

        let bounds = shape_list_bounds(&self.shapes)
            .expand_vector(striation_amplitude.abs())
            .expand_vector(padding_size.abs());

        let approx_cells = bounds.size() / self.settings_voxels.voxel_size;
        let dim = [
            approx_cells.x.ceil() as u32,
            approx_cells.y.ceil() as u32,
            approx_cells.z.ceil() as u32,
        ];

        let mut voxels = VolumeData::new(1.0f32, dim);
        let mut voxel_workers = voxels.to_workers(self.settings_voxels.worker_group_size, false);

        let transform = Mat4::from_scale_rotation_translation(
            self.settings_voxels.voxel_size,
            Quat::IDENTITY,
            bounds.minimum,
        );

        // Sample island SDF in chunks
        voxels.data = voxel_workers
            .par_iter_mut()
            .flat_map(|worker| -> Vec<f32> {
                for i in 0u32..worker.range_width {
                    let [x, y, z] = voxels.delinearize(i + worker.range_min);

                    let sample_pos =
                        transform.transform_point3(Vec3::new(x as f32, y as f32, z as f32));
                    let sample = sample_shape_list(&self.shapes, sample_pos);

                    worker.data[i as usize] = sample;
                }

                worker.data.clone()
            })
            .collect();

        if self.settings_voxels.sdf_smooth_iterations > 0 {
            // Perform smoothing blurs, swapping between current and a buffer.
            // DON'T recreate the buffer each time, because it guzzles performance.
            let mut blur_buffer = VolumeData::new(1.0, dim);

            for _i in 0u32..self.settings_voxels.sdf_smooth_iterations {
                voxels.blur(
                    self.settings_voxels.sdf_smooth_radius_voxels,
                    self.settings_voxels.sdf_smooth_weight,
                    self.settings_voxels.worker_group_size,
                    &mut blur_buffer,
                );

                // Swap buffers
                swap(&mut voxels, &mut blur_buffer);

                // Avoid bleeding over edges
                voxels.set_padding(1, 1.0);
            }
        }

        // TODO: add noise

        voxels.set_padding(self.settings_voxels.voxel_padding, 10.0);

        self.bounds = bounds;
        self.voxels = Some(voxels);
    }

    // Bakes a preview mesh if able.
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
            let mut grids: Vec<[f32; IslandChunkSize::USIZE]> = vec![];
            let mut grid_offset: Vec<Vec3> = vec![];
            grids.reserve_exact(grid_count);
            for _ in 0..grid_count {
                grids.push([1.0f32; IslandChunkSize::USIZE]);
                grid_offset.push(Vec3::ZERO);
            }

            let volume_per_voxel = self.settings_voxels.voxel_size.x
                * self.settings_voxels.voxel_size.y
                * self.settings_voxels.voxel_size.z;
            let mut volume: f32 = 0.0;

            for x in 0..grids_x {
                for y in 0..grids_y {
                    for z in 0..grids_z {
                        let grid_idx = linearize_nets(grid_strides, x, y, z);
                        let offset = Vec3::new(
                            ((x as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                            ((y as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                            ((z as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                        ) * self.settings_voxels.voxel_size
                            + self.bounds.minimum;
                        grid_offset[grid_idx] = offset;

                        for i in 0usize..IslandChunkSize::USIZE {
                            // Local XYZ coordinate of Surface Nets volume
                            let coord = IslandChunkSize::delinearize(i as u32);
                            // Global index of Voxel Grid
                            let voxels_idx = voxels.linearize(
                                (x as u32) * (VOLUME_MAX_CELLS - 2) + coord[0],
                                (y as u32) * (VOLUME_MAX_CELLS - 2) + coord[1],
                                (z as u32) * (VOLUME_MAX_CELLS - 2) + coord[2],
                            );

                            let sample = voxels.get_linear(voxels_idx as usize);
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
                        [VOLUME_MAX_CELLS - 1; 3],
                        &mut buffer,
                    );

                    mesh_from_nets(buffer, voxel_size, grid_offset[idx])
                })
                .collect();

            // Now, join all meshes together
            let mut mesh_final = TriangleMesh::default();

            for mesh_option in meshes.iter() {
                if let Some(mesh) = mesh_option {
                    mesh_final.join(mesh);
                }
            }

            self.volume = volume;
            self.mesh_preview = Some(mesh_final);
        }
    }
}
