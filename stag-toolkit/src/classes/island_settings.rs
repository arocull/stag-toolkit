use crate::mesh::island::{
    IslandBuilderSettingsCollision, IslandBuilderSettingsMesh, IslandBuilderSettingsVoxels,
};
use godot::classes::Material;
use godot::prelude::*;

/// General settings for an [IslandBuilder].
#[derive(GodotClass)]
#[class(init,base=Resource,tool)]
pub struct IslandBuilderSettings {
    /// Voxel generation settings.
    #[export]
    voxels: Option<Gd<IslandBuilderSettingsVoxels>>,
    /// Mesh generation settings.
    #[export]
    mesh: Option<Gd<IslandBuilderSettingsMesh>>,
    /// Collision settings.
    #[export]
    collision: Option<Gd<IslandBuilderSettingsCollision>>,

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
impl IslandBuilderSettings {
    /// Emitted when the `voxels` settings resource is changed, or a voxels setting changes.
    #[signal]
    fn setting_changed_voxels();

    /// Emitted when the `mesh` settings resource is changed, or a mesh setting changes.
    #[signal]
    fn setting_changed_mesh();

    /// Emitted when the `collision` settings resource is changed, or a collision setting changes.
    #[signal]
    fn setting_changed_collision();
}
