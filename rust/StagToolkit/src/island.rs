use core::f32;
use godot::classes::ImporterMesh;
use rayon::prelude::*;
use std::f32::consts::PI;
use std::mem::swap;

use crate::{
    math::{
        sdf::{ShapeOperation, sample_shape_list},
        types::{ToColor, ToVector2, ToVector3, Vec3Godot},
        volumetric::{PerlinField, VolumeData},
    },
    mesh::{
        godot::{GodotSurfaceArrays, GodotWhitebox},
        nets::mesh_from_nets,
        trimesh::{TriangleMesh, TriangleOperations},
    },
};
use fast_surface_nets::{
    SurfaceNetsBuffer,
    ndshape::{ConstShape, ConstShape3u32},
    surface_nets,
};
use glam::{FloatExt, Mat4, Vec2, Vec3, Vec4};
use godot::{
    classes::{
        ArrayMesh, CollisionShape3D, ConvexPolygonShape3D, Material, MeshInstance3D,
        ProjectSettings, RigidBody3D, WorkerThreadPool, mesh::PrimitiveType,
        physics_server_3d::BodyAxis,
    },
    prelude::*,
};

/// The node group IslandBuilder nodes should be stored in.
pub const GROUP_NAME: &str = "StagToolkit_IslandBuilder";

const VOLUME_MAX_CELLS: u32 = 48;
const VOLUME_MAX_CELLS_TRIM: u32 = 44;
const VOLUME_WORK_GROUP_SIZE: u32 = 48 * 48 * 48;
type IslandChunkSize = ConstShape3u32<VOLUME_MAX_CELLS, VOLUME_MAX_CELLS, VOLUME_MAX_CELLS>;

/// Container for working data about a given island.
#[derive(Clone)]
pub struct IslandBuildData {
    whitebox: GodotWhitebox,
    noise: PerlinField,

    // CONFIGURATION //
    cell_padding: u32,
    cell_size: f32,
    smoothing_repetitions: u32,
    smoothing_radius_voxels: u32,
    smoothing_weight: f32,
    noise_amplitude: f32,
    noise_w: f64,
    /// Distance threshold for triangles to be merged together and collapsed for the visual mesh.
    mesh_merge_distance: f32,
    /// Distance threshold for triangles to be merged together and collapsed for the physics collisions.
    collision_merge_distance: f32,
    /// Angle threshold for decimating triangles used in physics collisions. In radians.
    collision_decimate_angle: f32,
    /// Max number of iterations for performing decimation.
    collision_decimate_iterations: i32,
    mask_range_dirt: Vec2,
    mask_range_sand: Vec2,
    mask_power_sand: f32,
    mask_perlin_scale: Vec3,

    /// Mesh generated via surface nets.
    /// Stored as option in case it was not already generated.
    mesh: Option<TriangleMesh>,
    /// Whether this mesh has been optimized since generation.
    optimized: bool,

    // OUTPUTS //
    /// Estimated volume of island.
    volume: f32,
}
impl Default for IslandBuildData {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandBuildData {
    /// Generate a new set of IslandBuildData for working with.
    pub fn new() -> Self {
        Self {
            whitebox: GodotWhitebox::new(),
            noise: PerlinField::new(0, 1, 2, 1.0),

            cell_padding: 2,
            cell_size: 0.275,
            smoothing_repetitions: 3,
            smoothing_radius_voxels: 2,
            smoothing_weight: 0.5,
            noise_amplitude: 0.3,
            noise_w: 1.0,
            mesh_merge_distance: 0.04,
            collision_merge_distance: 0.15,
            collision_decimate_angle: PI / 60.0,
            collision_decimate_iterations: 100,
            mask_range_dirt: Vec2::new(-0.1, 0.8),
            mask_range_sand: Vec2::new(0.7, 1.0),
            mask_power_sand: 3.0,
            mask_perlin_scale: Vec3::new(0.75, 0.33, 0.75),

            mesh: None,
            optimized: false,

            volume: 0.0,
        }
    }

    /// Perform Naive Surface Nets algorithm on geometry.
    fn nets(&mut self) {
        let aabb: Aabb = self
            .whitebox
            .get_aabb()
            .grow(self.noise_amplitude.abs() + self.cell_size * (self.cell_padding as f32));
        let minimum_bound: Vec3 = aabb.position.to_vector3();
        let aabb_size: Vec3 = aabb.size.to_vector3();

        // Transformation matrix for quickly moving points
        let cell_size: Vec3 = Vec3::splat(self.cell_size);
        let trans: Mat4 = Mat4::from_translation(minimum_bound) * Mat4::from_scale(cell_size);

        // Prepare volume estimates
        let mut volume: f32 = 0.0;
        let volume_per_voxel = self.cell_size * self.cell_size * self.cell_size;

        let shapes = self.whitebox.get_shapes();
        let approx_cells = aabb_size / Vec3::splat(self.cell_size);
        let dim = [
            approx_cells.x.ceil() as u32,
            approx_cells.y.ceil() as u32,
            approx_cells.z.ceil() as u32,
        ];

        // If voxel data was not already initialized, initialize it
        let mut voxels = VolumeData::new(1.0f32, dim);

        // Sample SDF at every voxel
        let mut voxel_workers = voxels.to_workers(VOLUME_WORK_GROUP_SIZE, false);

        // Sample island SDF in chunks
        voxels.data = voxel_workers
            .par_iter_mut()
            .flat_map(|worker| -> Vec<f32> {
                for i in 0u32..worker.range_width {
                    let [x, y, z] = voxels.delinearize(i + worker.range_min);

                    let sample_pos: Vec3 =
                        trans.transform_point3(Vec3::new(x as f32, y as f32, z as f32));
                    let sample = sample_shape_list(shapes, sample_pos);

                    worker.data[i as usize] = sample;
                }

                worker.data.clone()
            })
            .collect();

        // Factor noise in
        voxels.noise_add(&self.noise, trans, self.noise_w, self.noise_amplitude);

        if self.smoothing_repetitions > 0 {
            // Perform smoothing blurs, swapping between current and a buffer.
            // DON'T recreate the buffer each time, because it guzzles performance.
            let mut blur_buffer = VolumeData::new(1.0, dim);

            for _i in 0u32..self.smoothing_repetitions {
                voxels.blur(
                    self.smoothing_radius_voxels,
                    self.smoothing_weight,
                    VOLUME_WORK_GROUP_SIZE,
                    &mut blur_buffer,
                );

                // Swap buffers
                swap(&mut voxels, &mut blur_buffer);
            }
        }

        // Perform padding
        voxels.set_padding(self.cell_padding, 10.0);

        // NOW, convert voxel data to buffers for Surface Nets

        // First, figure out how many grids we need...
        let grids_x = (dim[0] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;
        let grids_y = (dim[1] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;
        let grids_z = (dim[2] as f32 / VOLUME_MAX_CELLS_TRIM as f32).ceil() as usize;
        let gridcount = grids_x * grids_y * grids_z;
        let grid_strides = [1, grids_x, grids_x * grids_y];

        fn linearize_nets(strides: [usize; 3], x: usize, y: usize, z: usize) -> usize {
            x + strides[1].wrapping_mul(y) + strides[2].wrapping_mul(z)
        }

        // Then, allocate our grids
        let mut grids: Vec<[f32; IslandChunkSize::USIZE]> = vec![];
        let mut grid_offset: Vec<Vec3> = vec![];
        grids.reserve_exact(gridcount);
        for _ in 0..gridcount {
            grids.push([1.0f32; IslandChunkSize::USIZE]);
            grid_offset.push(Vec3::ZERO);
        }

        // Begin filling our grids
        for x in 0..grids_x {
            for y in 0..grids_y {
                for z in 0..grids_z {
                    let grid_idx = linearize_nets(grid_strides, x, y, z);
                    let offset = Vec3::new(
                        ((x as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                        ((y as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                        ((z as u32) * (VOLUME_MAX_CELLS - 2)) as f32,
                    ) * cell_size
                        + minimum_bound;
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

        self.volume = volume;
        self.optimized = false; // This new mesh has not been optimized yet

        // Perform Surface Nets algorithm on all grids in parallel, storing corresponding mesh
        let mut meshes: Vec<Option<TriangleMesh>> = grids
            .par_iter_mut()
            .enumerate()
            .map(|(idx, grid)| -> Option<TriangleMesh> {
                // Perform surface nets
                let mut buffer = SurfaceNetsBuffer::default();
                surface_nets(
                    grid,
                    &IslandChunkSize {},
                    [0; 3],
                    [VOLUME_MAX_CELLS - 1; 3],
                    &mut buffer,
                );

                // Parse and store result
                mesh_from_nets(buffer, cell_size, grid_offset[idx])
            })
            .collect();

        // Now, join all meshes together
        let mesh = meshes.iter_mut().reduce(|a, b| {
            // If we have a mesh on left side
            if let Some(amesh) = a {
                // ...and a mesh on right side...
                if let Some(bmesh) = b {
                    // ...join them!
                    amesh.join(bmesh);
                    return a;
                }
                return a;
            }
            b
        });

        // Unwrap result once
        if let Some(m) = mesh {
            self.mesh = m.clone();
        } else {
            self.mesh = None;
        }

        if self.mesh.is_none() {
            godot_warn!("IslandBuilder: Generated mesh buffer was empty.");
        }
    }

    /// Optimizes the mesh, if it exists and has not been optimized already.
    fn optimize_mesh(&mut self) {
        // Mesh has already been optimized, or does not exist. No-op.
        if self.optimized || self.mesh.is_none() {
            return;
        }

        if let Some(mesh) = self.mesh.as_mut() {
            mesh.optimize(self.mesh_merge_distance);

            self.optimized = true;
        }
    }

    /// Returns a SurfaceArrays object containing preview mesh data.
    /// Returns `None` if no mesh is currently stored.
    fn get_mesh(&self) -> Option<GodotSurfaceArrays> {
        // let positions = &self.mesh.as_ref().unwrap().positions;
        // let tris = convex_hull(positions);
        // let m = TriangleMesh::from_triangles(tris, positions.to_vec());

        // return Some(GodotSurfaceArrays::from_trimesh(&m));

        self.mesh.as_ref().map(GodotSurfaceArrays::from_trimesh)
    }
    /// Fetches the preview mesh and bakes additional data for shading into it.
    /// Returns `None` if no mesh is currently stored.
    fn get_mesh_baked(&self) -> Option<GodotSurfaceArrays> {
        let arr = self.get_mesh();
        match arr {
            Some(mut x) => {
                // We know that there is a mesh, because get_mesh returned data
                if let Some(mesh) = self.mesh.as_ref() {
                    let buffer_len = mesh.count_vertices();

                    let mut colors: Vec<Vec4> = vec![];
                    let mut uv1: Vec<Vec2> = vec![];
                    let mut uv2: Vec<Vec2> = vec![];
                    colors.reserve_exact(buffer_len);
                    uv1.reserve_exact(buffer_len);
                    uv2.reserve_exact(buffer_len);

                    for idx in 0..buffer_len {
                        let pos = mesh.positions[idx];
                        let norm = mesh.normals[idx];

                        // Bake normals
                        uv1.push(Vec2::new(pos.x + pos.z, pos.y));
                        uv2.push(Vec2::new(pos.x, pos.z));

                        // TODO: Bake ambient occlusion, somehow

                        // Dot product with up vector for masking, then build dirt and sand masks
                        let dot = norm.dot(Vec3::Y);
                        let mask_dirt = dot
                            .remap(self.mask_range_dirt.x, self.mask_range_dirt.y, 0.0, 1.0)
                            .clamp(0.0, 1.0);
                        let mask_sand = dot
                            .remap(self.mask_range_sand.x, self.mask_range_sand.y, 0.0, 1.0)
                            .clamp(0.0, 1.0)
                            .powf(self.mask_power_sand);

                        // Sample noise and store it in mesh for extra variation
                        let noise_sample = self.noise.sample(pos * self.mask_perlin_scale, 100.0);
                        let noise = (noise_sample.x + noise_sample.y + noise_sample.z)
                            .remap(-3.0, 3.0, 0.0, 1.0);

                        // Store masks in vertex color data
                        colors.push(Vec4::new(1.0, mask_dirt, mask_sand, noise));
                    }

                    x.set_colors(colors.to_color());
                    x.set_uv1(uv1.to_vector2());
                    x.set_uv2(uv2.to_vector2());

                    return Some(x);
                }
                None
            }
            None => None,
        }
    }

    /// Iterates through all positions on the mesh, assigning them to nearby collision hulls.
    /// Returns an empty vector if no hulls were generated.
    fn get_hulls(&self) -> Vec<TriangleMesh> {
        let mesh: &TriangleMesh = match &self.mesh {
            Some(trimesh) => trimesh,
            None => return vec![],
        };

        let mut hulls: Vec<TriangleMesh> = vec![];

        // Fetch the shape list, retaining only Union shapes
        let mut shapes = self.whitebox.get_shapes().clone();
        shapes.retain(|shape| shape.operation == ShapeOperation::Union);

        // Allocate a collision hull for each shape
        hulls.reserve_exact(shapes.len());

        // Number of triangles to pre-allocate, based on mesh + shape size
        let tri_prealloc = mesh.triangles.len() / shapes.len();

        for _ in shapes.iter() {
            // Pre-allocate some space for triangles, and build trimesh
            let trimesh = TriangleMesh::new(
                Vec::with_capacity(tri_prealloc),
                mesh.positions.clone(),
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

                let d = shape.sample(center);
                if d < min_dist {
                    min_dist = d;
                    min_shape_idx = shape_idx;
                }
            }

            hulls[min_shape_idx].triangles.push(*tri);
        }

        // Optimize collision meshes in parallel
        hulls.par_iter_mut().for_each(|mesh| {
            if self.collision_decimate_angle > 0.0 {
                mesh.decimate_planar(
                    self.collision_decimate_angle,
                    self.collision_decimate_iterations,
                    8,
                );
            }

            mesh.optimize(self.collision_merge_distance);
        });

        // Remove hulls with an insignificant amount of triangles.
        hulls.retain(|hull| hull.triangles.len() >= 6);

        hulls
    }
}

// GODOT CLASSES //

/// Navigation properties for Abyss islands.
/// These are utilized for A* pathing with Character AI.
#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct NavIslandProperties {
    #[export]
    #[init(val=Aabb::new(Vector3::ZERO, Vector3::ZERO))]
    aabb: Aabb,
    #[export]
    #[init(val=Vector3::ZERO)]
    center: Vector3,
    #[export]
    #[init(val = 5.0)]
    radius: f32,
    #[export]
    #[init(val = 1.0)]
    surface_flatness: f32,
    base: Base<Resource>,
}

/// The `IslandBuilder` is used to convert whitebox geometry into game-ready islands using procedural geometry.
/// To create a mesh, add CSGBox and CSGSphere nodes as descendants to the IslandBuilder,
/// then `serialize()`, `net()` and fetch your related data.
#[derive(GodotClass)]
#[class(base=Node3D,tool)]
pub struct IslandBuilder {
    data: IslandBuildData,

    /// Node to target for storing generation output, and modifying data.
    /// If empty or target is not found, uses this node instead.
    #[export]
    output_to: NodePath,

    /// Number of cells to pad on each side of the IslandBuilder volume.
    #[export(range = (0.0, 6.0, or_greater))]
    generation_cell_padding: u32,
    /// Number of cells to pad on each side of the IslandBuilder volume.
    #[export(range = (0.01, 1.0, 0.001, or_greater, suffix="m"))]
    generation_cell_size: f32,
    /// Number of times box-blur should be applied to the volume.
    #[export(range = (0.0, 20.0, or_greater))]
    generation_smoothing_iterations: u32,
    /// Radius (in cells) that box-blur smoothing should utilize.
    #[export(range = (1.0, 4.0, or_greater))]
    generation_smoothing_radius_voxels: u32,
    /// What proportion of the smoothing should be used.
    #[export(range = (0.0, 1.0))]
    generation_smoothing_weight: f32,
    /// Corner radius, in meters, to use around boxes.
    #[export(range = (0.0, 2.0, or_greater, suffix="m"))]
    generation_edge_radius: f32,
    /// Noise seed to use for generation.
    #[export(range = (0.0, 1000.0, or_greater))]
    noise_seed: u32,
    /// Noise frequency.
    #[export]
    noise_frequency: f32,
    /// Noise amplitude, in meters.
    /// This value is directly added to the SDF result in the volume pass.
    /// Advized to keep below 1.0 meter.
    #[export(range = (0.0, 1.0, or_greater, suffix="m"))]
    noise_amplitude: f32,
    /// W position for sampling noise.
    #[export]
    noise_w: f64,

    /// Distance threshold for triangles to be merged together and collapsed for the visual mesh.
    #[export(range = (0.0, 0.5, 0.001, or_greater, suffix="m"))]
    mesh_merge_distance: f32,
    /// Distance threshold for triangles to be merged together and collapsed for the physics collisions.
    #[export(range = (0.0, 1.0, 0.001, or_greater, suffix="m"))]
    collision_merge_distance: f32,
    /// Angular threshold for decimating triangles used in physics collisions. In degrees.
    /// If zero, mesh decimation will not occur.
    #[export(range = (0.0, 179.9, 0.001, or_greater, degrees))]
    collision_decimation_angle: f32,
    /// Maximum number of iterations for performing collision mesh decimation.
    /// If the mesh has not changed after an iteration, not all iterations will be used.
    #[export(range = (0.0, 500.0, 1.0, or_greater))]
    collision_decimate_iterations: i32,

    /// Approximate physical density of material to use when calculating mass.
    /// Kilograms per meter cubed.
    #[export(range = (0.01,50.0,0.01, or_greater, suffix="kg/m³"))]
    gameplay_density: f32,
    /// Approximate health density of material to use when calculating island health.
    /// Hit Points per meter cubed.
    #[export(range = (0.001,10.0,0.001, or_greater, suffix="HP/m³"))]
    gameplay_health_density: f32,

    /// Material to use in final product.
    #[export]
    material_baked: Option<Gd<Material>>,
    /// Material to use in preview modes.
    #[export]
    material_preview: Option<Gd<Material>>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for IslandBuilder {
    /// Initializes the IslandBuilder.
    fn init(base: Base<Node3D>) -> Self {
        Self {
            data: IslandBuildData::new(),
            output_to: NodePath::from("."),
            generation_cell_padding: 2,
            generation_cell_size: 0.275,
            generation_smoothing_iterations: 3,
            generation_smoothing_radius_voxels: 2,
            generation_smoothing_weight: 0.5,
            generation_edge_radius: 1.0,
            noise_seed: 0,
            noise_frequency: 0.335,
            noise_amplitude: 0.3,
            noise_w: 1.0,
            mesh_merge_distance: 0.04,
            collision_merge_distance: 0.15,
            collision_decimation_angle: 2.0,
            collision_decimate_iterations: 100,
            gameplay_density: 23.23,
            gameplay_health_density: 1.0,
            material_baked: None,
            material_preview: None,
            base,
        }
    }

    /// Called upon ready notification.
    fn ready(&mut self) {
        // Add the IslandBuilder to a node group for indexing
        self.base_mut()
            .add_to_group_ex(GROUP_NAME)
            .persistent(true)
            .done();
    }

    // Modifies property list of node. Godot 4.3 and onward only
    // fn get_property_list(&mut self) -> Vec<PropertyInfo> { return vec![] }
}

#[godot_api]
impl IslandBuilder {
    // Signals //

    /// Emitted when build data is applied. Useful for awaiting in multi-threaded contexts.
    #[signal]
    pub fn applied_build_data();

    // Getters //

    /// Computes and returns the Axis-Aligned Bounding Box with the current serialization.
    #[func]
    pub fn get_aabb(&self) -> Aabb {
        self.data.whitebox.get_aabb()
    }

    /// Returns the pre-computed volume of the SDF. Returns 0 if not pre-computed.
    #[func]
    pub fn get_volume(&self) -> f32 {
        self.data.volume
    }

    /// Returns the number of currently serialized shapes.
    #[func]
    pub fn get_shape_count(&self) -> i32 {
        self.data.whitebox.get_shape_count() as i32
    }

    /// Checks if there is valid IslandBuilderData for working with.
    #[func]
    pub fn is_precomputed(&self) -> bool {
        self.data.mesh.is_some()
    }

    /// Returns an estimation of the AABB for the island based off the serialized whitebox,
    /// factoring noise into account.
    #[func]
    fn estimate_aabb(&self) -> Aabb {
        self.data
            .whitebox
            .get_aabb()
            .expand(Vec3Godot::splat(self.noise_amplitude))
    }

    // Setters //

    // Build Steps //

    /// Applies Godot settings to corresponding whitebox and mesh data.
    fn apply_settings(&mut self) {
        // Apply whitebox settings
        self.data.cell_padding = self.generation_cell_padding;
        self.data.cell_size = self.generation_cell_size;
        self.data.smoothing_repetitions = self.generation_smoothing_iterations;
        self.data.smoothing_radius_voxels = self.generation_smoothing_radius_voxels;
        self.data.smoothing_weight = self.generation_smoothing_weight;
        self.data.whitebox.default_edge_radius = self.generation_edge_radius;

        // Force a mesh re-optimize
        if self.data.mesh_merge_distance != self.mesh_merge_distance {
            self.data.optimized = false;
            self.data.mesh_merge_distance = self.mesh_merge_distance;
        }
        self.data.collision_merge_distance = self.collision_merge_distance;
        self.data.collision_decimate_angle = self.collision_decimation_angle.to_radians();
        self.data.collision_decimate_iterations = self.collision_decimate_iterations;

        // Check if random seeds have changed
        // Don't bother setting seed if they haven't changed
        let (seed_x, seed_y, seed_z) = self.data.noise.get_seed();
        let nseed_x: u32 = self.noise_seed;
        let nseed_y: u32 = self.noise_seed + 1;
        let nseed_z: u32 = self.noise_seed + 2;
        if seed_x != nseed_x || seed_y != nseed_y || seed_z != nseed_z {
            self.data.noise.set_seed(nseed_x, nseed_y, nseed_z);
        }

        // Apply noise settings
        self.data.noise.frequency = [
            self.noise_frequency as f64,
            self.noise_frequency as f64,
            self.noise_frequency as f64,
            self.noise_frequency as f64,
        ];
        self.data.noise_amplitude = self.noise_amplitude;
        self.data.noise_w = self.noise_w;
    }

    /// Reads and stores children CSG shapes as whitebox geometry for processing.
    /// Supports Union, Intersection and Subtraction.
    ///
    /// Supported shapes include: [CSGBox3D], [CSGSphere3D], [CSGCylinder3D], and [CSGTorus3D].
    ///
    /// **Note**: The IslandBuilder only utilizes convex hulls in order to support Godot's physics implmentation.
    /// A corresponding hull is generated for each of the provided provided CSG shape Unions.
    /// Shapes with holes punctured through them via Subtract operations--or the [CSGTorus3D] node--will
    /// *not* generate accurate collisions, because they are concave.
    /// These operations are still permitted for cosmetic usage.
    #[func]
    pub fn serialize(&mut self) {
        self.data.whitebox.clear();
        self.apply_settings();
        self.data.whitebox.serialize_from(self.base().to_godot());
    }

    /// Performs Surface Nets Algorithm, storing it in the IslandBuilderData for future use.
    /// Returns true if the generated mesh is empty.
    #[func]
    pub fn net(&mut self) -> bool {
        self.apply_settings();
        self.data.nets();
        self.data.mesh.is_none()
    }

    /// Optimizes the mesh, if it has not been optimized already, for baking steps.
    #[func]
    pub fn optimize(&mut self) {
        self.data.optimize_mesh();
    }

    /// Returns an unoptimized triangle mesh for previewing with no extra information baked-in.
    /// Returns an empty mesh if not pre-computed.
    #[func]
    pub fn generate_preview_mesh(&self, recycle_mesh: Option<Gd<ArrayMesh>>) -> Gd<ArrayMesh> {
        let mut mesh: Gd<ArrayMesh>;
        match recycle_mesh {
            Some(recycle) => {
                mesh = recycle;
                mesh.clear_surfaces();
            }
            _ => {
                mesh = ArrayMesh::new_gd();
            }
        }
        let arrs_opt = self.data.get_mesh();

        match arrs_opt {
            Some(arrs) => {
                mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrs.get_surface_arrays());
                mesh.surface_set_name(0, "island");
                // Add a material, if valid
                if let Some(mat) = &self.material_preview {
                    mesh.surface_set_material(0, mat);
                }

                mesh
            }
            _ => mesh,
        }
    }
    /// Bakes and returns a triangle mesh with vertex colors, UVs, and LODs.
    /// Returns an empty mesh if not pre-computed.
    /// Optimizes the mesh data beforehand, if not already optimized.
    #[func]
    pub fn generate_baked_mesh(&mut self) -> Gd<ArrayMesh> {
        self.apply_settings();
        self.optimize();

        let arrs_opt = self.data.get_mesh_baked();
        match arrs_opt {
            Some(arrs) => {
                let mut importer = ImporterMesh::new_gd();
                importer.add_surface(PrimitiveType::TRIANGLES, &arrs.get_surface_arrays());
                importer.generate_lods(25.0, 60.0, &varray![]);
                importer.set_surface_name(0, "island");

                // If we have a material, assign it!
                if let Some(mat) = &self.material_baked {
                    importer.set_surface_material(0, mat);
                }

                // If we were able to successfully generate a mesh, return it
                if let Some(mesh) = importer.get_mesh() {
                    return mesh;
                }

                // If LOD generation fails, fall back to a plain array mesh
                godot_warn!("IslandBuilder: LOD generation failed. Returning island with no LODs.");

                let mut mesh = ArrayMesh::new_gd();
                mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrs.get_surface_arrays());
                mesh.surface_set_name(0, "island");

                if let Some(mat) = &self.material_baked {
                    mesh.surface_set_material(0, mat);
                }

                mesh
            }
            _ => ArrayMesh::new_gd(),
        }
    }
    /// Computes and returns a list of collision hulls for the IslandBuilder shape.
    /// Returns an empty array if not pre-computed.
    /// Optimizes the mesh data beforehand, if not already optimized.
    #[func]
    pub fn generate_collision_hulls(&mut self) -> Array<Gd<ConvexPolygonShape3D>> {
        self.apply_settings();
        self.optimize();

        let hull_pts = self.data.get_hulls();

        let mut hulls = Array::<Gd<ConvexPolygonShape3D>>::new();
        for hull in hull_pts.iter() {
            let mut shape = ConvexPolygonShape3D::new_gd();

            // Fetch remaining positions from the hull
            let pos: PackedVector3Array = hull.positions.clone().to_vector3();

            shape.set_points(&pos);
            hulls.push(&shape);
        }

        hulls
    }
    /// Computes and returns the navigation properties of the island.
    /// Properties will be zero'd if not pre-computed.
    #[func]
    fn generate_navigation_properties(&self) -> Gd<NavIslandProperties> {
        let mut props = NavIslandProperties::new_gd();
        let aabb = self.get_aabb();

        let size: Vec3 = aabb.size.to_vector3();
        let rad: f32 = (size * Vec3::new(1.0, 0.0, 1.0)).length() / 2.0;

        {
            let mut props_mut = props.bind_mut();
            props_mut.aabb = aabb;
            props_mut.radius = rad;
            props_mut.center =
                (aabb.center() * Vec3Godot::new(1.0, 0.0, 1.0)) + (aabb.support(Vec3Godot::UP));
        }

        props
    }

    /// Applies the given mesh to the island output.
    #[func]
    fn apply_mesh(&mut self, mesh: Gd<ArrayMesh>) {
        self.target_mesh().set_mesh(&mesh);
    }

    /// Applies the given list of collision shapes to the island output.
    /// Sets up physics properties on RigidBodies when possible.
    #[func]
    fn apply_collision_hulls(&mut self, hulls: Array<Gd<ConvexPolygonShape3D>>, volume: f32) {
        let mut target = self.target();

        // Remove all current collider children
        for child in target.get_children().iter_shared() {
            // If this is a CollisionShape3D, destroy it
            match child.try_cast::<CollisionShape3D>() {
                Ok(mut collision) => {
                    target.remove_child(&collision);
                    collision.queue_free();
                }
                Err(_as_node_again) => {}
            }
        }

        // Fetch color for debug drawing
        let settings = ProjectSettings::singleton();
        let debug_color_variant: Variant = settings
            .get_setting_ex("addons/stag_toolkit/island_builder/collision_color")
            .default_value(&Variant::from(Color::from_rgba(1.0, 0.0, 0.667, 1.0)))
            .done();
        let debug_color: Color;

        // Ensure variant is of proper type
        if let Ok(color) = debug_color_variant.try_to::<Color>() {
            debug_color = color;
        } else {
            // Otherwise, use default
            debug_color = Color::from_rgba(1.0, 0.0, 0.667, 1.0);
        }

        // Get collision hulls
        for (idx, hull) in hulls.iter_shared().enumerate() {
            let mut shape = CollisionShape3D::new_alloc();
            shape.set_shape(&hull);
            shape.set_name(&format!("collis{0}", idx));
            shape.set_debug_color(debug_color); // Apply debug draw color

            target.add_child(&shape); // Add shape to scene

            // Set shape owner so it is included and saved within the scene
            if let Some(tree) = target.get_tree() {
                if let Some(root) = tree.get_edited_scene_root() {
                    shape.set_owner(&root);
                }
            }
        }

        // Apply physics properties
        if let Ok(mut rigid) = target.clone().try_cast::<RigidBody3D>() {
            rigid.set_mass(volume * self.gameplay_density);
            rigid.set_axis_lock(BodyAxis::ANGULAR_X, true);
            rigid.set_axis_lock(BodyAxis::ANGULAR_Z, true);
            rigid.set_axis_lock(BodyAxis::LINEAR_Y, true);
        }

        // If possible, apply maximum health too
        if let Some(mut p) = target.clone().get_parent() {
            if p.has_method("set_maximum_health") {
                p.call(
                    "set_maximum_health",
                    &[Variant::from(volume * self.gameplay_health_density)],
                );
            }
        }
    }

    /// Applies the given [NavIslandProperties] to the island output or its corresponding parent, if possible.
    ///
    /// Searches specifically for an object method `set_navigation_properties(...)` with a single [NavIslandProperties] argument.
    #[func]
    fn apply_navigation_properties(&mut self, props: Gd<NavIslandProperties>) {
        let mut p = self.target();

        // Apply navigation properties if target has them available
        if p.has_method("set_navigation_properties") {
            p.callv("set_navigation_properties", &varray![Variant::from(props)]);
            return;
        }

        // Otherwise, apply navigation properties to target's parent
        if let Some(mut parent) = p.get_parent() {
            if parent.has_method("set_navigation_properties") {
                parent.callv("set_navigation_properties", &varray![Variant::from(props)]);
            }
        }
    }

    /// Fetches the output node for this IslandBuilder.
    /// If no output is specified, uses this node instead.
    #[func]
    fn target(&mut self) -> Gd<Node> {
        let target = self.base().get_node_or_null(&self.output_to);
        match target {
            Some(node) => node,
            None => self.base_mut().clone().upcast::<Node>(),
        }
    }

    /// Fetches the output mesh for this IslandBuilder.
    /// Creates one if none was found.
    /// If the mesh is newly created, its render layers are specified by
    /// `"addons/stag_toolkit/island_builder/render_layers"`
    /// in the Project Settings.
    #[func]
    fn target_mesh(&mut self) -> Gd<MeshInstance3D> {
        let mut target = self.target();

        // Find a mesh
        for child in target.get_children().iter_shared() {
            match child.try_cast::<MeshInstance3D>() {
                Ok(mesh) => return mesh,
                Err(_as_node) => {}
            }
        }

        // If no mesh found, create one
        let mut mesh = MeshInstance3D::new_alloc();

        // Get render layers mask from Project Settings
        let settings = ProjectSettings::singleton();
        let mask = settings
            .get_setting_ex("addons/stag_toolkit/island_builder/render_layers")
            .default_value(&Variant::from(5))
            .done();
        mesh.set_layer_mask(mask.to());

        // Add mesh to scene
        mesh.set_name("mesh_island");
        target.add_child(&mesh);

        // Ensure scene owns mesh object
        // If no scene tree found, instead use target node as owner
        if let Some(tree) = target.get_tree() {
            mesh.set_owner(&tree.get_edited_scene_root().unwrap_or(target));
        }

        mesh
    }

    /// Destroys all MeshInstance3D and CollisionShape3D nodes directly under the output node.
    /// Clears all working data: The IslandBuilder will have to be re-serialized and netted.
    /// Removes PackedScene references on the IslandBuilder's target node.
    #[func]
    fn destroy_bakes(&mut self) {
        self.data.whitebox.clear();
        self.data.mesh = None;
        self.data.optimized = false;

        let mut out = self.target();

        out.set_scene_file_path(""); // Clear scene file path

        // Iterate over all children.
        for child in out.get_children().iter_shared() {
            // If this is a MeshInstance3D, destroy it
            match child.try_cast::<MeshInstance3D>() {
                Ok(mut mesh) => {
                    mesh.set_mesh(Gd::null_arg());
                }
                Err(as_node) => {
                    // OR, if this is a CollisionShape3D, destroy it
                    match as_node.try_cast::<CollisionShape3D>() {
                        Ok(mut collision) => {
                            out.remove_child(&collision);
                            collision.queue_free();
                        }
                        Err(_as_node_again) => {}
                    }
                }
            }
        }
    }

    /// Performs all `IslandBuilder` baking steps in order, and applies the results.
    /// If running on a thread, pass `true` for `thread` safe-calls only.
    #[func]
    fn build(&mut self, threaded: bool) {
        // Perform initial data setup
        self.apply_settings();
        if !threaded {
            self.serialize();
        }
        self.net();
        self.optimize();

        // Generate result data
        let mesh = self.generate_baked_mesh();
        let hulls = self.generate_collision_hulls();
        let navprops = self.generate_navigation_properties();
        let volume = self.get_volume();

        // Apply results
        if threaded {
            // Make a deferred call if necessary.
            self.base_mut().call_deferred(
                "apply_build_data",
                &[
                    mesh.to_variant(),
                    hulls.to_variant(),
                    Variant::from(volume),
                    navprops.to_variant(),
                ],
            );
        } else {
            self.apply_build_data(mesh, hulls, volume, navprops);
        }
    }

    /// Applies the provided build data to the island output.
    /// Called by `build(...)` automatically, this function is separated for multi-threading purposes only.
    #[func]
    fn apply_build_data(
        &mut self,
        mesh: Gd<ArrayMesh>,
        hulls: Array<Gd<ConvexPolygonShape3D>>,
        volume: f32,
        navprops: Gd<NavIslandProperties>,
    ) {
        self.apply_mesh(mesh);
        self.apply_collision_hulls(hulls, volume);
        self.apply_navigation_properties(navprops);

        let target = self.base().get_node_or_null(&self.output_to);
        if target.is_some() {
            self.base_mut().set_visible(false);
        }

        self.signals().applied_build_data().emit();
    }

    /// Returns a list of ALL IslandBuilder nodes within the `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    #[func]
    fn all_builders(mut tree: Gd<SceneTree>) -> Array<Gd<Self>> {
        let nodes = tree.get_nodes_in_group(GROUP_NAME);
        let mut builders: Array<Gd<Self>> = array![];

        for node in nodes.iter_shared() {
            match node.try_cast::<Self>() {
                Ok(isle) => builders.push(&isle),
                Err(_none) => {}
            }
        }

        builders
    }

    /// Destroys bakes on **ALL** IslandBuilder nodes within the `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    #[func]
    fn all_destroy_bakes(tree: Gd<SceneTree>) {
        for mut builder in Self::all_builders(tree).iter_shared() {
            builder.bind_mut().destroy_bakes();
        }
    }

    /// Serializes, precomputes and bakes on **ALL** IslandBuilder nodes within the
    /// `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    /// The IslandBuilder will destroy bakes beforehand.
    ///
    /// NOTE: Currently, due to multi-threading, the results may be deferred to the end of frame.
    /// Optionally await `applied_build_data` on an island of your choice to get its ASAP.
    ///
    /// @experimental: This function may change in the future. This function utilizes multi-threading, which may be unstable.
    #[func]
    fn all_bake(tree: Gd<SceneTree>) {
        // Fetch all builder shapes in the scene tree and serialize them
        let builders = Self::all_builders(tree);

        // Ensure all Island Builders are serialized before threading
        for builder in builders.iter_shared() {
            builder.clone().bind_mut().serialize();
        }

        // Get callable to our class' static single-island bake method,
        // and bind our list of working islands to it.
        let bake_method = Callable::from_sync_fn("all_bake_single", |args: &[&Variant]| {
            let idx: i32 = args[0].to();
            let isles: Array<Gd<Self>> = args[1].to();
            // Ensure we don't go out of bounds
            if idx as usize > isles.len() {
                return Ok(Variant::from(false));
            }

            let mut isle = isles.at(idx as usize).clone();
            isle.bind_mut().build(true);
            Ok(Variant::from(true))
        })
        .bind(&[builders.to_variant()]);

        // Fetch worker pool
        let mut workerpool = WorkerThreadPool::singleton();

        // Allocate and run worker threads
        let group_id = workerpool
            .add_group_task_ex(&bake_method, builders.len() as i32)
            .high_priority(true)
            .description("StagToolkit > IslandBuilder > bake all islands in scene")
            .done();

        // Wait for groups to finish
        workerpool.wait_for_group_task_completion(group_id);
    }
}

#[cfg(test)]
mod tests {
    // fn test_binds() {

    // }
}
