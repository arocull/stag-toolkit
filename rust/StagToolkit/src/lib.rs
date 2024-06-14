use godot::engine::mesh::ArrayType;
use godot::engine::mesh::PrimitiveType;
use godot::engine::ArrayMesh;
use godot::engine::Material;
use godot::engine::MeshDataTool;
use godot::obj::WithBaseField;
use godot::prelude::*;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use utilities::ceili;
use utilities::maxi;
use utilities::push_warning;
use utilities::{exp, log};
use utilities::maxf;
use fast_surface_nets::ndshape::{ConstShape, ConstShape3u32};
use fast_surface_nets::{surface_nets, SurfaceNetsBuffer};

use core::fmt;
use std::borrow::BorrowMut;
use std::cmp::min;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}

// Utility Functions
pub fn rotate_xyz(v: Vector3, euler: Vector3) -> Vector3 {
    // Rotation Order: YXZ
    return v.rotated(
        Vector3::FORWARD, euler.z
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
    fn distance(&mut self, position: Vector3) -> f64 {
        let offset = rotate_xyz(self.position - position, self.rotation);

        match self.shape {
            BuilderShape::Sphere => {
                return (offset * self.scale).length() as f64 - self.radius; // distance minus radius
            },
            BuilderShape::Box => { // SDF rounded box
                let q = offset.abs() - self.scale;
                return q.coord_max(Vector3::ZERO).length() as f64 + maxf(
                        maxf(maxf(q.x as f64, q.y as f64), q.z as f64), 0.0
                    ) - self.radius;
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
const MAX_VOLUME_GRID_SIZE: u32 = 32;
type ChunkShape = ConstShape3u32<MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE, MAX_VOLUME_GRID_SIZE>;

#[derive(GodotClass)]
#[class(base=Node3D,tool)]
struct IslandBuilder {
    #[export]
    shapes: Array<Gd<IslandBuilderShape>>,
    #[export]
    cell_size: f32, // Size of volume cell, in meters 
    #[export]
    volume_padding: f32,
    #[export]
    island_material: Option<Gd<Material>>,

    base: Base<Node3D>,
}
#[godot_api]
impl INode3D for IslandBuilder {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            shapes: Array::new(),
            cell_size: 0.1,
            volume_padding: 0.2,
            island_material: None,
            base,
        }
    }
}
#[godot_api]
impl IslandBuilder {
    #[func]
    fn generate(&mut self, mut mesh: Gd<ArrayMesh>) { //  -> Gd<ArrayMesh>
        godot_print!("Generating...");

        // Figure out how big we're going
        let aabb = self.get_aabb().expand(Vector3::splat(self.volume_padding));
        let cells_x = ceili((aabb.size.x / self.cell_size) as f64);
        let cells_y = ceili((aabb.size.y / self.cell_size) as f64);
        let cells_z = ceili((aabb.size.z / self.cell_size) as f64);
        let max_cells = min(maxi(maxi(cells_x, cells_y), cells_z) as u32, MAX_VOLUME_GRID_SIZE - 1);

        // Generate an island chunk buffer of max size 
        let mut sdf = [1.0 as f32; ChunkShape::USIZE];
        for i in 0u32..ChunkShape::SIZE {
            let [x, y, z] = ChunkShape::delinearize(i);
            
            // Get field position at given point
            let pos = aabb.position + Vector3::new(
                x as f32 * self.cell_size, y as f32 * self.cell_size, z as f32 * self.cell_size
            );

            // Accumulate signed distance field values at this point
            let mut d = 1.0;
            for mut shape in self.shapes.iter_shared() {
                // Smooth union new shape with current one
                d = sdf_smooth_min(d, shape.bind_mut().distance(pos), 3.0);
            }
            sdf[i as usize] = d as f32; // Finally, store value
        }
        
        // Create a SurfaceNet buffer, then create surface net
        let mut buffer = SurfaceNetsBuffer::default();
        surface_nets(&sdf, &ChunkShape {}, [0; 3], [max_cells; 3], &mut buffer);

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
        for idx in 0..buffer.indices.len()-1 {
            array_indices.set(idx, buffer.indices[idx] as i32);
        }

        let mut array_positions = PackedVector3Array::new();
        let mut array_normals = PackedVector3Array::new();
        array_positions.resize(buffer.positions.len());
        array_normals.resize(buffer.normals.len());
        for idx in 0..buffer.positions.len()-1 {
            let pos = buffer.positions[idx];
            let normal = buffer.normals[idx];
            array_positions.set(idx, Vector3::new(pos[0], pos[1], pos[2]) + aabb.position);
            array_normals.set(idx, Vector3::new(normal[0], normal[1], normal[2]).normalized());
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

    #[func]
    fn get_aabb(&self) -> Aabb {
        let mut aabb = Aabb{position: Vector3::ZERO, size: Vector3::ZERO};

        for shape in self.shapes.iter_shared() {
            aabb = aabb.merge(&shape.bind().get_aabb());
        }

        return aabb;
    }

    #[func]
    fn smooth_union(a: f64, b: f64, k: f64) -> f64 {
        return sdf_smooth_min(a, b, k);
    }

    #[signal]
    fn generated(); // mesh: Gd<ArrayMesh>
}


// HELLO WORLD
use godot::engine::Sprite2D;

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Player {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>
}

use godot::engine::ISprite2D;

#[godot_api]
impl ISprite2D for Player {
    fn init(base: Base<Sprite2D>) -> Self {
        godot_print!("Hello, world!");

        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        let radians = (self.angular_speed * delta) as f32;
        // self.base().rotate(radians);
        self.base_mut().rotate(radians);

        let rotation = self.base().get_rotation();
        let velocity = Vector2::UP.rotated(rotation) * self.speed as f32;
        self.base_mut().translate(velocity * delta as f32);
    }
}

#[godot_api]
impl Player {
    #[func]
    fn increase_speed(&mut self, amount: f64) {
        self.speed += amount;
        self.base_mut().emit_signal("speed_increased".into(), &[]);
    }

    #[signal]
    fn speed_increased();
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
