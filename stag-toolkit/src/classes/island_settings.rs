use godot::classes::Material;
use godot::prelude::*;

/// General settings for an [IslandBuilder].
#[derive(GodotClass)]
#[class(init,base=Resource,tool)]
pub struct IslandBuilderSettings {
    /// Number of voxels to pad on each side of the [IslandBuilder] volume.
    #[export(range=(0.0, 6.0, or_greater))]
    #[init(val = 3)]
    voxel_padding: u32,
    /// Width/height/depth of a voxel. This is the approximate resolution of the resulting [IslandBuilder] mesh.
    #[export(range=(0.05, 1.0, 0.001, or_greater, suffix="m"))]
    #[init(val = 0.275)]
    voxel_size: f32,

    /// Rounding distance to apply to edges of Signed Distance Field primitives.
    #[export(range=(0.0,2.0,or_greater,suffix="m"))]
    #[init(val = 1.6)]
    sdf_edge_radius: f32,
    /// Number of smoothing iterations to apply to voxels immediately after sampling Signed Distance Fields.
    #[export(range=(0.0,20.0,or_greater))]
    #[init(val = 4)]
    sdf_smooth_iterations: u32,
    /// Radius of voxels to include in each smoothing pass applied immediately after sampling Signed Distance Fields.
    #[export(range=(0.0,5.0,or_greater))]
    #[init(val = 3)]
    sdf_smooth_radius_voxels: u32,
    /// Weighting of each smoothing pass applied immediately after sampling Signed Distance Fields.
    #[export(range=(0.0,1.0))]
    #[init(val = 0.95)]
    sdf_smooth_weight: f32,

    /// Frequency scale for striation noise on local X and Z axii.
    #[export(range=(0.0,10.0,0.001,or_greater))]
    #[init(val = 0.1)]
    striation_scale_xz: f32,
    /// Frequency scale for striation noise on local Y axis.
    #[export(range=(0.0,10.0,0.001,or_greater))]
    #[init(val = 10.0)]
    striation_scale_y: f32,
    /// Amplitude of striation noise on local X and Z axii.
    #[export(range=(0.0,10.0,0.001,or_greater,suffix="m"))]
    #[init(val = 0.2)]
    striation_amplitude_xz: f32,
    /// Amplitude of striation noise on local Y axis.
    #[export(range=(0.0,10.0,0.001,or_greater))]
    #[init(val = 0.01)]
    striation_amplitude_y: f32,

    /// Distance threshold for vertices to be merged for the visual mesh.
    #[export(range = (0.0, 1.0, 0.001, or_greater, suffix="m"))]
    #[init(val = 0.04)]
    mesh_vertex_merge_distance: f32,

    /// Whether to bake Ambient Occlusion to the Red channel.
    /// The Red channel defaults to 1.0 if Ambient Occlusion is not baked.
    /// @experimental: Ambient Occlusion still needs implementation.
    #[export]
    #[init(val = true)]
    ao_enabled: bool,
    /// Weighting value for linearly blending a base value of 1.0 with the baked Ambient Occlusion.
    /// @experimental: Ambient Occlusion still needs implementation.
    #[export(range=(0.0,1.0,0.001))]
    #[init(val = 1.0)]
    ao_strength: f32,

    /// Minimum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[export(range=(-1.0,1.0,0.001))]
    #[init(val=-0.2)]
    mask_dirt_minimum: f32,
    /// Maximum dot value for adding dirt gradation into the Green channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[export(range=(-1.0,1.0,0.001))]
    #[init(val = 0.8)]
    mask_dirt_maximum: f32,
    /// Arbitrary exponent to apply to the dirt gradient.
    #[export(range=(-5.0,5.0,0.001,or_greater))]
    #[init(val = 1.0)]
    mask_dirt_exponent: f32,

    /// Minimum dot value for adding sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[export(range=(-1.0,1.0,0.001))]
    #[init(val = 0.7)]
    mask_sand_minimum: f32,
    /// Maximum dot value for baking sand gradation into the Blue channel.
    /// The dot value is computed from a dot product of the triangle's normal to the local-space up vector.
    #[export(range=(-1.0,1.0,0.001))]
    #[init(val = 1.0)]
    mask_sand_maximum: f32,
    /// Arbitrary exponent to apply to the sand gradient.
    #[export(range=(-5.0,5.0,0.001,or_greater))]
    #[init(val = 3.0)]
    mask_sand_exponent: f32,

    /// X frequency scale when sampling perlin noise for baking into the Alpha channel.
    #[export(range=(0.0,2.0,0.001,or_greater))]
    #[init(val = 0.75)]
    mask_perlin_scale_x: f32,
    /// Y frequency scale when sampling perlin noise for baking into the Alpha channel.
    #[export(range=(0.0,2.0,0.001,or_greater))]
    #[init(val = 0.33)]
    mask_perlin_scale_y: f32,
    /// Z frequency scale when sampling perlin noise for baking into the Alpha channel.
    #[export(range=(0.0,2.0,0.001,or_greater))]
    #[init(val = 0.75)]
    mask_perlin_scale_z: f32,

    /// Whether to merge collision vertices on non-manifold edges.
    #[export]
    #[init(val = false)]
    collision_merge_nonmanifold_edges: bool,
    /// Whether to perform collision decimation on non-manifold edges.
    #[export]
    #[init(val = false)]
    collision_decimate_nonmanifold_edges: bool,
    /// Distance threshold for vertices to be merged for the collision hull.
    #[export(range = (0.0, 1.0, 0.001, or_greater, suffix="m"))]
    #[init(val = 0.15)]
    collision_vertex_merge_distance: f32,
    /// Angular threshold for decimating triangles used in physics collisions. In degrees.
    /// If zero, mesh decimation will not occur.
    #[export(range=(0.0, 179.9, 0.001, or_greater, degrees))]
    #[init(val = 2.0)]
    collision_decimation_angle: f32,
    /// Maximum number of iterations for performing collision mesh decimation.
    /// The mesh will automatically stop decimating if nothing changes after an iteration.
    #[export(range=(0.0,500.0,1.0,or_greater))]
    #[init(val = 100)]
    collision_decimation_iterations: u32,

    /// Approximate physical density of material to use when calculating mass.
    /// Kilograms per meter cubed.
    #[export(range=(0.0,50.0,0.01,or_greater,suffix="kg/m³"))]
    #[init(val = 23.23)]
    physics_density: f32,
    /// Approximate health density of material to use when calculating health.
    /// Hit Points per meter cubed.
    #[export(range = (0.001,10.0,0.001, or_greater, suffix="HP/m³"))]
    #[init(val = 0.75)]
    physics_health_density: f32,

    /// Material to apply to baked meshes.
    #[export]
    #[init(val=None)]
    material_baked: Option<Gd<Material>>,
    /// Material to apply to preview meshes.
    #[export]
    #[init(val=None)]
    material_preview: Option<Gd<Material>>,

    base: Base<Resource>,
}

#[godot_api]
impl IslandBuilderSettings {}
