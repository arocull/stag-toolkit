use crate::mesh::island::{
    IslandBuilderSettingsCollision, IslandBuilderSettingsMesh, IslandBuilderSettingsVoxels,
    SettingsCollision, SettingsMesh, SettingsVoxels,
};
use godot::classes::Material;
use godot::prelude::*;
use godot::register::ConnectHandle;

/// General settings for an [IslandBuilder].
/// @experimental: This is recently refactored code, and probably bound for more refactoring.
#[derive(GodotClass)]
#[class(init,base=Resource,tool)]
pub struct IslandBuilderSettings {
    /// Voxel generation settings, used for generating the base data.
    /// If no settings are provided, sensible defaults are used.
    #[var(get, set=set_voxels)]
    #[export]
    voxels: Option<Gd<IslandBuilderSettingsVoxels>>,
    /// Mesh generation settings, used for generating the renderable mesh.
    /// If no settings are provided, sensible defaults are used.
    #[var(get, set=set_mesh)]
    #[export]
    mesh: Option<Gd<IslandBuilderSettingsMesh>>,
    /// Collision settings, used for generating the physics collision.
    /// If no settings are provided, sensible defaults are used.
    #[var(get, set=set_collision)]
    #[export]
    collision: Option<Gd<IslandBuilderSettingsCollision>>,

    /// Approximate physical density of material to use when calculating mass.
    /// Kilograms per meter cubed.
    ///
    /// If the [IslandBuilder] target is a [RigidBody3D],
    /// the mass is automatically applied to the target upon collision generation.
    #[var(get, set = set_physics_density)]
    #[export(range=(0.001,50.0,0.001,or_greater,suffix="kg/m³"))]
    #[init(val = 23.23)]
    physics_density: f32,
    /// Approximate health density of material to use when calculating health.
    /// Hit Points per meter cubed.
    ///
    /// If the parent of the [IslandBuilder] target has a method named `set_maximum_health`,
    /// the method is called upon collision generation with the computed health value.
    #[var(get, set = set_physics_health_density)]
    #[export(range = (0.001,10.0,0.001, or_greater, suffix="HP/m³"))]
    #[init(val = 0.75)]
    physics_health_density: f32,

    /// Optional material to apply to baked/finalized meshes.
    #[var(get, set = set_material_baked)]
    #[export]
    #[init(val=None)]
    material_baked: Option<Gd<Material>>,
    /// Optional material to apply to preview meshes, such as the real-time preview in-editor.
    #[var(get,set = set_material_preview)]
    #[export]
    #[init(val=None)]
    material_preview: Option<Gd<Material>>,

    #[var(get,set = set_collision_color)]
    #[export]
    #[init(val=Color::from_rgba(1.0, 0.0, 0.667, 1.0))]
    collision_color: Color,

    #[var(get,set = set_render_layers,hint=LAYERS_3D_RENDER)]
    #[export]
    #[init(val = 5)]
    render_layers: u32,

    /// A signal connection handle for disconnecting when the [IslandBuilderSettingsVoxels] resource is reassigned.
    #[init(val=None)]
    handle_voxels: Option<ConnectHandle>,
    /// A signal connection handle for disconnecting when the [IslandBuilderSettingsMesh] resource is reassigned.
    #[init(val=None)]
    handle_mesh: Option<ConnectHandle>,
    /// A signal connection handle for disconnecting when the [IslandBuilderSettingsCollision] resource is reassigned.
    #[init(val=None)]
    handle_collision: Option<ConnectHandle>,

    base: Base<Resource>,
}

#[godot_api]
impl IslandBuilderSettings {
    #[func]
    fn set_voxels(&mut self, voxels: Option<Gd<IslandBuilderSettingsVoxels>>) {
        // Disconnect the old event handle if present
        if let Some(connect_handle) = self.handle_voxels.take()
            && connect_handle.is_connected()
        {
            connect_handle.disconnect();
        }

        let changed = self.voxels != voxels;
        self.voxels = voxels.clone();

        // Connect setting changed events
        if let Some(voxels) = voxels {
            let settings = self.to_gd();
            self.handle_voxels = Some(
                voxels
                    .signals()
                    .changed()
                    .builder()
                    .connect_other_mut(&settings, Self::notify_changed_voxels),
            );
        }

        if changed {
            self.notify_changed_voxels();
        }
    }

    #[func]
    fn set_mesh(&mut self, mesh: Option<Gd<IslandBuilderSettingsMesh>>) {
        // Disconnect the old event handle if present
        if let Some(connect_handle) = self.handle_mesh.take()
            && connect_handle.is_connected()
        {
            connect_handle.disconnect();
        }

        let changed = self.mesh != mesh;
        self.mesh = mesh.clone();

        // Connect setting changed events
        if let Some(mesh) = mesh {
            let settings = self.to_gd();
            self.handle_mesh = Some(
                mesh.signals()
                    .changed()
                    .builder()
                    .connect_other_mut(&settings, Self::notify_changed_mesh),
            );
        }

        if changed {
            self.notify_changed_mesh();
        }
    }

    #[func]
    fn set_collision(&mut self, collision: Option<Gd<IslandBuilderSettingsCollision>>) {
        // Disconnect the old event handle if present
        if let Some(connect_handle) = self.handle_collision.take()
            && connect_handle.is_connected()
        {
            connect_handle.disconnect();
        }

        let changed = self.collision != collision;
        self.collision = collision.clone();

        // Connect setting changed events
        if let Some(collision) = collision {
            let settings = self.to_gd();
            self.handle_collision = Some(
                collision
                    .signals()
                    .changed()
                    .builder()
                    .connect_other_mut(&settings, Self::notify_changed_collision),
            );
        }

        if changed {
            self.notify_changed_collision();
        }
    }

    #[func]
    fn set_physics_density(&mut self, physics_density: f32) {
        self.physics_density = physics_density;
        self.base_mut().emit_changed();
    }

    #[func]
    fn set_physics_health_density(&mut self, physics_health_density: f32) {
        self.physics_health_density = physics_health_density;
        self.base_mut().emit_changed();
    }

    #[func]
    fn set_material_baked(&mut self, material: Option<Gd<Material>>) {
        self.material_baked = material;
        self.base_mut().emit_changed();
    }

    #[func]
    fn set_material_preview(&mut self, material: Option<Gd<Material>>) {
        self.material_preview = material;
        self.base_mut().emit_changed();
    }

    #[func]
    fn set_collision_color(&mut self, collision_color: Color) {
        self.collision_color = collision_color;
        self.base_mut().emit_changed();
    }

    #[func]
    fn set_render_layers(&mut self, render_layers: u32) {
        self.render_layers = render_layers;
        self.base_mut().emit_changed();
    }

    /// Emits signals `changed` and `setting_changed_voxels`.
    #[func]
    fn notify_changed_voxels(&mut self) {
        self.base_mut().emit_changed();
        self.signals().setting_changed_voxels().emit();
    }

    /// Emits signals `changed` and `setting_changed_mesh`.
    #[func]
    fn notify_changed_mesh(&mut self) {
        self.base_mut().emit_changed();
        self.signals().setting_changed_mesh().emit();
    }

    /// Emits signals `changed` and `setting_changed_collision`.
    #[func]
    fn notify_changed_collision(&mut self) {
        self.base_mut().emit_changed();
        self.signals().setting_changed_collision().emit();
    }

    /// Emitted when the `voxels` settings resource is changed, or a voxels setting changes.
    #[signal]
    fn setting_changed_voxels();

    /// Emitted when the `mesh` settings resource is changed, or a mesh setting changes.
    #[signal]
    fn setting_changed_mesh();

    /// Emitted when the `collision` settings resource is changed, or a collision setting changes.
    #[signal]
    fn setting_changed_collision();

    pub fn get_internal_voxel_settings(&self) -> SettingsVoxels {
        if let Some(settings) = self.voxels.clone() {
            return settings.bind().to_struct();
        }
        SettingsVoxels::default()
    }

    pub fn get_internal_mesh_settings(&self) -> SettingsMesh {
        if let Some(settings) = self.mesh.clone() {
            return settings.bind().to_struct();
        }
        SettingsMesh::default()
    }

    pub fn get_internal_collision_settings(&self) -> SettingsCollision {
        if let Some(settings) = self.collision.clone() {
            return settings.bind().to_struct();
        }
        SettingsCollision::default()
    }
}
