use godot::global::sqrt;
use godot::prelude::*;
use godot::engine::mesh::{ArrayType, PrimitiveType};
use godot::engine::{ArrayMesh, CollisionShape3D, ConvexPolygonShape3D, CsgBox3D, CsgSphere3D, ImporterMesh, Material};
use godot::obj::WithBaseField;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use noise::{NoiseFn, Perlin, Seedable};
use godot::engine::utilities::{clampf, pow, remap, exp, log, maxf};
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

use core::fmt;
use std::borrow::BorrowMut;
use std::f64::INFINITY;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}

// Utility Functions
/// Rotates a Godot vector using the provided Euler Angles, in radians
pub fn rotate_xyz(v: Vector3, euler: Vector3) -> Vector3 {
    // Rotation Order: YXZ
    return v.rotated(
        Vector3::FORWARD, -euler.z
    ).rotated(
        Vector3::RIGHT, euler.x
    ).rotated(
        Vector3::UP, euler.y
    );
}

/// Smooth unions two SDF shapes, k = 32.0 was original suggestion
pub fn sdf_smooth_min(a: f64, b: f64, k: f64) -> f64 {
    let res = exp(-k * a) + exp(-k * b);
    return -log(maxf(0.0001, res)) / k;
}
/// Returns the maximum value of a given Godot vector
pub fn max_vector(a: Vector3) -> f32 {
    return f32::max(f32::max(a.x, a.y), a.z);
}

/// Initializes Surface Arrays of a Godot vector
pub fn initialize_surface_array() -> Array<Variant> {
    let mut surface_arrays = Array::new();
    surface_arrays.resize(ArrayType::MAX.ord() as usize, &Array::<Variant>::new().to_variant());
    
    // Bind vertex data
    surface_arrays.set(ArrayType::VERTEX.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::NORMAL.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::TANGENT.ord() as usize, Variant::nil());
    
    // Bind masking data
    surface_arrays.set(ArrayType::COLOR.ord() as usize, Variant::nil());
    
    // Bind UV projections
    surface_arrays.set(ArrayType::TEX_UV.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::TEX_UV2.ord() as usize, Variant::nil());

    // Bind custom arrays
    surface_arrays.set(ArrayType::CUSTOM0.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::CUSTOM1.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::CUSTOM2.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::CUSTOM3.ord() as usize, Variant::nil());

    // Bind skeleton
    surface_arrays.set(ArrayType::BONES.ord() as usize, Variant::nil());
    surface_arrays.set(ArrayType::WEIGHTS.ord() as usize, Variant::nil());
    
    // FINALLY, bind indices
    surface_arrays.set(ArrayType::INDEX.ord() as usize, Variant::nil());

    return surface_arrays;
}
/// Returns vertex data from the buffer at the given index, in Position, Normal format
fn get_buffer_data(pos: [f32; 3], norm: [f32; 3], cell_size: Vector3, offset: Vector3) -> (Vector3, Vector3) {
    return (
        Vector3::new(pos[0], pos[1], pos[2]) * cell_size + offset,
        Vector3::new(-norm[0], -norm[1], -norm[2]).normalized(),
    );
}

// Enums
#[derive(GodotConvert, Var, Export)]
#[godot(via = i64)]
pub enum BuilderShape {
    Box = 0,
    Sphere = 1,
}
impl fmt::Display for BuilderShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuilderShape::Box => write!(f, "box"),
            BuilderShape::Sphere => write!(f, "sphere"),
        }
    }
}

#[derive(GodotClass)]
#[class(base=Resource)]
struct NavIslandProperties {
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

#[derive(GodotClass)]
#[class(base=Resource)]
struct IslandBuilderShape {
    /// Type of shape, describes the used SDF function and how transforms are applied
    #[export]
    shape: BuilderShape,
    /// 3D Position of shape
    #[export]
    position: Vector3,
    /// Euler rotation of shape, in radians
    #[export]
    rotation: Vector3,
    /// 3D scale of shape
    #[export]
    scale: Vector3,
    /// Radius for sphere shapes
    #[export]
    radius: f64,
    /// Smoothing radius for box shapes
    #[export]
    edge_radius: f32,
    /// Z-Score threshold for discarding hull points. Discards if point score is over threshold
    #[export]
    hull_zscore: f64,
    base: Base<Resource>,
}
#[godot_api]
impl IResource for IslandBuilderShape {
    fn init(base: Base<Resource>) -> Self {
        Self {
            shape: BuilderShape::Box,
            position: Vector3::ZERO,
            rotation: Vector3::ZERO,
            scale: Vector3::ONE,
            radius: 1.0,
            edge_radius: 0.0,
            hull_zscore: 2.0,
            base,
        }
    }
    fn to_string(&self) -> GString {
        let Self { shape, position, rotation, scale,  .. } = &self;

        let obj = object! {
            "shape": stringify!(shape),
            "position": [position.x, position.y, position.z],
            "rotation": [rotation.x, rotation.y, rotation.z],
            "scale": [scale.x, scale.y, scale.z],
            "radius": stringify!(radius),
        };

        return json::stringify(obj).into();
    }
}
#[godot_api]
impl IslandBuilderShape {
    #[func]
    fn to_local(&self, position: Vector3) -> Vector3 {
        return rotate_xyz(position - self.position, -self.rotation);
    }

    #[func]
    fn distance(&self, position: Vector3) -> f64 {
        let offset = self.to_local(position);

        match self.shape {
            BuilderShape::Sphere => {
                return (offset / self.scale).length() as f64 - self.radius; // distance minus radius
            },
            BuilderShape::Box => { // SDF rounded box
                let q = offset.abs() - (self.scale / 2.0) + Vector3::splat(self.edge_radius);
                let m = q.coord_max(Vector3::ZERO);
                return (m.length() + f32::min(max_vector(q), 0.0) - self.edge_radius) as f64;
                // https://github.com/jasmcole/Blog/blob/master/CSG/src/fragment.ts#L13
                // https://github.com/fogleman/sdf/blob/main/sdf/d3.py#L140
            },
        }
    }

    /// Gets the bounding-box corners of the given shape. NOT axis-aligned.
    #[func]
    fn get_corners(&self) -> PackedVector3Array {
        // Pre-allocate array
        let mut pts: PackedVector3Array = PackedVector3Array::new();
        pts.resize(8);

        // Allocate points at each corner 
        let half_scale = self.scale.abs() / 2.0;
        pts[0] = Vector3::new( half_scale.x,   half_scale.y,   half_scale.z); // +X +Y +Z
        pts[1] = Vector3::new(-half_scale.x,   half_scale.y,   half_scale.z); // -X +Y +Z
        pts[2] = Vector3::new( half_scale.x,  -half_scale.y,   half_scale.z); // +X -Y +Z
        pts[3] = Vector3::new( half_scale.x,   half_scale.y,  -half_scale.z); // +X +Y -Z
        pts[4] = Vector3::new(-half_scale.x,  -half_scale.y,   half_scale.z); // -X -Y +Z
        pts[5] = Vector3::new( half_scale.x,  -half_scale.y,  -half_scale.z); // +X -Y -Z
        pts[6] = Vector3::new(-half_scale.x,   half_scale.y,  -half_scale.z); // -X +Y -Z
        pts[7] = Vector3::new(-half_scale.x,  -half_scale.y,  -half_scale.z); // -X -Y -Z

        // If this is a sphere, scale the corners up by the sphere radius
        let scale_factor: f32;
        match self.shape {
            BuilderShape::Sphere => scale_factor = 2.0 * (self.radius as f32),
            BuilderShape::Box => scale_factor = 1.0,
        }

        // Perform shape transformations
        for i in 0..=7 {
            pts[i] = self.position + rotate_xyz(pts[i] * scale_factor, -self.rotation);
        }

        return pts;
    }

    /// Returns the Axis-Aligned Bounding Box of the given shape
    #[func]
    fn get_aabb(&self) -> Aabb {
        // Create an empty AABB at the shape center, with no volume
        let mut aabb = Aabb{position: self.position, size: Vector3::ZERO};

        // Get corners of shape
        let corners = self.get_corners();

        // Ensure AABB contains all corners
        for i in 0..=corners.len()-1 {
            aabb = aabb.expand(corners[i]);
        }
        
        return aabb;
    }

    /// Simplify the given vector of points based on this IslandBuilderShape
    /// Utilizes zscores
    fn simplify_hull_internal(&self, pts: Vec<Vector3>) -> Vec<Vector3> {
        let mut distances = Vec::<f64>::new();
        distances.reserve_exact(pts.len());

        // Calculate mean distance and store distances
        let mut mean: f64 = 0.0;
        for i in 0..pts.len() {
            distances.push(self.distance(pts[i]));
            mean += distances[i] as f64;
        }
        mean /= pts.len() as f64;


        // Now calculate standard deviation of the data set
        let mut sdeviation: f64 = 0.0;
        for dist in distances.iter() {
            sdeviation += pow(dist - mean, 2.0);
        }
        let sdeviation = sqrt(sdeviation * (1.0 / (distances.len() as f64)));

        // Allocate new points vector
        let mut new_pts: Vec<Vector3> = Vec::<Vector3>::new();
        new_pts.reserve_exact(pts.len()); // Worst-case scenario, we use all this space

        // Calculate z-score of every point...
        for i in 0..distances.len() {
            let zscore = (distances[i] - mean) / sdeviation;

            // ...and only include point if it falls within our Z-Score threshold
            if zscore < self.hull_zscore {
                new_pts.push(pts[i]);
            }
        }

        return new_pts; // Return new hull
    }
}


// VOXEL RANGES
const MAX_VOLUME_GRID_SIZE: u32 = 64;
type ChunkShape = ConstShape3u32<MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE>;

#[derive(GodotClass)]
#[class(base=Node3D,tool)]
struct IslandBuilder {
    /// When additional nodes are spawned, they are outputed to this object
    #[export]
    output_to: NodePath,
    /// Serialized list of builder shapes
    #[export]
    shapes: Array<Gd<IslandBuilderShape>>,
    /// How many extra cells of padding should be added to the bounding box during generation
    #[export(range = (0.0,10.0, or_greater))]
    cell_padding: i32,
    #[export]
    smoothing_value: f32,
    #[export(range = (0.0, 10.0, 0.05, or_greater))]
    default_edge_radius: f32,
    #[export(range = (0.0, 10.0, 0.01, or_greater))]
    default_hull_zscore: f64,

    #[export(range=(10.0,5000.0,1.0,or_greater))]
    density: f64,
    #[export(range=(1.0,100.0,0.1))]
    density_health: f64,

    noise: Perlin,
    #[var(get = get_noise_seed, set = set_noise_seed, usage_flags = [EDITOR])]
    noise_seed: i64,
    #[export(range=(0.0,1.0,0.001,or_greater))]
    noise_frequency: f32,
    #[export(range=(0.0,1.0,0.001,or_greater))]
    noise_amplitude: f64,

    #[export]
    island_material: Option<Gd<Material>>,
    #[export]
    preview_material: Option<Gd<Material>>,
    #[export]
    mask_range_dirt: Vector2,
    #[export]
    mask_range_sand: Vector2,
    #[export(range = (0.0, 10.0, 0.01, or_greater))]
    mask_power_sand: f64,
    #[export(range = (0.0, 89.9, 0.1))]
    lod_normal_merge_angle: f32,
    #[export(range = (0.0, 179.9, 0.1))]
    lod_normal_split_angle: f32,

    base: Base<Node3D>,
}
#[godot_api]
impl INode3D for IslandBuilder {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            output_to: NodePath::from("."),
            shapes: Array::new(),
            cell_padding: 8,
            smoothing_value: 2.5,
            default_edge_radius: 0.8,
            default_hull_zscore: 2.5,
            density: 2323.0,
            density_health: 20.0,
            noise: Perlin::new(0),
            noise_seed: 0,
            noise_frequency: 0.75,
            noise_amplitude: 0.125,
            island_material: None,
            preview_material: None,
            mask_range_dirt: Vector2::new(-0.1, 0.8),
            mask_range_sand: Vector2::new(0.7, 1.0),
            mask_power_sand: 3.0,
            lod_normal_merge_angle: 25.0,
            lod_normal_split_angle: 65.0,
            base,
        }
    }
}
#[godot_api]
impl IslandBuilder {
    /// Re-Initializes the Perlin noise generator on the IslandBuilder
    #[func]
    pub fn get_noise_seed(&self) -> i64 {
        return self.noise.seed() as i64;
    }
    #[func]
    pub fn set_noise_seed(&mut self, new_seed: i64) {
        self.noise = self.noise.set_seed(new_seed as u32);
    }

    /// Serializes children into IslandBuilderShape objects
    #[func]
    fn serialize(&mut self) {
        self.shapes.clear(); // Clear out all existing shapes

        // Make sure we're visible for parsing, if possible
        let was_visible = self.base().clone().is_visible();
        self.base().clone().set_visible(true);

        // Iterate through children and find
        for child in self.base().get_children().iter_shared() {
            self.serialize_walk(child);
        }

        self.base().clone().set_visible(was_visible); // Return to original visibility

        // Let editor know we've finished serializing
        self.base_mut().emit_signal("completed_serialize".into(), &[]);
    }
    /// Walks child node trees and performs IslandBuilderShape serialization
    fn serialize_walk(&mut self, node: Gd<Node>) {
        // Walk through all of this node's children and perform same operation recursively
        for child in node.get_children().iter_shared() {
            self.serialize_walk(child);
        }
        
        // Attempt to cast node into a CSG Box to see if it is a valid shape
        let csg_box = node.try_cast::<CsgBox3D>();
        match csg_box {

            // If cast succeeds, create a Box shape and pull corresponding data
            Ok(csg_box) => {
                if !csg_box.is_visible_in_tree() { // Ignore this shape if it is not visible
                    return;
                }

                // Instance an IslandBuilderShape placed at our node transform
                let mut shape = self.initialize_shape(csg_box.get_global_transform());
                shape.bind_mut().shape = BuilderShape::Box;
                shape.bind_mut().scale *= csg_box.get_size();
                // Fetch/update edge_radius metadata from box node
                shape.bind_mut().edge_radius = self.fetch_edge_radius(csg_box.clone().upcast());
                shape.bind_mut().hull_zscore = self.fetch_hull_zscore(csg_box.clone().upcast());

                self.shapes.push(shape);
            },

            // If box cast fails, try to cast it to a CSG Sphere instead
            Err(node) => {
                let csg_sphere = node.try_cast::<CsgSphere3D>();
                match csg_sphere {

                    // If cast succeeds, create a Sphere shape and pull corresponding data
                    Ok(csg_sphere) => {
                        if !csg_sphere.is_visible_in_tree() { // Ignore this shape if it is not visible
                            return;
                        }

                        // Instance an IslandBuilderShape placed at our node transform
                        let mut shape = self.initialize_shape(csg_sphere.get_global_transform());
                        shape.bind_mut().shape = BuilderShape::Sphere;
                        shape.bind_mut().radius = csg_sphere.get_radius().into();
                        shape.bind_mut().hull_zscore = self.fetch_hull_zscore(csg_sphere.upcast());
                        self.shapes.push(shape);
                    },

                    // Cast failed, do nothing
                    Err(_node) => {},
                }
            },
        }
    }
    /// Initializes and returns an IslandBuilderShape relative to this Islandbuilder using the given global-space Transform3D
    #[func]
    fn initialize_shape(&self, global_transform: Transform3D) -> Gd<IslandBuilderShape> {
        let mut shape = IslandBuilderShape::new_gd();

        // Get transform of node relative to the IslandBuilder transform
        let t: Transform3D = self.base().get_global_transform().affine_inverse() * global_transform;
        // Fetch node's relative position
        shape.bind_mut().position = t.origin;
        // Fetch rotation, ensuring pitch, yaw, and roll are on expected axii
        shape.bind_mut().rotation = t.basis.to_quat().to_euler(EulerOrder::ZXY);
        // Fetch node's scale
        shape.bind_mut().scale = t.basis.scale();

        return shape;
    }
    /// Fetches the edge radius metadata from the given node, or creates a property for it if not
    fn fetch_edge_radius(&self, mut node: Gd<Node3D>) -> f32 {
        if node.has_meta("edge_radius".into()) {
            return node.get_meta("edge_radius".into()).to();
        }
        
        // If no edge radius was set, set a default one
        node.set_meta("edge_radius".into(), self.default_edge_radius.to_variant());
        return self.default_edge_radius;
    }
    /// Fetches the hull ZScore metadata from the given node, or creates a property for it if not
    fn fetch_hull_zscore(&self, mut node: Gd<Node3D>) -> f64 {
        if node.has_meta("hull_zscore".into()) {
            return node.get_meta("hull_zscore".into()).to();
        }
        
        // If no edge radius was set, set a default one
        node.set_meta("hull_zscore".into(), self.default_hull_zscore.to_variant());
        return self.default_hull_zscore;
    }

    /// Generates a surface net and returns important data
    fn do_surface_nets(&self) -> (SurfaceNetsBuffer, Aabb, Vector3, f64) {
        // Figure out how big we're going
        let aabb = self.get_aabb();
        let cell_size = self.get_cell_size_internal(aabb); // Fetch cell size of mesh
        let aabb = self.get_aabb_padded_internal(aabb, cell_size);

        // Generate an island chunk buffer of max size 
        let mut sdf = [1.0 as f32; ChunkShape::USIZE];
        // Offset cell size by 1 for bounding box, then make sure samples are centered at center of the cell
        let position_offset = aabb.position + cell_size * 0.5;
        // Get cubic size of a volume chunk
        let volume_chunk = (cell_size.x * cell_size.y * cell_size.z) as f64;
        // Prepare to calculate volume
        let mut volume: f64 = 0.0;
        // Sample every voxel of island chunk buffer
        for i in 0u32..ChunkShape::SIZE {
            // Get corresponding X, Y, Z indices of buffer
            let [x, y, z] = ChunkShape::delinearize(i);
            
            // Get field position at given point
            let sample_position = position_offset + Vector3::new(
                x as f32 * cell_size.x, y as f32 * cell_size.y, z as f32 * cell_size.z
            );

            // Sample the SDF at the given position
            let sample = self.sample_at(sample_position);

            // If our sample point is inside shape, add it to our volume estimation
            if sample <= 0.0 {
                volume += volume_chunk;
            }

            // Finally, store the value. Note that SurfaceNet library wants negated distance values
            sdf[i as usize] = -sample as f32;
        }
        
        // Create a SurfaceNet buffer, then create surface net
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &ChunkShape {}, [0; 3], [MAX_VOLUME_GRID_SIZE - 1; 3], &mut buffer);

        // Finally, return it all
        return (buffer, aabb, cell_size, volume);
    }


    /// Generates mesh data from the given SurfaceNetsBuffer.
    /// Returns in order: Indices, Positions, Normals, Valid
    fn generate_mesh_data(&self, buffer: SurfaceNetsBuffer, aabb: Aabb, cell_size: Vector3) -> (PackedInt32Array, PackedVector3Array, PackedVector3Array, bool) {
        // If our buffer is empty, do nothing
        if buffer.indices.is_empty() {
            godot_warn!("IslandBuilder: Island mesh buffer is empty!");
            return (PackedInt32Array::new(), PackedVector3Array::new(), PackedVector3Array::new(), false);
        }

        // Initialize array indices and unwrap our buffer into them
        let mut array_indices = PackedInt32Array::new();
        array_indices.resize(buffer.indices.len());
        for idx in 0..buffer.indices.len() {
            array_indices[idx] = buffer.indices[idx] as i32;
        }

        // Initialize arrays for position data 
        let mut array_positions = PackedVector3Array::new();
        let mut array_normals = PackedVector3Array::new();
        // ...and pre-allocate array size so we're not constantly re-allocating
        array_positions.resize(buffer.positions.len());
        array_normals.resize(buffer.normals.len());

        // For every vertex position...
        for idx in 0..buffer.positions.len() {
            // ...set up mesh data...
            (array_positions[idx], array_normals[idx]) = get_buffer_data(buffer.positions[idx], buffer.normals[idx], cell_size, aabb.position);
        }

        return (array_indices, array_positions, array_normals, true);
    }
    /// Bakes a mesh from the provided SurfaceNetsBuffer data
    fn generate_mesh_baked_internal(&self, array_indices: PackedInt32Array, array_positions: PackedVector3Array, array_normals: PackedVector3Array) -> Gd<ArrayMesh> {
        // Initialize arrays for baking shader data
        let mut array_colors = PackedColorArray::new();
        let mut array_uv1 = PackedVector2Array::new();
        let mut array_uv2 = PackedVector2Array::new();
        // ...and pre-allocate array size
        array_colors.resize(array_positions.len());
        array_uv1.resize(array_positions.len());
        array_uv2.resize(array_positions.len());

        // For every vertex position...
        for idx in 0..array_positions.len() {
            // ...fetch up mesh data...
            let pos = array_positions[idx];
            let normal = array_normals[idx];

            // ...and bake shader data
            array_uv1[idx] = Vector2::new(pos.x + pos.z, pos.y);
            array_uv2[idx] = Vector2::new(pos.x, pos.z);

            // Get ambient occlusion mask, must calculate AO
            let mask_ao = 1.0;

            // Do dot product with up vector for masking, then build dirt and sand masks
            let dot = normal.dot(Vector3::UP) as f64;
            let mask_dirt = clampf(
                remap(dot, self.mask_range_dirt.x.into(), self.mask_range_dirt.y.into(), 0.0, 1.0)
                , 0.0, 1.0
            );
            let mask_sand = pow(
                clampf(
                    remap(dot, self.mask_range_dirt.x.into(), self.mask_range_dirt.y.into(), 0.0, 1.0)
                    , 0.0, 1.0
                ), self.mask_power_sand.into()
            );
            array_colors[idx] = Color::from_rgb(mask_ao as f32, mask_sand as f32, mask_dirt as f32);
        }

        // Initialize mesh surface arrays. To properly use vertex indices, we have to pass *ALL* of the arrays :skull:
        let mut surface_arrays = initialize_surface_array();
        // Bind vertex data
        surface_arrays.set(ArrayType::VERTEX.ord() as usize, array_positions.to_variant());
        surface_arrays.set(ArrayType::NORMAL.ord() as usize, array_normals.to_variant());
        // Bind masking data
        surface_arrays.set(ArrayType::COLOR.ord() as usize, array_colors.to_variant());
        // Bind UV projections
        surface_arrays.set(ArrayType::TEX_UV.ord() as usize, array_uv1.to_variant());
        surface_arrays.set(ArrayType::TEX_UV2.ord() as usize, array_uv2.to_variant());
        // FINALLY, bind indices
        surface_arrays.set(ArrayType::INDEX.ord() as usize, array_indices.to_variant());

        // Add our data to a mesh
        // TODO: Currently ImporterMesh isn't implemented: https://github.com/godot-rust/gdext/issues/156
        // let mut mesh_importer = ImporterMesh::new_gd();
        // mesh_importer.clear();
        // mesh_importer.add_surface(PrimitiveType::TRIANGLES, surface_arrays);
        // mesh_importer.generate_lods(self.lod_normal_merge_angle, self.lod_normal_split_angle, VariantArray::new());
        // let mut mesh = mesh_importer.get_mesh().expect("IslandBuilder: MeshImporter failed to provide ArrayMesh");
        let mut mesh = ArrayMesh::new_gd();
        mesh.borrow_mut().add_surface_from_arrays(PrimitiveType::TRIANGLES, surface_arrays);

        // Set mesh surface material, if provided
        if self.island_material.is_some() {
            mesh.surface_set_name(0, "island".into());
            mesh.surface_set_material(0, self.island_material.clone().expect("No island material specified"));
        }

        return mesh;
    }

    /// Generates a preview island mesh using our IslandBuilderShape list
    #[func]
    fn generate_mesh_preview(&self) -> Gd<ArrayMesh> {
        let (buffer, aabb, cell_size, _) = self.do_surface_nets(); // Precompute Surface Nets
        let (array_indices, array_positions, array_normals, valid) = self.generate_mesh_data(buffer, aabb, cell_size);
        if !valid { return ArrayMesh::new_gd(); }
        
        // Create a new mesh to put data in
        let mut mesh = ArrayMesh::new_gd();
        // Initialize mesh surface arrays, and bind vertex data
        let mut surface_arrays = initialize_surface_array();
        surface_arrays.set(ArrayType::VERTEX.ord() as usize, array_positions.to_variant());
        surface_arrays.set(ArrayType::NORMAL.ord() as usize, array_normals.to_variant());        
        surface_arrays.set(ArrayType::INDEX.ord() as usize, array_indices.to_variant());
        // Add our data to the mesh
        mesh.borrow_mut().add_surface_from_arrays(PrimitiveType::TRIANGLES, surface_arrays);

        // Set mesh surface material, if provided
        if self.island_material.is_some() {
            mesh.surface_set_name(0, "island".into());
            mesh.surface_set_material(0, self.preview_material.clone().expect("No island PREVIEW material specified"));
        }

        return mesh;
    }
    /// Generates a baked mesh using our IslandBuilderShape
    #[func]
    fn generate_mesh_baked(&self) -> Gd<ArrayMesh> {
        let (buffer, aabb, cell_size, _) = self.do_surface_nets(); // Precompute Surface Nets
        let (array_indices, array_positions, array_normals, valid) = self.generate_mesh_data(buffer, aabb, cell_size);
        if !valid { return ArrayMesh::new_gd(); }
        return self.generate_mesh_baked_internal(array_indices, array_positions, array_normals);
    }

    /// Generates ConvexPolygonShape3D based
    #[func]
    fn generate_collision(&self, pts: PackedVector3Array) -> Array::<Gd<ConvexPolygonShape3D>> {
        // Validate that we have usable collision shapes
        let mut hulls = Array::<Gd<ConvexPolygonShape3D>>::new();
        if self.shapes.len() <= 0 {
            godot_error!("Attempt to generate collision hulls for an island with no shapes!");
            return hulls;
        }
        
        // Initialize hull storage
        let mut hull_pts = Vec::<Vec::<Vector3>>::new();

        // Initialize unique point arrays for each hull
        for _ in 0..self.shapes.len() {
            let arr: Vec::<Vector3> = Vec::<Vector3>::new();
            hull_pts.push(arr);
        }

        // Assign each point to the nearest collision hull
        let pts_slice = pts.as_slice();
        for idx in 0..pts_slice.len() {
            let pt = pts_slice[idx];
            let mut min_dist = INFINITY;
            let mut min_shape_idx = 0;

            for shape_idx in 0..self.shapes.len() {
                let mut shape = self.shapes.get(shape_idx).unwrap();
                let d = shape.bind_mut().distance(pt);
                if d < min_dist {
                    min_dist = d;
                    min_shape_idx = shape_idx;
                }
            }

            hull_pts[min_shape_idx].push(pt);
        }

        // Prune unuseful points
        // TODO: multi-thread this
        for i in 0..hull_pts.len() {
            let mut shape = ConvexPolygonShape3D::new_gd();
            shape.set_points(PackedVector3Array::from(self.shapes.get(i).unwrap().bind().simplify_hull_internal(hull_pts[i].clone()).as_slice()));
            hulls.push(shape);
        }

        return hulls;
    }

    /// DO IT ALL. Returns an array and emits an event for different uses.
    /// In this order, returns: ArrayMesh, array of ConvexPolygonShape3D, volume of SDF as a float, and NavIslandProperties
    #[func]
    fn bake(&mut self) -> Array::<Variant> {
        let (buffer, aabb, cell_size, volume) = self.do_surface_nets(); // Precompute Surface Nets
        let (array_indices, array_positions, array_normals, valid) = self.generate_mesh_data(buffer, aabb, cell_size);

        if !valid {
            return Array::<Variant>::new();
        }

        // Bake mesh and generate collision
        let mesh = self.generate_mesh_baked_internal(array_indices, array_positions.clone(), array_normals);
        let collis = self.generate_collision(array_positions);
        let nav_props = self.get_navigation_properties();
        
        // Once complete, package all data up for sending
        let output = &[mesh.to_variant(), collis.to_variant(), Variant::from(volume), nav_props.to_variant()];

        // Emit as a signal in case we're threaded...
        self.base_mut().emit_signal("completed_bake".into(),  output);

        // ...and return as an array!
        return Array::<Variant>::from(output);
    }

    /// Samples the IslandBuilder SDF at the given local position
    #[func]
    fn sample_at(&self, sample_position: Vector3) -> f64 {
        let mut d = 1.0;
        // Accumulate signed distance field values at this point
        for mut shape in self.shapes.iter_shared() {
            // Smooth union each shape together
            // Adjust k value for smoothness
            d = sdf_smooth_min(d, shape.bind_mut().distance(sample_position), self.smoothing_value.into());
        }

        let noise_pos = sample_position * self.noise_frequency;
        let noise = self.noise.get([noise_pos.x as f64, noise_pos.y as f64, noise_pos.z as f64]) * self.noise_amplitude;

        return d + noise;
    }

    /// Calculates the Axis-Aligned Bounding Box for the IslandBuilder given our shape list
    #[func]
    fn get_aabb(&self) -> Aabb {
        let mut aabb = Aabb{position: Vector3::ZERO, size: Vector3::ZERO};

        for shape in self.shapes.iter_shared() {
            aabb = aabb.merge(shape.bind().get_aabb());
        }

        return aabb;
    }
    /// Calculates the Axis-Aligned Bounding Box for the IslandBuilder, with some extra padding
    #[func]
    fn get_aabb_padded(&self) -> Aabb {
        let aabb = self.get_aabb();
        return self.get_aabb_padded_internal(aabb, self.get_cell_size_internal(aabb));
    }
    /// Calculates the Axis-Aligned Bounding Box for the IslandBuilder, with some extra padding, using pre-calculated values
    fn get_aabb_padded_internal(&self, mut aabb: Aabb, cell_size: Vector3) -> Aabb {
        aabb.position -= cell_size * (self.cell_padding as f32 * 0.5);
        aabb.size += cell_size * (self.cell_padding as f32);
        return aabb;
    }

    /// Returns the anticipated cell size of the IslandBuilder volume
    #[func]
    fn get_cell_size(&self) -> Vector3 {
        return self.get_cell_size_internal(self.get_aabb());
    }
    /// Returns the anticipated cell size of the IslandBuilder volume, using a pre-made AABB
    fn get_cell_size_internal(&self, aabb: Aabb) -> Vector3 {
        let grid_size = (MAX_VOLUME_GRID_SIZE - (self.cell_padding as u32)) as f32;
        return Vector3::new(aabb.size.x / grid_size, aabb.size.y / grid_size, aabb.size.z / grid_size);
    }

    /// Returns the estimated navigation properties of the island
    #[func]
    fn get_navigation_properties(&self) -> Gd<NavIslandProperties> {
        let mut props = NavIslandProperties::new_gd();
        let aabb = self.get_aabb();
        props.bind_mut().aabb = aabb;
        props.bind_mut().radius = (aabb.size * Vector3::new(1.0, 0.0, 1.0)).length() / 2.0;
        props.bind_mut().center = (aabb.center() * Vector3::new(1.0, 0.0, 1.0)) + (aabb.support(Vector3::UP) * Vector3::UP);
        return props;
    }

    /// Emitted when IslandBuilder has finished serializing builder shapes
    #[signal]
    fn completed_serialize();
    /// Emitted when IslandBuilder has finished generating an island mesh
    #[signal]
    fn completed_bake(mesh: Gd<ArrayMesh>, hulls: Array<Gd<CollisionShape3D>>, volume: f64, navigation_properties: Gd<NavIslandProperties>);
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
