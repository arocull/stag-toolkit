use godot::engine::mesh::ArrayType;
use godot::engine::mesh::PrimitiveType;
use godot::engine::ArrayMesh;
use godot::engine::CsgBox3D;
use godot::engine::CsgSphere3D;
use godot::engine::Material;
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use utilities::{exp, log};
use utilities::maxf;
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

use core::fmt;
use std::borrow::BorrowMut;

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

        let half_scale = self.scale / 2.0;
        pts.set(0,Vector3::new(half_scale.x, half_scale.y, half_scale.z));   // +X +Y +Z
        pts.set(1,Vector3::new(-half_scale.x, half_scale.y, half_scale.z));  // -X +Y +Z
        pts.set(2,Vector3::new(half_scale.x, -half_scale.y, half_scale.z));  // +X -Y +Z
        pts.set(3,Vector3::new(half_scale.x, half_scale.y, -half_scale.z));  // +X +Y -Z
        pts.set(4,Vector3::new(-half_scale.x, -half_scale.y, half_scale.z)); // -X -Y +Z
        pts.set(5,Vector3::new(half_scale.x, -half_scale.y, -half_scale.z)); // +X -Y -Z
        pts.set(6,Vector3::new(-half_scale.x, half_scale.y, -half_scale.z)); // -X +Y -Z
        pts.set(7,Vector3::new(-half_scale.x, -half_scale.y, -half_scale.z)); // -X -Y -Z

        // If this is a sphere, scale the corners up by the sphere radius
        let scale_factor: f32;
        match self.shape {
            BuilderShape::Sphere => scale_factor = 2.0 * (self.radius as f32),
            BuilderShape::Box => scale_factor = 1.0,
        }

        for i in 0..=7 {
            pts.set(i, self.position + rotate_xyz(pts.get(i) * scale_factor, self.rotation));
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
const MAX_VOLUME_GRID_SIZE: u32 = 48;
type ChunkShape = ConstShape3u32<MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE>;

#[derive(GodotClass)]
#[class(base=Node3D,tool)]
struct IslandBuilder {
    #[export]
    shapes: Array<Gd<IslandBuilderShape>>,
    #[export]
    smoothing_value: f32,
    #[export]
    default_edge_radius: f32,
    #[export]
    island_material: Option<Gd<Material>>,

    base: Base<Node3D>,
}
#[godot_api]
impl INode3D for IslandBuilder {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            shapes: Array::new(),
            smoothing_value: 3.0,
            default_edge_radius: 0.2,
            island_material: None,
            base,
        }
    }
}
#[godot_api]
impl IslandBuilder {
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
    fn generate(&mut self, mut mesh: Gd<ArrayMesh>) { //  -> Gd<ArrayMesh>
        godot_print!("Generating...");

        // Figure out how big we're going
        let aabb = self.get_aabb();
        let cell_size = self.get_cell_size_internal(aabb); // Fetch cell size of mesh
        let aabb = aabb.grow(cell_size[cell_size.max_axis_index()]);

        // Generate an island chunk buffer of max size 
        let mut sdf = [1.0 as f32; ChunkShape::USIZE];
        for i in 0u32..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            
            // Get field position at given point
            let sample_position = aabb.position + cell_size + Vector3::new(
                x as f32 * cell_size.x, y as f32 * cell_size.y, z as f32 * cell_size.z
            );

            sdf[i as usize] = -self.sample_at(sample_position) as f32; // Finally, sample and store value
        }
        
        // Create a SurfaceNet buffer, then create surface net
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &ChunkShape {}, [0; 3], [MAX_VOLUME_GRID_SIZE - 1; 3], &mut buffer);

        // Create a new mesh to put data in
        // let mesh = ArrayMesh::new_gd();

        // If our buffer is empty, do nothing
        if buffer.indices.is_empty() {
            godot_warn!("Island buffer is empty");
            // return mesh;
            return;
        }

        // Create a mesh data tool to start parsing mesh data
        godot_print!("Island buffer is NOT empty, building mesh...");

        if buffer.positions.len() != buffer.normals.len() {
            godot_warn!("Position buffer length does not match normal buffer length");
        }

               // Process and pipe data into Godot mesh
               let mut array_indices = PackedInt32Array::new();
        
               array_indices.resize(buffer.indices.len());
               for idx in 0..buffer.indices.len() {
                   array_indices.set(idx,  buffer.indices[idx] as i32);
               }
       
               let mut array_positions = PackedVector3Array::new();
               let mut array_normals = PackedVector3Array::new();
               array_positions.resize(buffer.positions.len());
               array_normals.resize(buffer.normals.len());
               for idx in 0..buffer.positions.len() {
                   let pos = buffer.positions[idx];
                   let normal = buffer.normals[idx];
                   array_positions.set(idx, Vector3::new(pos[0], pos[1], pos[2]) * cell_size + aabb.position);
                   array_normals.set(idx, Vector3::new(-normal[0], -normal[1], -normal[2]).normalized());
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
               surface_arrays.set(ArrayType::INDEX.ord() as usize, array_indices.to_variant());

        // Add our data to the mesh
        mesh.borrow_mut().add_surface_from_arrays(PrimitiveType::TRIANGLES, surface_arrays);

        // Return mesh through a signal (TODO, how do we unborrow the object)
        self.base_mut().emit_signal("generated".into(), &[]);
        godot_print!("All done!");

        // return mesh;
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
        return d;
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

    /// Returns the anticipated cell size of the volume
    #[func]
    fn get_cell_size(&self) -> Vector3 {
        return self.get_cell_size_internal(self.get_aabb());
    }
    fn get_cell_size_internal(&self, aabb: Aabb) -> Vector3 {
        return Vector3::new(aabb.size.x / ((MAX_VOLUME_GRID_SIZE-2) as f32), aabb.size.y / ((MAX_VOLUME_GRID_SIZE-2) as f32), aabb.size.z / ((MAX_VOLUME_GRID_SIZE-2) as f32));
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
    fn generated();
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
