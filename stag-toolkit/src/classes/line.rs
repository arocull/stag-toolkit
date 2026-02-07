use glam::Vec3;
use stag_toolkit_codegen::{ExposeSettings, settings_resource_from};
use {crate::math::types::ToVector3, godot::prelude::*};

#[derive(Copy, Clone, PartialEq, ExposeSettings)]
#[settings_resource_from(StagLineMeshSettings, Resource)]
pub struct Settings {
    /// Maximum number of points the line can have.
    /// This is used to determine shader buffer sizes.
    #[setting(default = 2, min = 2.0, max = 100.0, soft_max)]
    pub max_points: u32,

    /// Number of sides of the line mesh.
    /// If the number if sides is equal to 2, the line is forcibly aligned with the camera view vector.
    #[setting(default = 2, min = 2.0, max = 64.0, soft_max)]
    pub sides: u32,

    /// Radius of the line.
    /// During mesh generation, the line will be infinitely thin.
    /// This value is to be handled by the shader.
    #[setting(
        default = 0.1,
        min = 0.0,
        max = 1.0,
        incr = 0.001,
        soft_max,
        unit = "m"
    )]
    pub radius: f32,
}

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct StagLineMesh {
    #[export]
    pub settings: OnEditor<Gd<StagLineMeshSettings>>,

    /// An ordered list of 3D points describing the path the resulting line should take.
    points: Vec<Vec3>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for StagLineMesh {
    fn init(base: Base<Node3D>) -> Self {
        Self {
            base,
            settings: OnEditor::default(),
            points: vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)],
        }
    }
}

#[godot_api]
impl StagLineMesh {
    #[func]
    fn redraw(&mut self) {}
}
