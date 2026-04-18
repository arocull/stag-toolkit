use crate::math::bounding_box::BoundingBox;
use crate::math::types::{ToVector3, gdmath, gdmath::ToColor};
use crate::mesh::godot::GodotSurfaceArrays;
use crate::mesh::strip::TriangleStripMesh;
use glam::{Vec2, Vec3, Vec4};
use godot::builtin::VarDictionary;
use godot::classes::rendering_server::{ArrayFormat, PrimitiveType};
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

#[derive(GodotConvert, Var, Export, Default, Clone, PartialEq)]
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

    /// Generated Godot-facing mesh.
    surface_arrays: GodotSurfaceArrays,

    /// Bounding box from current generation.
    bounds: Aabb,

    has_colors: bool,

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

            surface_arrays: GodotSurfaceArrays::default(),
            bounds: Aabb::default(),
            has_colors: false,
            base,
        }
    }

    fn surface_get_array_len(&self, index: i32) -> i32 {
        match index {
            0 => (self.points.len() * 2) as i32, // TODO: determine what this even is
            _ => 0,
        }
    }

    fn surface_get_array_index_len(&self, index: i32) -> i32 {
        match index {
            0 => (self.points.len() * 2) as i32,
            _ => 0,
        }
    }

    // TODO: do mesh conversion, and/or cache this on generation
    fn surface_get_arrays(&self, _index: i32) -> AnyArray {
        println!("surface get arrays called for index {_index}");
        (&self.surface_arrays.surface_arrays)
            .clone()
            .upcast_any_array()
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
        if self.has_colors {
            return (ArrayFormat::INDEX.ord()
                | ArrayFormat::VERTEX.ord()
                | ArrayFormat::NORMAL.ord()
                | ArrayFormat::COLOR.ord()
                | ArrayFormat::TEX_UV.ord()) as u32;
        }

        (ArrayFormat::INDEX.ord()
            | ArrayFormat::VERTEX.ord()
            | ArrayFormat::NORMAL.ord()
            | ArrayFormat::TEX_UV.ord()) as u32
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
    fn surface_get_primitive_type(&self, _index: i32) -> u32 {
        PrimitiveType::TRIANGLE_STRIP.ord() as u32
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
        let mut strip = TriangleStripMesh::new(points.len() * 2);

        let mut total_length: f32 = 1.0f32; // Total length of the line (if calculated)
        let mut current_length: f32 = 0.0f32;
        if self.uv_mode == StagLineMeshUVMode::FactorProportional {
            for i in 1..points.len() {
                total_length += (points[i] - points[i - 0]).length();
            }
        }

        // Create the starting corner
        strip.push(points[0], (points[1] - points[0]).normalize(), Vec2::ZERO);

        for i in 0..points.len() - 1 {
            let offset = points[i + 1] - points[i];
            let length = offset.length();
            strip.push(
                points[i],
                offset / length,
                Vec2::new(((i + 1) % 2) as f32, current_length / total_length),
            );
            current_length += length;
        }

        // Cap off line with end corner
        strip.push(
            points[points.len() - 1],
            (points[points.len() - 1] - points[points.len() - 2]).normalize(),
            Vec2::new(1.0f32, current_length / total_length),
        );

        // Handle color sampling at as necessary
        let mut colors: Option<PackedColorArray> = None;
        if let Some(mut gradient) = self.gradient.clone() {
            let mut color_samples = PackedColorArray::new();
            color_samples.resize(strip.uv1.len());

            for (idx, uv) in strip.uv1.iter().enumerate() {
                color_samples[idx] = gradient.sample(uv.y / total_length);
            }
            colors = Some(color_samples);
            self.has_colors = true;
        } else {
            self.has_colors = false;
        }

        self.surface_arrays = GodotSurfaceArrays::from_tristrip(&strip, colors);

        self.bounds = BoundingBox::from(&strip.position)
            .expand_margin(self.radius)
            .to_aabb();

        println!("regenerated mesh!!");
    }
}
