use godot::engine::Json;
use godot::prelude::*;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use json::JsonValue;
use utilities::{exp, log};
use utilities::maxf;

use core::fmt;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}

// Utility Functions
pub fn rotate_xyz(v: Vector3, euler: Vector3) -> Vector3 {
    // Rotation Order: YXZ
    return v.rotated(
        Vector3::UP, euler.y
    ).rotated(
        Vector3::RIGHT, euler.x
    ).rotated(
        Vector3::FORWARD, euler.z
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
        let Self { shape, position, rotation, scale, radius, .. } = &self;

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

#[derive(GodotClass)]
#[class(base=Node3D,tool)]
struct IslandBuilder {
    #[export]
    shapes: Array<Gd<IslandBuilderShape>>,
    base: Base<Node3D>,
}
#[godot_api]
impl INode3D for IslandBuilder {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            shapes: Array::new(),
            base,
        }
    }
}
#[godot_api]
impl IslandBuilder {
    #[func]
    fn generate(&mut self) {
        godot_print!("Generate!");

        self.base_mut().emit_signal("generated".into(), &[]);
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
    fn generated();
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
