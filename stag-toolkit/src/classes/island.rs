use crate::classes::island_settings::IslandBuilderSettings;
use crate::mesh::island::{
    Data, IslandBuilderSettingsTweaks, SettingsCollision, SettingsMesh, SettingsTweaks,
    SettingsVoxels,
};
use crate::{
    classes::utils::editor_lock,
    math::types::{ToVector3, gdmath::Vec3Godot},
    mesh::godot::{GodotSurfaceArrays, GodotWhitebox},
};
use core::f32;
use glam::Vec3;
use godot::classes::ImporterMesh;
use godot::{
    classes::{
        ArrayMesh, CollisionShape3D, ConvexPolygonShape3D, MeshInstance3D, ProjectSettings,
        RigidBody3D, WorkerThreadPool, mesh::PrimitiveType, physics_server_3d::BodyAxis,
    },
    prelude::*,
};

/// The node group IslandBuilder nodes should be stored in.
pub const GROUP_NAME: &str = "StagToolkit_IslandBuilder";

// GODOT CLASSES //

/// Navigation properties for Abyss islands.
/// These are utilized for A* pathing with Character AI.
#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct NavIslandProperties {
    #[export]
    #[init(val=Aabb::new(Vector3::ZERO, Vector3::ZERO))]
    aabb: Aabb,
    #[export]
    #[init(val=Vector3::ZERO)]
    center: Vector3,
    #[export]
    #[init(val = 5.0)]
    radius: f32,
    #[export]
    #[init(val = 1.0)]
    surface_flatness: f32,
    base: Base<Resource>,
}

/// The `IslandBuilder` is used to convert whitebox geometry into game-ready islands using procedural geometry.
/// To create a mesh, add CSGBox and CSGSphere nodes as descendants to the IslandBuilder,
/// then `serialize()`, `net()` and fetch your related data.
#[derive(GodotClass)]
#[class(init,base=Node3D,tool)]
pub struct IslandBuilder {
    #[init(val=Data::default())]
    data: Data,

    /// Node to target for storing generation output, and modifying data.
    /// If empty or target is not found, uses this node instead.
    #[export]
    #[init(val=NodePath::from("."))]
    output_to: NodePath,

    #[export]
    #[init(val=None)]
    tweaks: Option<Gd<IslandBuilderSettingsTweaks>>,
    #[export]
    #[init(val=None)]
    settings: Option<Gd<IslandBuilderSettings>>,

    settings_internal: Gd<IslandBuilderSettings>,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for IslandBuilder {
    /// Called upon ready notification.
    fn ready(&mut self) {
        // Add the IslandBuilder to a node group for indexing
        self.base_mut()
            .add_to_group_ex(GROUP_NAME)
            .persistent(true)
            .done();
    }
}

#[godot_api]
impl IslandBuilder {
    // Signals //

    /// Emitted when build data is applied. Useful for awaiting in multi-threaded contexts.
    #[signal]
    pub fn applied_build_data();

    // Getters //

    /// Computes and returns the Axis-Aligned Bounding Box with the current serialization.
    #[func]
    pub fn get_aabb(&self) -> Aabb {
        self.data.get_bounds().to_aabb()
    }

    /// Returns the pre-computed volume of the SDF. Returns 0 if not pre-computed.
    #[func]
    pub fn get_volume(&self) -> f32 {
        self.data.get_volume()
    }

    /// Returns the number of currently serialized shapes.
    #[func]
    pub fn get_shape_count(&self) -> i32 {
        self.data.get_shapes().len() as i32
    }

    // Setters //

    // Build Steps //

    /// Applies Godot settings to corresponding whitebox and mesh data.
    fn apply_settings(&mut self) {
        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();
            self.data
                .set_voxel_settings(settings.get_internal_voxel_settings());
            self.data
                .set_mesh_settings(settings.get_internal_mesh_settings());
            self.data
                .set_collision_settings(settings.get_internal_collision_settings());
        } else {
            // TODO: attempt to load default config from project settings
            self.data.set_voxel_settings(SettingsVoxels::default());
            self.data.set_mesh_settings(SettingsMesh::default());
            self.data
                .set_collision_settings(SettingsCollision::default());
        }

        if let Some(tweaks) = self.tweaks.clone() {
            self.data.set_tweaks(tweaks.bind().to_struct());
        } else {
            self.data.set_tweaks(SettingsTweaks::default());
        }
    }

    /// Reads and stores children CSG shapes as whitebox geometry for processing.
    /// Supports Union, Intersection and Subtraction.
    ///
    /// Supported shapes include: [CSGBox3D], [CSGSphere3D], [CSGCylinder3D], and [CSGTorus3D].
    #[func]
    pub fn serialize(&mut self) {
        let mut whitebox = GodotWhitebox::new();
        whitebox.serialize_from(self.base().to_godot());
        self.data.set_shapes(whitebox.get_shapes().clone());
    }

    /// Returns an unoptimized triangle mesh for previewing with no extra information baked-in.
    /// Bakes underlying voxel and mesh data if necessary.
    /// Returns an empty mesh if there is no data to bake.
    #[func]
    pub fn generate_preview_mesh(&mut self, recycle_mesh: Option<Gd<ArrayMesh>>) -> Gd<ArrayMesh> {
        self.data.bake_voxels();
        self.data.bake_preview();

        let mut mesh: Gd<ArrayMesh>;
        match recycle_mesh {
            Some(recycle) => {
                mesh = recycle;
                mesh.clear_surfaces();
            }
            _ => {
                mesh = ArrayMesh::new_gd();
            }
        }

        match self.data.get_mesh_baked() {
            Some(trimesh) => {
                let surface_arrays = GodotSurfaceArrays::from_trimesh(trimesh);
                mesh.add_surface_from_arrays(
                    PrimitiveType::TRIANGLES,
                    surface_arrays.get_surface_arrays(),
                );
                mesh.surface_set_name(0, "island");
                // Add a material, if valid
                if let Some(material) = &self.settings_internal.bind().get_material_baked() {
                    mesh.surface_set_material(0, material);
                }

                mesh
            }
            _ => mesh,
        }
    }
    /// Bakes and returns a triangle mesh with vertex colors, UVs, and LODs.
    /// Bakes underlying voxel and mesh data if necessary.
    /// Returns an empty mesh if there is no data to bake.
    #[func]
    pub fn generate_baked_mesh(&mut self) -> Gd<ArrayMesh> {
        self.data.bake_voxels();
        self.data.bake_preview();
        self.data.bake_mesh();

        match self.data.get_mesh_baked() {
            Some(trimesh) => {
                let surface_arrays = GodotSurfaceArrays::from_trimesh(trimesh);
                let mut importer = ImporterMesh::new_gd();
                importer.add_surface(
                    PrimitiveType::TRIANGLES,
                    surface_arrays.get_surface_arrays(),
                );
                importer.generate_lods(25.0, 60.0, &varray![]);
                importer.set_surface_name(0, "island");

                // If we have a material, assign it!
                let material = &self.settings_internal.bind().get_material_baked();
                if let Some(material) = material {
                    importer.set_surface_material(0, material);
                }

                // If we were able to successfully generate a mesh, return it
                if let Some(mesh) = importer.get_mesh() {
                    return mesh;
                }

                // If LOD generation fails, fall back to a plain array mesh
                godot_warn!("IslandBuilder: LOD generation failed. Returning island with no LODs.");

                let mut mesh = ArrayMesh::new_gd();
                mesh.add_surface_from_arrays(
                    PrimitiveType::TRIANGLES,
                    surface_arrays.get_surface_arrays(),
                );
                mesh.surface_set_name(0, "island");

                if let Some(material) = material {
                    mesh.surface_set_material(0, material);
                }

                mesh
            }
            _ => ArrayMesh::new_gd(),
        }
    }
    /// Computes and returns a list of collision hulls.
    /// Bakes underlying voxel and mesh data if necessary.
    /// Returns an empty array if there is no data to bake.
    #[func]
    pub fn generate_collision_hulls(&mut self) -> Array<Gd<ConvexPolygonShape3D>> {
        self.data.bake_voxels();
        self.data.bake_preview();
        self.data.bake_collision();

        let hull_pts = self.data.get_hulls();

        Array::<Gd<ConvexPolygonShape3D>>::from_iter(hull_pts.iter().map(|pts| {
            let mut shape = ConvexPolygonShape3D::new_gd();
            shape.set_points(&pts.positions.to_vector3()); // Fetch remaining positions from the hull
            shape
        }))
    }
    /// Computes and returns the navigation properties of the island.
    /// Properties will be zero'd if not pre-computed.
    #[func]
    fn generate_navigation_properties(&self) -> Gd<NavIslandProperties> {
        let mut props = NavIslandProperties::new_gd();
        let aabb = self.get_aabb();

        let size: Vec3 = aabb.size.to_vector3();
        let rad: f32 = (size * Vec3::new(1.0, 0.0, 1.0)).length() / 2.0;

        {
            let mut props_mut = props.bind_mut();
            props_mut.aabb = aabb;
            props_mut.radius = rad;
            props_mut.center =
                (aabb.center() * Vec3Godot::new(1.0, 0.0, 1.0)) + (aabb.support(Vec3Godot::UP));
        }

        props
    }

    /// Applies the given mesh to the island output.
    #[func]
    fn apply_mesh(&mut self, mesh: Gd<ArrayMesh>) {
        self.target_mesh().set_mesh(&mesh);
    }

    /// Applies the given list of collision shapes to the island output.
    /// Sets up physics properties on RigidBodies when possible.
    #[func]
    fn apply_collision_hulls(&mut self, hulls: Array<Gd<ConvexPolygonShape3D>>, volume: f32) {
        let mut target = self.target();

        // Remove all current collider children
        for child in target.get_children().iter_shared() {
            // If this is a CollisionShape3D, destroy it
            match child.try_cast::<CollisionShape3D>() {
                Ok(mut collision) => {
                    target.remove_child(&collision);
                    collision.queue_free();
                }
                Err(_as_node_again) => {}
            }
        }

        // Fetch color for debug drawing
        let settings = ProjectSettings::singleton();
        let debug_color_variant: Variant = settings
            .get_setting_ex("addons/stag_toolkit/island_builder/collision_color")
            .default_value(&Variant::from(Color::from_rgba(1.0, 0.0, 0.667, 1.0)))
            .done();
        let debug_color: Color;

        // Ensure variant is of proper type
        if let Ok(color) = debug_color_variant.try_to::<Color>() {
            debug_color = color;
        } else {
            // Otherwise, use default
            debug_color = Color::from_rgba(1.0, 0.0, 0.667, 1.0);
        }

        // Get collision hulls
        for (idx, hull) in hulls.iter_shared().enumerate() {
            let mut shape = CollisionShape3D::new_alloc();
            shape.set_shape(&hull);
            shape.set_name(&format!("collis{idx}"));
            shape.set_debug_color(debug_color); // Apply debug draw color
            editor_lock(shape.clone().upcast(), true); // Lock editing

            target.add_child(&shape); // Add shape to scene

            // Set shape owner so it is included and saved within the scene
            if let Some(tree) = target.get_tree()
                && let Some(root) = tree.get_edited_scene_root()
            {
                shape.set_owner(&root);
            }
        }

        // Apply physics properties
        if let Ok(mut rigid) = target.clone().try_cast::<RigidBody3D>() {
            rigid.set_mass(volume * self.settings_internal.bind().get_physics_density());
            rigid.set_axis_lock(BodyAxis::ANGULAR_X, true);
            rigid.set_axis_lock(BodyAxis::ANGULAR_Z, true);
            rigid.set_axis_lock(BodyAxis::LINEAR_Y, true);
        }

        // If possible, apply maximum health too
        if let Some(mut p) = target.clone().get_parent()
            && p.has_method("set_maximum_health")
        {
            p.call(
                "set_maximum_health",
                &[Variant::from(
                    volume * self.settings_internal.bind().get_physics_health_density(),
                )],
            );
        }
    }

    /// Applies the given [NavIslandProperties] to the island output or its corresponding parent, if possible.
    ///
    /// Searches specifically for an object method `set_navigation_properties(...)` with a single [NavIslandProperties] argument.
    #[func]
    fn apply_navigation_properties(&mut self, props: Gd<NavIslandProperties>) {
        let mut p = self.target();

        // Apply navigation properties if target has them available
        if p.has_method("set_navigation_properties") {
            p.callv("set_navigation_properties", &varray![Variant::from(props)]);
            return;
        }

        // Otherwise, apply navigation properties to target's parent
        if let Some(mut parent) = p.get_parent()
            && parent.has_method("set_navigation_properties")
        {
            parent.callv("set_navigation_properties", &varray![Variant::from(props)]);
        }
    }

    /// Fetches the output node for this IslandBuilder.
    /// If no output is specified, uses this node instead.
    #[func]
    fn target(&mut self) -> Gd<Node> {
        let target = self.base().get_node_or_null(&self.output_to);
        match target {
            Some(node) => node,
            None => self.base_mut().clone().upcast::<Node>(),
        }
    }

    /// Fetches the output mesh for this IslandBuilder.
    /// Creates one if none was found.
    /// If the mesh is newly created, its render layers are specified by
    /// `"addons/stag_toolkit/island_builder/render_layers"`
    /// in the Project Settings.
    #[func]
    fn target_mesh(&mut self) -> Gd<MeshInstance3D> {
        let mut target = self.target();

        // Find a mesh
        for child in target.get_children().iter_shared() {
            match child.try_cast::<MeshInstance3D>() {
                Ok(mesh) => return mesh,
                Err(_as_node) => {}
            }
        }

        // If no mesh found, create one
        let mut mesh = MeshInstance3D::new_alloc();
        // Editor lock the mesh so users don't mess with it
        editor_lock(mesh.clone().upcast(), true); // Lock editing

        // Get render layers mask from Project Settings
        let settings = ProjectSettings::singleton();
        let mask = settings
            .get_setting_ex("addons/stag_toolkit/island_builder/render_layers")
            .default_value(&Variant::from(5))
            .done();
        mesh.set_layer_mask(mask.to());

        // Add mesh to scene
        mesh.set_name("mesh_island");
        target.add_child(&mesh);

        // Ensure scene owns mesh object
        // If no scene tree found, instead use target node as owner
        if let Some(tree) = target.get_tree() {
            mesh.set_owner(&tree.get_edited_scene_root().unwrap_or(target));
        }

        mesh
    }

    /// Destroys all MeshInstance3D and CollisionShape3D nodes directly under the output node.
    /// Clears all working data: The IslandBuilder will have to be re-serialized and netted.
    /// Removes PackedScene references on the IslandBuilder's target node.
    #[func]
    fn destroy_bakes(&mut self) {
        self.data.dirty_voxels();

        let mut out = self.target();
        out.set_scene_file_path(""); // Clear scene file path

        // Iterate over all children.
        for child in out.get_children().iter_shared() {
            // match_class! {}

            // If this is a MeshInstance3D, destroy it
            match child.try_cast::<MeshInstance3D>() {
                Ok(mut mesh) => {
                    mesh.set_mesh(Gd::null_arg());
                }
                Err(as_node) => {
                    // OR, if this is a CollisionShape3D, destroy it
                    match as_node.try_cast::<CollisionShape3D>() {
                        Ok(mut collision) => {
                            out.remove_child(&collision);
                            collision.queue_free();
                        }
                        Err(_as_node_again) => {}
                    }
                }
            }
        }
    }

    /// Performs all `IslandBuilder` baking steps in order, and applies the results.
    /// If running on a thread, pass `true` for `thread` safe-calls only.
    #[func]
    fn build(&mut self, threaded: bool) {
        // Perform initial data setup
        self.apply_settings();
        if !threaded {
            self.serialize();
        }

        // Generate result data
        let mesh = self.generate_baked_mesh();
        let hulls = self.generate_collision_hulls();
        let navprops = self.generate_navigation_properties();
        let volume = self.get_volume();

        // Apply results
        if threaded {
            // Make a deferred call if necessary.
            self.base_mut().call_deferred(
                "apply_build_data",
                &[
                    mesh.to_variant(),
                    hulls.to_variant(),
                    Variant::from(volume),
                    navprops.to_variant(),
                ],
            );
        } else {
            self.apply_build_data(mesh, hulls, volume, navprops);
        }
    }

    /// Applies the provided build data to the island output.
    /// Called by `build(...)` automatically, this function is separated for multi-threading purposes only.
    #[func]
    fn apply_build_data(
        &mut self,
        mesh: Gd<ArrayMesh>,
        hulls: Array<Gd<ConvexPolygonShape3D>>,
        volume: f32,
        navprops: Gd<NavIslandProperties>,
    ) {
        self.apply_mesh(mesh);
        self.apply_collision_hulls(hulls, volume);
        self.apply_navigation_properties(navprops);

        let target = self.base().get_node_or_null(&self.output_to);
        if target.is_some() {
            self.base_mut().set_visible(false);
        }

        self.signals().applied_build_data().emit();
    }

    /// Returns a list of ALL IslandBuilder nodes within the `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    #[func]
    fn all_builders(mut tree: Gd<SceneTree>) -> Array<Gd<Self>> {
        let nodes = tree.get_nodes_in_group(GROUP_NAME);
        let mut builders: Array<Gd<Self>> = array![];

        for node in nodes.iter_shared() {
            match node.try_cast::<Self>() {
                Ok(isle) => builders.push(&isle),
                Err(_none) => {}
            }
        }

        builders
    }

    /// Destroys bakes on **ALL** IslandBuilder nodes within the `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    #[func]
    fn all_destroy_bakes(tree: Gd<SceneTree>) {
        for mut builder in Self::all_builders(tree).iter_shared() {
            builder.bind_mut().destroy_bakes();
        }
    }

    /// Serializes, precomputes and bakes on **ALL** IslandBuilder nodes within the
    /// `"StagToolkit_IslandBuilder"` group in the given SceneTree.
    /// The IslandBuilder will destroy bakes beforehand.
    ///
    /// NOTE: Currently, due to multi-threading, the results may be deferred to the end of frame.
    /// Optionally await `applied_build_data` on an island of your choice to get its ASAP.
    ///
    /// @experimental: This function may change in the future. This function utilizes multi-threading, which may be unstable.
    #[func]
    fn all_bake(tree: Gd<SceneTree>) {
        // Fetch all builder shapes in the scene tree and serialize them
        let builders = Self::all_builders(tree);

        // Ensure all Island Builders are serialized before threading
        for builder in builders.iter_shared() {
            builder.clone().bind_mut().serialize();
        }

        // Get callable to our class' static single-island bake method,
        // and bind our list of working islands to it.
        let bake_method = Callable::from_sync_fn("all_bake_single", |args: &[&Variant]| {
            let idx: i32 = args[0].to();
            let isles: Array<Gd<Self>> = args[1].to();
            // Ensure we don't go out of bounds
            if idx as usize > isles.len() {
                return Ok(Variant::from(false));
            }

            let mut isle = isles.at(idx as usize).clone();
            isle.bind_mut().build(true);
            Ok(Variant::from(true))
        })
        .bind(&[builders.to_variant()]);

        // Fetch worker pool
        let mut workerpool = WorkerThreadPool::singleton();

        // Allocate and run worker threads
        let group_id = workerpool
            .add_group_task_ex(&bake_method, builders.len() as i32)
            .high_priority(true)
            .description("StagToolkit > IslandBuilder > bake all islands in scene")
            .done();

        // Wait for groups to finish
        workerpool.wait_for_group_task_completion(group_id);
    }
}

#[cfg(test)]
mod tests {
    // fn test_binds() {

    // }
}
