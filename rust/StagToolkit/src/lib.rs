use godot::engine::mesh::ArrayType;
use godot::engine::mesh::PrimitiveType;
use godot::engine::ArrayMesh;
use godot::engine::CsgBox3D;
use godot::engine::CsgSphere3D;
use godot::engine::Material;
use godot::engine::RigidBody3D;
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use noise::NoiseFn;
use noise::Perlin;
use noise::Seedable;
use utilities::clampf;
use utilities::pow;
use utilities::remap;
use utilities::{exp, log};
use utilities::maxf;
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

use core::fmt;
use std::borrow::{Borrow, BorrowMut};
use std::cell;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}

// Utility Functions
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

// Smooth unions two SDF shapes, k = 32.0 was original suggestion
pub fn sdf_smooth_min(a: f64, b: f64, k: f64) -> f64 {
    let res = exp(-k * a) + exp(-k * b);
    return -log(maxf(0.0001, res)) / k;
}

pub fn max_vector(a: Vector3) -> f32 {
    return f32::max(f32::max(a.x, a.y), a.z);
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
// impl BuilderShape {
//     fn from_string(string_value: &str) -> BuilderShape {
//         return match string_value {
//             "sphere" => BuilderShape::Sphere,
//             _ => BuilderShape::Box,
//         }

//     }
// }


#[derive(GodotClass)]
#[class(base=Resource)]
struct IslandBuilderShape {
    #[export]
    shape: BuilderShape,
    #[export]
    position: Vector3,
    #[export]
    rotation: Vector3,
    #[export]
    scale: Vector3,
    #[export]
    radius: f64,
    #[export]
    edge_radius: f32,
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
    // #[func]
    // fn from_json(obj: Dictionary) -> Gd<Self> {
    //     let shape = BuilderShape::from_string(obj.get("shape").unwrap().into());
    //     let position = obj.get("position").unwrap().try_to(Vector3);
    //     let rotation = obj.get("position").unwrap();
    //     let scale = obj.get("position").unwrap();
    //     Gd::from_object(Self {
    //         shape,
    //         position,
    //         rotation,
    //         scale,
    //         base,
    //     })
    // }
    #[func]
    fn to_local(&self, position: Vector3) -> Vector3 {
        return rotate_xyz(position - self.position, -self.rotation);
    }

    #[func]
    fn distance(&mut self, position: Vector3) -> f64 {
        let offset = self.to_local(position);

        match self.shape {
            BuilderShape::Sphere => {
                return (offset / self.scale).length() as f64 - self.radius; // distance minus radius
            },
            BuilderShape::Box => { // SDF rounded box
                let q = offset.abs() - (self.scale / 2.0) + Vector3::splat(self.edge_radius);
                let m = q.coord_max(Vector3::ZERO);
                return (m.length() + f32::min(q[q.max_axis_index()], 0.0) - self.edge_radius) as f64;
                // https://github.com/jasmcole/Blog/blob/master/CSG/src/fragment.ts#L13
                // https://github.com/fogleman/sdf/blob/main/sdf/d3.py#L140
            },
        }
    }

    #[func]
    fn get_corners(&self) -> PackedVector3Array {
        let mut pts: PackedVector3Array = PackedVector3Array::new();
        pts.resize(8);

        let half_scale = self.scale.abs() / 2.0;
        pts.set(0,Vector3::new( half_scale.x,   half_scale.y,   half_scale.z)); // +X +Y +Z
        pts.set(1,Vector3::new(-half_scale.x,   half_scale.y,   half_scale.z)); // -X +Y +Z
        pts.set(2,Vector3::new( half_scale.x,  -half_scale.y,   half_scale.z)); // +X -Y +Z
        pts.set(3,Vector3::new( half_scale.x,   half_scale.y,  -half_scale.z)); // +X +Y -Z
        pts.set(4,Vector3::new(-half_scale.x,  -half_scale.y,   half_scale.z)); // -X -Y +Z
        pts.set(5,Vector3::new( half_scale.x,  -half_scale.y,  -half_scale.z)); // +X -Y -Z
        pts.set(6,Vector3::new(-half_scale.x,   half_scale.y,  -half_scale.z)); // -X +Y -Z
        pts.set(7,Vector3::new(-half_scale.x,  -half_scale.y,  -half_scale.z)); // -X -Y -Z

        // If this is a sphere, scale the corners up by the sphere radius
        let scale_factor: f32;
        match self.shape {
            BuilderShape::Sphere => scale_factor = 2.0 * (self.radius as f32),
            BuilderShape::Box => scale_factor = 1.0,
        }

        for i in 0..=7 {
            pts.set(i, self.position + rotate_xyz(pts.get(i) * scale_factor, -self.rotation));
        }

        return pts;
    }

    #[func]
    fn get_aabb(&self) -> Aabb {
        // Create an empty AABB at the shape center, with no volume
        let mut aabb = Aabb{position: self.position, size: Vector3::ZERO};

        // Get corners of shape
        let corners = self.get_corners();

        // Ensure AABB contains all corners
        for i in 0..=corners.len()-1 {
            aabb = aabb.expand(corners.get(i));
        }
        
        return aabb;
    }
}


// VOXEL RANGES
const MAX_VOLUME_GRID_SIZE: u32 = 64;
type ChunkShape = ConstShape3u32<MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE>;

#[derive(GodotClass)]
#[class(base=Node3D,tool)]
struct IslandBuilder {
    #[export]
    shapes: Array<Gd<IslandBuilderShape>>,
    #[export(range = (0.0,10.0, or_greater))]
    cell_padding: i32,
    #[export]
    smoothing_value: f32,
    #[export(range = (0.0, 10.0, 0.05, or_greater))]
    default_edge_radius: f32,

    noise: Perlin,
    #[var(get = get_noise_seed, set = set_noise_seed, usage_flags = [EDITOR])]
    noise_seed: i64,
    #[export]
    noise_frequency: f32,
    #[export]
    noise_amplitude: f64,

    #[export]
    island_material: Option<Gd<Material>>,
    #[export]
    mask_range_dirt: Vector2,
    #[export]
    mask_range_sand: Vector2,
    #[export(range = (0.0, 10.0, 0.01, or_greater))]
    mask_power_sand: f64,

    base: Base<Node3D>,
}
#[godot_api]
impl INode3D for IslandBuilder {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            shapes: Array::new(),
            cell_padding: 2,
            smoothing_value: 3.0,
            default_edge_radius: 0.2,
            noise: Perlin::new(0),
            noise_seed: 0,
            noise_frequency: 1.0,
            noise_amplitude: 1.0,
            island_material: None,
            mask_range_dirt: Vector2::new(-0.1, 0.8),
            mask_range_sand: Vector2::new(0.7, 1.0),
            mask_power_sand: 3.0,
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

        // Iterate through children and find
        for child in self.base().get_children().iter_shared() {
            self.serialize_walk(child);
        }
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
                if !csg_box.is_visible() { // Ignore this shape if it is not visible
                    return;
                }

                // Instance an IslandBuilderShape placed at our node transform
                let mut shape = self.initialize_shape(csg_box.get_global_transform());
                shape.bind_mut().shape = BuilderShape::Box;
                shape.bind_mut().scale *= csg_box.get_size();
                // Fetch/update edge_radius metadata from box node
                shape.bind_mut().edge_radius = self.fetch_edge_radius(csg_box.upcast());

                self.shapes.push(shape);
            },

            // If box cast fails, try to cast it to a CSG Sphere instead
            Err(node) => {
                let csg_sphere = node.try_cast::<CsgSphere3D>();
                match csg_sphere {

                    // If cast succeeds, create a Sphere shape and pull corresponding data
                    Ok(csg_sphere) => {
                        if !csg_sphere.is_visible() { // Ignore this shape if it is not visible
                            return;
                        }

                        // Instance an IslandBuilderShape placed at our node transform
                        let mut shape = self.initialize_shape(csg_sphere.get_global_transform());
                        shape.bind_mut().shape = BuilderShape::Sphere;
                        shape.bind_mut().radius = csg_sphere.get_radius().into();
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

    /// Generates an island mesh using our IslandBuilderShape list
    #[func]
    fn generate_mesh(&mut self) -> Gd<ArrayMesh> { //  -> Gd<ArrayMesh>
        // Figure out how big we're going
        let aabb = self.get_aabb();
        let cell_size = self.get_cell_size_internal(aabb); // Fetch cell size of mesh
        let aabb = self.get_aabb_padded_internal(aabb, cell_size);

        // Generate an island chunk buffer of max size 
        let mut sdf = [1.0 as f32; ChunkShape::USIZE];
        // Offset cell size by 1 for bounding box, then make sure samples are centered at center of the cell
        let position_offset = aabb.position + cell_size * 0.5;
        // Sample every voxel of island chunk buffer
        for i in 0u32..ChunkShape::SIZE {
            // Get corresponding X, Y, Z indices of buffer
            let [x, y, z] = ChunkShape::delinearize(i);
            
            // Get field position at given point
            let sample_position = position_offset + Vector3::new(
                x as f32 * cell_size.x, y as f32 * cell_size.y, z as f32 * cell_size.z
            );

            // Finally, sample and store value. Note that SurfaceNet library wants negated distance values
            sdf[i as usize] = -self.sample_at(sample_position) as f32;
        }
        
        // Create a SurfaceNet buffer, then create surface net
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &ChunkShape {}, [0; 3], [MAX_VOLUME_GRID_SIZE - 1; 3], &mut buffer);

        // Create a new mesh to put data in
        let mut mesh = ArrayMesh::new_gd();

        // If our buffer is empty, do nothing
        if buffer.indices.is_empty() {
            godot_warn!("Island mesh buffer is empty");
            return mesh;
        }
        // If buffers are out of whack, do nothing
        if buffer.positions.len() != buffer.normals.len() {
            godot_warn!("Position buffer length does not match normal buffer length");
            return mesh;
        }

        // Process and pipe data into Godot mesh
        let mut array_indices = PackedInt32Array::new();

        array_indices.resize(buffer.indices.len());
        for idx in 0..buffer.indices.len() {
            array_indices.set(idx,  buffer.indices[idx] as i32);
        }

        // Initialize arrays for position data 
        let mut array_positions = PackedVector3Array::new();
        let mut array_normals = PackedVector3Array::new();
        // ...and pre-allocate array size so we're not constantly re-allocating
        array_positions.resize(buffer.positions.len());
        array_normals.resize(buffer.normals.len());

        // Initialize arrays for baking shader data
        let mut array_colors = PackedColorArray::new();
        let mut array_uv1 = PackedVector2Array::new();
        let mut array_uv2 = PackedVector2Array::new();
        // ...and pre-allocate array size
        array_colors.resize(buffer.normals.len());
        array_uv1.resize(buffer.positions.len());
        array_uv2.resize(buffer.positions.len());

        // For every vertex position...
        for idx in 0..buffer.positions.len() {
            // ...set up mesh data...
            let pos = Vector3::new(buffer.positions[idx][0], buffer.positions[idx][1], buffer.positions[idx][2]) * cell_size + aabb.position;
            let normal = Vector3::new(-buffer.normals[idx][0], -buffer.normals[idx][1], -buffer.normals[idx][2]).normalized();
            array_positions.set(idx, pos);
            array_normals.set(idx, normal);

            // ...and bake shader data
            array_uv1.set(idx, Vector2::new(pos.x + pos.z, pos.y));
            array_uv2.set(idx, Vector2::new(pos.x, pos.z));

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
            array_colors.set(idx, Color::from_rgb(mask_ao as f32, mask_sand as f32, mask_dirt as f32));
        }

        // Initialize mesh surface arrays
        // To properly use vertex indices, we have to pass *ALL* of the arrays :skull:
        let mut surface_arrays = Array::new();
        surface_arrays.resize(ArrayType::MAX.ord() as usize, &Array::<Variant>::new().to_variant());
        
        // Bind vertex data
        surface_arrays.set(ArrayType::VERTEX.ord() as usize, array_positions.to_variant());
        surface_arrays.set(ArrayType::NORMAL.ord() as usize, array_normals.to_variant());
        surface_arrays.set(ArrayType::TANGENT.ord() as usize, Variant::nil());
        
        // Bind masking data
        surface_arrays.set(ArrayType::COLOR.ord() as usize, array_colors.to_variant());
        
        // Bind UV projections
        surface_arrays.set(ArrayType::TEX_UV.ord() as usize, array_uv1.to_variant());
        surface_arrays.set(ArrayType::TEX_UV2.ord() as usize, array_uv2.to_variant());

        // Bind custom arrays
        surface_arrays.set(ArrayType::CUSTOM0.ord() as usize, Variant::nil());
        surface_arrays.set(ArrayType::CUSTOM1.ord() as usize, Variant::nil());
        surface_arrays.set(ArrayType::CUSTOM2.ord() as usize, Variant::nil());
        surface_arrays.set(ArrayType::CUSTOM3.ord() as usize, Variant::nil());

        // Bind skeleton
        surface_arrays.set(ArrayType::BONES.ord() as usize, Variant::nil());
        surface_arrays.set(ArrayType::WEIGHTS.ord() as usize, Variant::nil());
        
        // FINALLY, bind indices
        surface_arrays.set(ArrayType::INDEX.ord() as usize, array_indices.to_variant());

        // Add our data to the mesh
        mesh.borrow_mut().add_surface_from_arrays(PrimitiveType::TRIANGLES, surface_arrays);

        // Set mesh surface material, if provided
        if self.island_material.is_some() {
            mesh.surface_set_name(0, "island".into());
            mesh.surface_set_material(0, self.island_material.clone().expect("No island material specified"));
        }

        // Return mesh through a signal (TODO, how do we unborrow the object)
        self.base_mut().emit_signal("generated_mesh".into(), &[mesh.borrow().to_variant()]);
        godot_print!("All done!");

        return mesh;
    }

    #[func]
    fn generate_collision(&self, _parent: Gd<RigidBody3D>) {
        
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
            aabb = aabb.merge(&shape.bind().get_aabb());
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

    /// Performs an SDF smooth union between two distances (a, b) with the given smoothing value (k)
    #[func]
    fn smooth_union(a: f64, b: f64, k: f64) -> f64 {
        return sdf_smooth_min(a, b, k);
    }

    /// Emitted when IslandBuilder has finished serializing builder shapes
    #[signal]
    fn serialized();

    /// Emitted when IslandBuilder has finished generating an island mesh
    #[signal]
    fn generated_mesh(mesh: Gd<ArrayMesh>);
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
