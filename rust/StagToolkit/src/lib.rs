use godot::engine::Json;
use godot::prelude::*;
use godot::engine::Node3D;
use godot::engine::Resource;
use json::object;
use json::JsonValue;

use core::fmt;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}


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
impl BuilderShape {
    fn from_string(string_value: &str) -> BuilderShape {
        return match string_value {
            "sphere" => BuilderShape::Sphere,
            _ => BuilderShape::Box,
        }

    }
}


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
    base: Base<Resource>,
}
#[godot_api]
impl IResource for IslandBuilderShape {
    fn init(base: Base<Resource>) -> Self {
        godot_print!("Hello shape testing");

        Self {
            shape: BuilderShape::Box,
            position: Vector3::ZERO,
            rotation: Vector3::ZERO,
            scale: Vector3::ONE,
            base,
        }
    }
    fn to_string(&self) -> GString {
        let Self { shape, position, rotation, scale, .. } = &self;

        let obj = object! {
            "shape": stringify!(shape),
            "position": [position.x, position.y, position.z],
            "rotation": [rotation.x, rotation.y, rotation.z],
            "scale": [scale.x, scale.y, scale.z],
        };

        return json::stringify(obj).into();
    }
}
// #[godot_api]
// impl IslandBuilderShape {
//     #[func]
//     fn from_json(obj: Dictionary) -> Gd<Self> {
//         let shape = BuilderShape::from_string(obj.get("shape").unwrap().into());
//         let position = obj.get("position").unwrap();
//         let rotation = obj.get("position").unwrap();
//         let scale = obj.get("position").unwrap();
//         Gd::from_object(Self {
//             shape,
//             position,
//             rotation,
//             scale,
//             base,
//         })
//     }
// }

#[derive(GodotClass)]
#[class(base=Node3D)]
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
    fn generate(&self) {
        godot_print!("Generate!");
    }
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
