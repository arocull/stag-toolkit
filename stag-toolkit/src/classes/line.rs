use crate::math::types::ToVector3;
use glam::Vec3;
use godot::builtin::VarDictionary;
use godot::classes::rendering_server::PrimitiveType;
use godot::classes::{Gradient, IMesh, Material, Mesh};
use godot::prelude::*;

// #[derive(Copy, Clone, PartialEq, ExposeSettings)]
// #[settings_resource_from(StagLineMeshSettings, Resource)]
// pub struct Settings {
//     /// Maximum number of points the line can have.
//     /// This is used to determine shader buffer sizes.
//     #[setting(default = 2, min = 2.0, max = 100.0, soft_max)]
//     pub max_points: u32,

//     /// Number of sides of the line mesh.
//     /// If the number if sides is equal to 2, the line is forcibly aligned with the camera view vector.
//     #[setting(default = 2, min = 2.0, max = 64.0, soft_max)]
//     pub sides: u32,

//     /// Radius of the line.
//     /// During mesh generation, the line will be infinitely thin.
//     /// This value is to be handled by the shader.
//     #[setting(
//         default = 0.1,
//         min = 0.0,
//         max = 1.0,
//         incr = 0.001,
//         soft_max,
//         unit = "m"
//     )]
//     pub radius: f32,
// }

#[derive(GodotConvert, Var, Export, Default, Clone)]
#[godot(via = i8)]
pub enum StagLineMeshUVMode {
    /// UV Y coordinate is the current point index divided by (the total number of points minus one).
    /// This mode is fastest because a length does not need to be computed beforehand.
    /// Range of [0, 1].
    #[default]
    Factor,
    /// UV Y coordinate is the current traveled distance along the line, divided by the total length of the line.
    /// Range of [0, 1].
    FactorProportional,
    /// UV Y coordinate is the current traveled distance along the line.
    /// Range of [0, infinity)
    Length,
}

#[derive(GodotConvert, Var, Export, Default, Clone)]
#[godot(via = i8)]
pub enum StagLineMeshBoundingMode {
    /// All points in the line are included in its bounding box.
    #[default]
    All,
    /// Only the two end points of the line are included in its bounding box.
    /// This is much faster to compute, but the bounding box may not encapsulate all points.
    EndPoints,
}

#[derive(GodotClass)]
#[class(base=Mesh,tool)]
/// Generates a mesh along a provided list of points.
///
/// @experimental: Initial implementation used for SimulatedRope.
pub struct StagLineMesh {
    /// Number of sides on the generated mesh.
    /// 2 sides generates a ribbon, otherwise a cylinder is generated.
    #[export(range=(2.0,64.0,or_greater))]
    pub sides: i32,

    /// Radius of the line.
    /// **Changing this property will regenerate the entire line mesh** (if `dynamic` is false).
    #[export(range=(0.01,1.0,0.001,or_greater,suffix="m"))]
    #[var(set=set_radius)]
    pub radius: f32,

    /// Local-space vector used to determine the alignment of the mesh vertices.
    ///
    /// If the direction from a point to the next point is directly parallel with this, the generated line may have artifacts.
    /// You can resolve this by enabling `dynamic` and aligning the points to your camera using a shader.
    #[export]
    pub cross: Vector3,

    /// Determines the UV generation mode for the mesh.
    /// The X coordinate of the UV will always be in the range [0, 1] and runs perpendicular to the direction of the line.
    #[export]
    pub uv_mode: StagLineMeshUVMode,

    /// If true, the line mesh assumes that width and face orientation will be handled by the shader material.
    /// This modifies how the mesh is generated (i.e. triangles become infinitely thin), so the line will not be visible without a proper shader.
    /// However, this also reduces the computation time for generating a mesh.
    #[export]
    pub dynamic: bool,

    /// Points on the line.
    /// **Changing this property will regenerate the entire line mesh.**
    #[export]
    #[var(set=set_points)]
    pub points: PackedVector3Array,

    /// Optional gradient that will be used to set vertex colors along the line.
    /// If no gradient is provided, vertex colors not baked into the mesh data.
    #[export]
    pub gradient: Option<Gd<Gradient>>,

    /// What material to use for the line mesh by default.
    #[export]
    pub material: Option<Gd<Material>>,

    /// Bounding box from current generation.
    bounds: Aabb,

    base: Base<Mesh>,
}

#[godot_api]
// Must implement these methods:
// https://godot-rust.github.io/docs/gdext/master/godot/classes/trait.IMesh.html
impl IMesh for StagLineMesh {
    fn init(base: Base<Mesh>) -> Self {
        let mut points = PackedVector3Array::new();
        points.push(Vector3::ZERO);
        points.push(Vector3::FORWARD);

        Self {
            sides: 2,
            radius: 0.05,
            cross: Vector3::UP,
            uv_mode: StagLineMeshUVMode::FactorProportional,
            dynamic: false,

            points: points,
            gradient: None,
            material: None,

            bounds: Aabb::default(),
            base,
        }
    }

    fn surface_get_array_len(&self, index: i32) -> i32 {
        match index {
            0 => todo!(), // TODO: determine what this even is
            _ => 0,
        }
    }

    fn surface_get_array_index_len(&self, index: i32) -> i32 {
        match index {
            0 => todo!(),
            _ => 0,
        }
    }

    // TODO: do mesh conversion, and/or cache this on generation
    fn surface_get_arrays(&self, _index: i32) -> AnyArray {
        todo!()
    }

    fn surface_get_blend_shape_arrays(&self, _index: i32) -> Array<AnyArray> {
        Array::new() // We do not use blend shapes
    }

    // TODO: figure this out
    fn surface_get_lods(&self, _index: i32) -> AnyDictionary {
        VarDictionary::new().upcast_any_dictionary()
    }

    // TODO: no idea what this is, is it array format??
    // https://docs.godotengine.org/en/stable/classes/class_mesh.html#enum-mesh-arrayformat
    fn surface_get_format(&self, _index: i32) -> u32 {
        1
    }

    fn surface_set_material(&mut self, index: i32, material: Option<Gd<Material>>) {
        if index == 0 {
            self.material = material;
        }
    }

    fn surface_get_material(&self, index: i32) -> Option<Gd<Material>> {
        if index == 0 {
            return self.material.clone();
        }
        None
    }

    // https://docs.godotengine.org/en/stable/classes/class_mesh.html#enum-mesh-primitivetype
    // TODO: investigate TRIANGLE_STRIPS
    fn surface_get_primitive_type(&self, _index: i32) -> u32 {
        PrimitiveType::TRIANGLES.ord() as u32
    }

    fn get_surface_count(&self) -> i32 {
        return 1;
    }

    fn get_blend_shape_count(&self) -> i32 {
        return 0;
    }

    fn get_blend_shape_name(&self, _index: i32) -> StringName {
        return StringName::default();
    }

    fn set_blend_shape_name(&mut self, _index: i32, _name: StringName) {
        // noop
    }

    fn get_aabb(&self) -> Aabb {
        return self.bounds;
    }
}

#[godot_api]
impl StagLineMesh {
    /// Regenerates the line mesh.
    #[func]
    fn redraw(&mut self) {
        self.redraw_internal();
    }

    /// Regenerates the line mesh, but not exposed to Godot, so binding cannot be locked.
    fn redraw_internal(&mut self) {
        let points: Vec<Vec3> = self.points.to_vector3();
        self.redraw_with_points(&points);
    }

    #[func]
    fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
        self.redraw_internal();
    }

    #[func]
    fn set_points(&mut self, points: PackedVector3Array) {
        self.points = points;
        self.redraw_internal();
    }

    /// Redraws the lines with the given array of points.
    /// Does NOT update `points`.
    /// Use this to use other Rust-side methods to directly re-render the rope, without worrying about copying memory.
    pub fn redraw_with_points(&mut self, points: &[Vec3]) {
        todo!()
    }
}
