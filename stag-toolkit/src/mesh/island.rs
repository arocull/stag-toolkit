use crate::math::sdf::Shape;
use godot::classes::TriangleMesh;

#[derive(Copy, Clone, PartialEq)]
pub struct Settings {
    /// Number of voxels to pad on each side of the [IslandBuilder] volume.
    pub voxel_padding: u32,
    /// Width/height/depth of a voxel. This is the approximate resolution of the resulting [IslandBuilder] mesh.
    pub voxel_size: f32,

    /// Rounding distance to apply to edges of Signed Distance Field primitives.
    pub sdf_edge_radius: f32,
    /// Number of smoothing iterations to apply to voxels immediately after sampling Signed Distance Fields.
    pub sdf_smooth_iterations: u32,
    /// Radius of voxels to include in each smoothing pass applied immediately after sampling Signed Distance Fields.
    pub sdf_smooth_radius_voxels: u32,
    /// Weighting of each smoothing pass applied immediately after sampling Signed Distance Fields.
    pub sdf_smooth_weight: f32,

    /// Frequency scale for striation noise on local X and Z axii.
    pub striation_scale_xz: f32,
    /// Frequency scale for striation noise on local Y axis.
    pub striation_scale_y: f32,
    /// Amplitude of striation noise on local X and Z axii.
    pub striation_amplitude_xz: f32,
    /// Amplitude of striation noise on local Y axis.
    pub striation_amplitude_y: f32,

    /// Distance threshold for vertices to be merged for the visual mesh.
    pub mesh_vertex_merge_distance: f32,

    /// Whether to bake Ambient Occlusion to the Red channel.
    /// The Red channel defaults to 1.0 if Ambient Occlusion is not baked.
    pub ao_enabled: bool,
    /// Weighting value for linearly blending a base value of 1.0 with the baked Ambient Occlusion.
    pub ao_strength: f32,

    /// Minimum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_dirt_minimum: f32,
    /// Maximum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_dirt_maximum: f32,
    /// Arbitrary exponent to apply to the dirt gradient.
    pub mask_dirt_exponent: f32,

    /// Minimum dot value for adding sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_sand_minimum: f32,
    /// Maximum dot value for baking sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    pub mask_sand_maximum: f32,
    /// Arbitrary exponent to apply to the sand gradient.
    pub mask_sand_exponent: f32,

    /// X frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_x: f32,
    /// Y frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_y: f32,
    /// Z frequency scale when sampling perlin noise for baking into the Alpha channel.
    pub mask_perlin_scale_z: f32,

    /// Whether to merge collision vertices on non-manifold edges.
    pub collision_merge_nonmanifold_edges: bool,
    /// Whether to perform collision decimation on non-manifold edges.
    pub collision_decimate_nonmanifold_edges: bool,
    /// Distance threshold for vertices to be merged for the collision hull.
    pub collision_vertex_merge_distance: f32,
    /// Angular threshold for decimating triangles used in physics collisions. In degrees.
    /// If zero, mesh decimation will not occur.
    pub collision_decimation_angle: f32,
    /// Maximum number of iterations for performing collision mesh decimation.
    /// The mesh will automatically stop decimating if nothing changes after an iteration.
    pub collision_decimation_iterations: u32,
}

pub struct Data {
    settings: Settings,

    shapes: Vec<Shape>,

    mesh_preview: Option<TriangleMesh>,
    mesh_baked: Option<TriangleMesh>,
    hulls: Vec<TriangleMesh>,

    /// Approximate volume of the Island.
    volume: f32,
}

impl Data {
    /// Creates a new data set for building from.
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            shapes: vec![],
            mesh_preview: None,
            mesh_baked: None,
            hulls: vec![],
            volume: 0.0,
        }
    }

    /// Clears all generated data.
    pub fn dirty(&mut self) {
        self.mesh_preview = None;
        self.mesh_baked = None;
        self.hulls.clear();
        self.volume = 0.0;
    }

    /// Updates the settings, dirtying the data if changed.
    pub fn change_settings(&mut self, settings: Settings) {
        if self.settings != settings {
            self.settings = settings;
            self.dirty();
        }
    }
}
