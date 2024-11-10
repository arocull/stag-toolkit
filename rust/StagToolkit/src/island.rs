use core::f32;
use std::ops::Index;

use crate::{
    math::{
        sdf::{sample_shape_list, ShapeOperation},
        types::{ToColor, ToVector2, ToVector3, Vec3Godot},
        volumetric::{PerlinField, VolumeData},
    },
    mesh::{
        godot::{GodotSurfaceArrays, GodotWhitebox},
        nets::mesh_from_nets,
        trimesh::TriangleMesh,
    },
};
use fast_surface_nets::{
    ndshape::{ConstShape, ConstShape3u32},
    surface_nets, SurfaceNetsBuffer,
};
use glam::{FloatExt, Mat4, Vec2, Vec3, Vec4};
use godot::{
    classes::{mesh::PrimitiveType, ArrayMesh, ConvexPolygonShape3D, Material},
    prelude::*,
};

const VOLUME_MAX_CELLS: u32 = 64;
type IslandChunkSize = ConstShape3u32<VOLUME_MAX_CELLS, VOLUME_MAX_CELLS, VOLUME_MAX_CELLS>;

/// Container for working data about a given island.
pub struct IslandBuildData {
    whitebox: GodotWhitebox,
    noise: PerlinField,

    // CONFIGURATION //
    cell_padding: u32,
    smoothing_repetitions: u32,
    smoothing_radius_voxels: u32,
    smoothing_weight: f32,
    noise_amplitude: f32,
    noise_w: f64,
    mask_range_dirt: Vec2,
    mask_range_sand: Vec2,
    mask_power_sand: f32,
    mask_perlin_scale: Vec3,

    /// Mesh generated via surface nets.
    /// Stored as option in case it was not already generated.
    mesh: Option<TriangleMesh>,

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
            smoothing_repetitions: 3,
            smoothing_radius_voxels: 2,
            smoothing_weight: 0.5,
            noise_amplitude: 0.4,
            noise_w: 1.0,
            mask_range_dirt: Vec2::new(-0.1, 0.8),
            mask_range_sand: Vec2::new(0.7, 1.0),
            mask_power_sand: 3.0,
            mask_perlin_scale: Vec3::new(0.75, 0.33, 0.75),

            mesh: None,

            volume: 0.0,
        }
    }

    /// Perform Naive Surface Nets algorithm on geometry.
    /// TODO: Utilize chunking to break up task.
    fn nets(&mut self) {
        // If voxel data was not already initialized, initialize it
        let mut voxels = VolumeData::new(
            1.0f32,
            [VOLUME_MAX_CELLS, VOLUME_MAX_CELLS, VOLUME_MAX_CELLS],
        );

        let aabb: Aabb = self.whitebox.get_aabb().grow(self.noise_amplitude.abs());
        let minimum_bound: Vec3 = aabb.position.to_vector3();
        let bound_size: Vec3 = aabb.size.to_vector3();

        let grid_size: f32 = (VOLUME_MAX_CELLS - (self.cell_padding * 2)) as f32;
        let cell_size: Vec3 = Vec3::new(
            bound_size.x / grid_size,
            bound_size.y / grid_size,
            bound_size.z / grid_size,
        );

        let minimum_bound = minimum_bound - cell_size * Vec3::splat(self.cell_padding as f32);

        // Estimate volume
        let mut volume: f32 = 0.0;
        let volume_per_voxel = cell_size.x * cell_size.y * cell_size.z;

        let shapes = self.whitebox.get_shapes();

        // Transformation matrix for quickly moving points
        let trans: Mat4 = Mat4::from_translation(minimum_bound) * Mat4::from_scale(cell_size);

        // Sample
        for i in 0u32..IslandChunkSize::SIZE {
            let [x, y, z] = IslandChunkSize::delinearize(i);

            let sample_pos: Vec3 = trans.transform_point3(Vec3::new(x as f32, y as f32, z as f32));

            // let noise = self.noise.sample(sample_position, 1.0) * noise_amplitude;
            let sample = sample_shape_list(shapes, sample_pos);

            voxels.set_linear(i as usize, sample);
        }

        // Factor noise in
        voxels.noise_add(&self.noise, trans, self.noise_w, self.noise_amplitude);

        // Perform smoothing blurs
        for _i in 0u32..self.smoothing_repetitions {
            voxels = voxels.blur(self.smoothing_radius_voxels, self.smoothing_weight);
        }

        // Perform padding
        voxels.trim_padding(self.cell_padding);

        // Convert voxel data to buffer for surface-nets
        let mut snbuffer = [1.0f32; IslandChunkSize::USIZE];
        for i in 0usize..IslandChunkSize::USIZE {
            let sample = voxels.get_linear(i);
            snbuffer[i] = -sample;

            if sample < 0.0 {
                volume += volume_per_voxel;
            }
        }

        self.volume = volume;

        // Perform surface nets algorithm
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(
            &snbuffer,
            &IslandChunkSize {},
            [0; 3],
            [VOLUME_MAX_CELLS - 1; 3],
            &mut buffer,
        );

        self.mesh = mesh_from_nets(buffer, cell_size, minimum_bound);
        if self.mesh.is_none() {
            godot_warn!("IslandBuilder: Generated mesh buffer was empty.");
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
                let mesh = self.mesh.as_ref().unwrap();
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

                Some(x)
            }
            None => None,
        }
    }

    /// Iterates through all positions on the mesh, assigning them to nearby collision hulls.
    /// Returns an empty vector if no hulls were generated.
    fn get_hulls(&self) -> Vec<Vec<Vec3>> {
        let mut hulls: Vec<Vec<Vec3>> = vec![];
        if self.mesh.is_none() {
            return hulls;
        }

        // Fetch the shape list, and allocate an equal amount of collision hulls
        let shapes = self.whitebox.get_shapes();
        hulls.reserve_exact(shapes.len());
        for _ in shapes.iter() {
            hulls.push(vec![]);
        }

        // Assign each point to the nearest collision hull
        let points = self.mesh.as_ref().unwrap().positions.clone();
        for pt in points.iter() {
            let mut min_dist = f32::INFINITY;
            let mut min_shape_idx = 0;

            for shape_idx in 0..shapes.len() {
                let shape = shapes.index(shape_idx);

                // Ignore non-union shapes
                if shape.operation != ShapeOperation::Union {
                    continue;
                }

                // TODO: somehow take intersection steps into account,
                // so collision shapes that are cut off via intersections,
                // do not include shapes added after said intersection.
                let d = shape.sample(*pt);
                if d < min_dist {
                    min_dist = d;
                    min_shape_idx = shape_idx;
                }
            }

            hulls[min_shape_idx].push(*pt);
        }

        // Remove unused hulls. Iterate over array backwards so we hit the end ones first
        for (idx, hull) in hulls.clone().iter().enumerate().rev() {
            if hull.is_empty() {
                hulls.remove(idx);
            }
        }

        // TODO: run decimate?

        hulls
    }
}

// GODOT CLASSES //

/// Navigation properties for Abyss islands.
/// These are utilized for A* pathing with Character AI.
#[derive(GodotClass)]
#[class(base=Resource)]
pub struct NavIslandProperties {
    #[export]
    aabb: Aabb,
    #[export]
    center: Vector3,
    #[export]
    radius: f32,
    #[export]
    surface_flatness: f32,
    base: Base<Resource>,
}
#[godot_api]
impl IResource for NavIslandProperties {
    /// Initialize `NavIslandProperties``
    fn init(base: Base<Resource>) -> Self {
        Self {
            aabb: Aabb::new(Vector3::ZERO, Vector3::ZERO),
            center: Vector3::ZERO,
            radius: 5.0,
            surface_flatness: 1.0,
            base,
        }
    }
}

/// The `IslandBuilder` is used to convert whitebox geometry into game-ready islands using procedural geometry.
/// To create a mesh, add CSGBox and CSGSphere nodes as descendants to the IslandBuilder,
/// then `serialize()`, `net()` and fetch your related data.
#[derive(GodotClass)]
#[class(base=Node3D,tool)]
pub struct IslandBuilder {
    data: IslandBuildData,

    /// Node to output the result on.
    /// This may affect how data is stored and applied via the plugin.
    #[export]
    output_to: NodePath,

    /// Number of cells to pad on each side of the IslandBuilder volume.
    #[export(range = (0.0, 6.0, or_greater))]
    generation_cell_padding: u32,
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
    #[export(range = (0.0, 2.0))]
    generation_edge_radius: f32,
    /// Z-Score to use for culling collision hull points.
    #[export]
    generation_hull_zscore: f32,
    /// Noise seed to use for generation.
    #[export(range = (0.0, 1000.0, or_greater))]
    noise_seed: u32,
    /// Noise frequency.
    #[export]
    noise_frequency: f32,
    /// Noise amplitude, in meters.
    /// This value is directly added to the SDF result in the volume pass.
    /// Advized to keep below 1.0 meter.
    #[export(range = (0.0, 1.0, or_greater))]
    noise_amplitude: f32,
    /// W position for sampling noise.
    #[export]
    noise_w: f64,

    /// Approximate physical density of material to use when calculating mass.
    /// Kilograms per meter cubed.
    #[export]
    gameplay_density: f32,
    /// Approximate health density of material to use when calculating island health.
    /// Hit Points per meter cubed.
    #[export]
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
    fn init(base: Base<Node3D>) -> Self {
        Self {
            data: IslandBuildData::new(),
            output_to: NodePath::from("."),
            generation_cell_padding: 2,
            generation_smoothing_iterations: 3,
            generation_smoothing_radius_voxels: 2,
            generation_smoothing_weight: 0.5,
            generation_edge_radius: 1.0,
            generation_hull_zscore: 2.0,
            noise_seed: 0,
            noise_frequency: 0.335,
            noise_amplitude: 0.4,
            noise_w: 1.0,
            gameplay_density: 23.23,
            gameplay_health_density: 2.0,
            material_baked: None,
            material_preview: None,
            base,
        }
    }
}

#[godot_api]
impl IslandBuilder {
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

    // Setters //

    // Build Steps //

    /// Applies Godot settings to corresponding whitebox and mesh data.
    fn apply_settings(&mut self) {
        // Apply whitebox settings
        self.data.cell_padding = self.generation_cell_padding;
        self.data.smoothing_repetitions = self.generation_smoothing_iterations;
        self.data.smoothing_radius_voxels = self.generation_smoothing_radius_voxels;
        self.data.smoothing_weight = self.generation_smoothing_weight;
        self.data.whitebox.default_edge_radius = self.generation_edge_radius;
        self.data.whitebox.default_hull_zscore = self.generation_hull_zscore;

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
    #[func]
    pub fn serialize(&mut self) {
        self.data.whitebox.clear();
        self.apply_settings();
        self.data.whitebox.serialize_from(self.base().to_godot());
        self.base_mut()
            .emit_signal("completed_serialize".into(), &[]);
    }

    /// Performs Surface Nets Algorithm, storing it in the IslandBuilderData for future use.
    /// Returns true if the generated mesh is empty.
    #[func]
    pub fn net(&mut self) -> bool {
        self.apply_settings();
        self.data.nets();

        self.base_mut().emit_signal("completed_nets".into(), &[]);

        self.data.mesh.is_none()
    }

    /// Returns a simple triangle mesh for previewing, without baking any data.
    /// Returns an empty mesh if not pre-computed.
    #[func]
    pub fn mesh_preview(&self, recycle_mesh: Option<Gd<ArrayMesh>>) -> Gd<ArrayMesh> {
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
                mesh.surface_set_name(0, "island".into());
                // Add a material, if valid
                if self.material_preview.is_some() {
                    mesh.surface_set_material(0, self.material_preview.clone());
                }
                mesh
            }
            _ => mesh,
        }
    }
    /// Bakes and returns a triangle mesh with vertex colors, UVs, (TODO: and LODs).
    /// Returns an empty mesh if not pre-computed.
    #[func]
    pub fn mesh_baked(&self) -> Gd<ArrayMesh> {
        let mut mesh = ArrayMesh::new_gd();
        let arrs_opt = self.data.get_mesh_baked();
        match arrs_opt {
            Some(arrs) => {
                mesh.add_surface_from_arrays(PrimitiveType::TRIANGLES, &arrs.get_surface_arrays());
                mesh.surface_set_name(0, "island".into());
                if self.material_baked.is_some() {
                    mesh.surface_set_material(0, self.material_baked.clone());
                }
                mesh
            }
            _ => mesh,
        }
    }
    /// Computes and returns a list of collision hulls for the IslandBuilder shape.
    /// Returns an empty array if not pre-computed.
    #[func]
    pub fn collision_hulls(&self) -> Array<Gd<ConvexPolygonShape3D>> {
        let hull_pts = self.data.get_hulls();

        let mut hulls = Array::<Gd<ConvexPolygonShape3D>>::new();
        for hull in hull_pts.iter() {
            let mut shape = ConvexPolygonShape3D::new_gd();
            shape.set_points(&hull.clone().to_vector3());
            hulls.push(shape);
        }

        hulls
    }
    /// Computes and returns the navigation properties of the island.
    /// Properties will be zero'd if not pre-computed.
    #[func]
    fn navigation_properties(&self) -> Gd<NavIslandProperties> {
        let mut props = NavIslandProperties::new_gd();
        let aabb = self.get_aabb();

        let size: Vec3 = aabb.size.to_vector3();
        let rad: f32 = (size * Vec3::new(1.0, 0.0, 1.0)).length() / 2.0;

        props.bind_mut().aabb = aabb;
        props.bind_mut().radius = rad;
        props.bind_mut().center =
            (aabb.center() * Vec3Godot::new(1.0, 0.0, 1.0)) + (aabb.support(Vec3Godot::UP));

        props
    }

    /// Emitted upon completing serialization.
    #[signal]
    pub fn completed_serialize();
    /// Emitted upon completing pre-computation step.
    #[signal]
    pub fn completed_nets();
}
