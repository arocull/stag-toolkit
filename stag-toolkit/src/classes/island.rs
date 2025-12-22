use crate::classes::island_settings::IslandBuilderSettings;
use crate::math::bounding_box::BoundingBox;
use crate::mesh::island::{Data, IslandBuilderSettingsTweaks, SettingsTweaks};
use crate::mesh::trimesh::TriangleMesh;
use crate::{
    classes::utils::editor_lock,
    math::types::ToVector3,
    mesh::godot::{GodotSurfaceArrays, GodotWhitebox},
};
use core::f32;
use glam::Vec3;
use godot::classes::{Engine, ImporterMesh, ResourceLoader};
use godot::register::ConnectHandle;
use godot::{
    classes::{
        ArrayMesh, CollisionShape3D, ConvexPolygonShape3D, MeshInstance3D, ProjectSettings,
        RigidBody3D, mesh::PrimitiveType, physics_server_3d::BodyAxis,
    },
    prelude::*,
};
use std::thread;
use std::thread::JoinHandle;

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

    /// If true, the node will watch for changes in its settings, and regenerate when needed.
    /// Only during editor.
    #[var(get,set = set_realtime_preview)]
    #[export]
    #[init(val = false)]
    realtime_preview: bool,
    /// Thread handle for real-time preview.
    #[init(val=None)]
    realtime_preview_thread: Option<JoinHandle<Option<TriangleMesh>>>,
    /// Swap buffer for real-time preview.
    #[init(val=None)]
    realtime_preview_mesh_buffer: Option<Gd<ArrayMesh>>,

    #[var(get, set=set_tweaks)]
    #[export]
    #[init(val=None)]
    tweaks: Option<Gd<IslandBuilderSettingsTweaks>>,
    #[var(get, set=set_settings)]
    #[export]
    #[init(val=None)]
    settings: Option<Gd<IslandBuilderSettings>>,

    #[init(val=None)]
    handle_tweaks: Option<ConnectHandle>,
    #[init(val=None)]
    handle_settings: Option<ConnectHandle>,

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

        // Ensure settings and fallback are up to date
        self.set_settings(self.settings.clone());
        self.apply_settings();
        self.apply_tweaks();
        self.set_realtime_preview(self.realtime_preview);
    }

    fn exit_tree(&mut self) {
        self.wait_for_preview_finish(); // wait for preview to finish
    }

    fn process(&mut self, _delta: f64) {
        if let Some(preview_thread) = &self.realtime_preview_thread
            && preview_thread.is_finished()
        {
            self.wait_for_preview_finish(); // join preview if it's done
        }
    }
}

#[godot_api]
impl IslandBuilder {
    // Setters //

    #[func]
    fn set_tweaks(&mut self, tweaks: Option<Gd<IslandBuilderSettingsTweaks>>) {
        // Disconnect existing tweaks handle if it exists
        if let Some(handle) = self.handle_tweaks.take()
            && handle.is_connected()
        {
            handle.disconnect();
        }

        self.tweaks = tweaks;

        let changed = self.data.set_tweaks(match &self.tweaks {
            Some(tweaks) => {
                // Connect to change events
                let builder = self.to_gd();
                self.handle_tweaks = Some(
                    tweaks
                        .signals()
                        .changed()
                        .builder()
                        .connect_other_mut(&builder, Self::apply_tweaks),
                );

                tweaks.bind().to_struct()
            }
            _ => SettingsTweaks::default(),
        });
        if changed {
            self.update_preview();
        }
    }

    #[func]
    fn set_settings(&mut self, settings: Option<Gd<IslandBuilderSettings>>) {
        // Disconnect existing settings handle if it exists
        if let Some(handle) = self.handle_settings.take()
            && handle.is_connected()
        {
            handle.disconnect();
        }

        self.settings = settings;

        // Pick best valid settings Resource (provided, project default, default)
        self.settings_internal = {
            match &self.settings {
                Some(settings) => settings.clone(),
                _ => {
                    let project_settings = ProjectSettings::singleton();
                    let defaults_path = project_settings
                        .get_setting_ex("addons/stag_toolkit/island_builder/default_settings")
                        .default_value(&"".to_variant())
                        .done();
                    let defaults_path: GString = defaults_path.to();

                    let new_settings: Gd<IslandBuilderSettings>;

                    // Attempt to load default settings if necessary
                    let mut resource_loader = ResourceLoader::singleton();
                    if !defaults_path.is_empty() && resource_loader.exists(&defaults_path) {
                        // Load the settings from the path
                        new_settings = try_load::<IslandBuilderSettings>(&defaults_path).unwrap_or_else(|bad_settings| {
                            godot_warn!(
                                "IslandBuilder failed to load default IslandBuilderSettings resource from project settings (addons/stag_toolkit/island_builder/default_settings): {0}",
                                bad_settings
                            );
                            IslandBuilderSettings::new_gd()
                        })
                    } else {
                        godot_warn!("No default IslandBuilder settings found for this project!");
                        new_settings = IslandBuilderSettings::new_gd();
                    }

                    new_settings
                }
            }
        };

        // Listen for future events
        let builder = self.to_gd();
        self.handle_settings = Some(
            self.settings_internal
                .signals()
                .changed()
                .builder()
                .connect_other_mut(&builder, Self::apply_settings),
        );
    }

    #[func]
    fn set_realtime_preview(&mut self, realtime_preview: bool) {
        self.realtime_preview = realtime_preview;
        self.base_mut()
            .set_process(realtime_preview && Engine::singleton().is_editor_hint());

        // Wait for any existing preview to finish before moving on
        self.wait_for_preview_finish();

        if realtime_preview {
            self.update_preview();
        }
    }

    fn wait_for_preview_finish(&mut self) {
        if let Some(handle) = self.realtime_preview_thread.take() {
            let data = handle.join().expect("realtime preview thread panicked");
            if let Some(trimesh) = data {
                // Fetch previously stored buffer and clear it for use, or create a new one
                let buffer_mesh: Gd<ArrayMesh> = match self.realtime_preview_mesh_buffer.take() {
                    Some(mut mesh) => {
                        mesh.clear_surfaces();
                        mesh
                    }
                    None => ArrayMesh::new_gd(),
                };

                // Store current IslandBuilder mesh as a new buffer if it exists
                let mesh_node = self.target_mesh();
                if let Some(base_mesh) = mesh_node.get_mesh() {
                    self.realtime_preview_mesh_buffer = match base_mesh.try_cast::<ArrayMesh>() {
                        Ok(array_mesh) => {
                            let mut result: Option<Gd<ArrayMesh>> = None;
                            if array_mesh != buffer_mesh {
                                // Make sure swap buffer isn't same as original buffer
                                result = Some(array_mesh);
                            }
                            result
                        }
                        Err(_) => None,
                    }
                }

                self.apply_preview_mesh(mesh_node, buffer_mesh, &trimesh);
            }
        }
    }

    /// Copies the tweak settings into the builder data.
    #[func]
    fn apply_tweaks(&mut self) {
        if self.data.set_tweaks(match &self.tweaks {
            Some(tweaks) => tweaks.bind().to_struct(),
            _ => SettingsTweaks::default(),
        }) {
            self.update_preview();
        }
    }

    /// Applies Godot settings to corresponding whitebox and mesh data.
    #[func]
    fn apply_settings(&mut self) {
        let settings = self.settings_internal.bind();
        let mut changed = self
            .data
            .set_voxel_settings(settings.get_internal_voxel_settings());
        changed = self
            .data
            .set_mesh_settings(settings.get_internal_mesh_settings())
            || changed;
        changed = self
            .data
            .set_collision_settings(settings.get_internal_collision_settings())
            || changed;
        drop(settings);

        if changed {
            self.base_mut().update_gizmos(); // Force redraw of IslandBuilder gizmo
            self.update_preview();
        }
    }

    /// Returns the currently used [IslandBuilderSettings] resource.
    #[func]
    fn fetch_settings(&self) -> Gd<IslandBuilderSettings> {
        self.settings_internal.clone()
    }

    // Signals //

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

    // Build Steps //

    /// Clears the build cache. Frees up system memory,
    /// but the island must be re-computed from scratch for further bakes.
    #[func]
    pub fn clear_cache(&mut self) {
        self.data.dirty_voxels();
    }

    /// Reads and stores children CSG shapes as whitebox geometry for processing.
    /// Supports Union, Intersection and Subtraction.
    ///
    /// Supported shapes include: [CSGBox3D], [CSGSphere3D], [CSGCylinder3D], and [CSGTorus3D].
    #[func]
    pub fn serialize(&mut self) {
        let mut whitebox = GodotWhitebox::new();
        whitebox.serialize_from(self.base().to_godot());
        let changed = self.data.set_shapes(whitebox.get_shapes().clone());

        if changed {
            self.base_mut().update_gizmos(); // Force redraw of IslandBuilder gizmo
        }
    }

    /// Performs a real-time preview update of the IslandBuilder.
    #[func]
    pub fn update_preview(&mut self) {
        // Ensure we're running in the editor.
        if !self.realtime_preview || !Engine::singleton().is_editor_hint() {
            return;
        }

        if let Some(preview_thread) = &self.realtime_preview_thread {
            if preview_thread.is_finished() {
                self.wait_for_preview_finish(); // collect task resources if necessary
            } else {
                // TODO: dirty preview so we update it again if not changing
                return; // don't spawn another thread
            }
        }

        self.serialize();

        let mut preview_data = self.data.clone_for_preview();
        let handle = thread::spawn(move || {
            preview_data.bake_bounding_box();
            preview_data.bake_voxels();
            preview_data.bake_preview();
            preview_data.take_mesh_preview()
        });

        self.realtime_preview_thread = Some(handle);
    }

    /// Returns an unoptimized triangle mesh for previewing with no extra information baked-in.
    /// Bakes underlying voxel and mesh data if necessary.
    /// Returns an empty mesh if there is no data to bake.
    #[func]
    pub fn generate_preview_mesh(&mut self, recycle_mesh: Option<Gd<ArrayMesh>>) -> Gd<ArrayMesh> {
        self.data.bake_voxels();
        self.data.bake_preview();

        let mesh: Gd<ArrayMesh> = match recycle_mesh {
            Some(recycle) => {
                // recycle.clear_surfaces(); // done beforehand
                recycle
            }
            _ => ArrayMesh::new_gd(),
        };

        if let Some(trimesh) = self.data.get_mesh_preview() {
            self.trimesh_to_preview(trimesh, mesh.clone());
        }

        mesh
    }

    fn apply_preview_mesh(
        &mut self,
        mut mesh_node: Gd<MeshInstance3D>,
        array_mesh: Gd<ArrayMesh>,
        trimesh: &TriangleMesh,
    ) {
        self.trimesh_to_preview(trimesh, array_mesh.clone());
        mesh_node.set_mesh(&array_mesh)
    }

    fn trimesh_to_preview(&self, trimesh: &TriangleMesh, mut array_mesh: Gd<ArrayMesh>) {
        let surface_arrays = GodotSurfaceArrays::from_trimesh(trimesh);
        array_mesh.add_surface_from_arrays(
            PrimitiveType::TRIANGLES,
            surface_arrays.get_surface_arrays(),
        );
        array_mesh.surface_set_name(0, "island");
        // Add a material, if valid
        if let Some(material) = &self.settings_internal.bind().get_material_preview() {
            array_mesh.surface_set_material(0, material);
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
        let aabb = match self.data.get_mesh_baked() {
            Some(mesh) => mesh.bounding_box(),
            None => BoundingBox::default(),
        };

        let mut props = NavIslandProperties::new_gd();

        let mut props_mut = props.bind_mut();
        props_mut.aabb = aabb.to_aabb();
        props_mut.radius = (aabb.size() * Vec3::new(1.0, 0.0, 1.0)).length() * 0.5;
        props_mut.center = aabb.center().to_vector3();
        drop(props_mut);

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
        let debug_color: Color = self.settings_internal.bind().get_collision_color();

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

    /// Fetches the output [Node] for this IslandBuilder.
    /// If no output is specified, uses this node instead.
    #[func]
    fn target(&mut self) -> Gd<Node> {
        let target = self.base().get_node_or_null(&self.output_to);
        match target {
            Some(node) => node,
            None => self.base_mut().clone().upcast::<Node>(),
        }
    }

    /// Fetches the output [MeshInstance3D] for this IslandBuilder.
    /// Creates one if none was found.
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

        mesh.set_layer_mask(self.settings_internal.bind().get_render_layers());

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

    /// Performs all IslandBuilder baking steps in order, and applies the results.
    /// Forcibly ends any real-time previews.
    ///
    /// This modifies the scene tree where necessary (IslandBuilder's children and the target node),
    /// and must be run on the main thread (or on a thread that owns the given node tree).
    /// The IslandBuilder automatically parallelizes what baking steps it can.
    #[func]
    fn build(&mut self) {
        // Perform initial data setup
        self.set_realtime_preview(false);
        self.apply_settings();
        self.serialize();

        // Generate result data
        let mesh = self.generate_baked_mesh();
        self.apply_mesh(mesh);

        let volume = self.get_volume();

        let hulls = self.generate_collision_hulls();
        self.apply_collision_hulls(hulls, volume);

        let navigation_properties = self.generate_navigation_properties();
        self.apply_navigation_properties(navigation_properties);

        // If our target node exists, then hide the builder
        let target = self.base().get_node_or_null(&self.output_to);
        if target.is_some() {
            self.base_mut().set_visible(false);
        }
    }

    /// Returns a list of all IslandBuilder nodes within the `"StagToolkit_IslandBuilder"` group in the given SceneTree.
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

    /// Destroys bakes and cache data on all provided IslandBuilder nodes.
    /// Must be run on main thread.
    #[func]
    fn all_destroy_bakes(builders: Array<Gd<Self>>) {
        for mut builder in builders.iter_shared() {
            builder.bind_mut().destroy_bakes();
        }
    }

    /// Serializes, precomputes and bakes on all provided IslandBuilder nodes.
    /// The IslandBuilder will destroy bakes beforehand.
    /// Cache data is removed after each bake in order to free up memory.
    ///
    /// Must be run on main thread.
    /// As the IslandBuilder baking processes are already parallelized where able,
    /// this function is single-threaded from the Godot-side, and blocks until completion.
    ///
    /// @experimental: This function may change in the future.
    #[func]
    fn all_bake(builders: Array<Gd<Self>>) {
        // Build everything, clearing the cache afterward
        for builder in builders.iter_shared() {
            let mut builder = builder.clone();
            builder.bind_mut().destroy_bakes();
            builder.bind_mut().build();
            builder.bind_mut().clear_cache();
        }
    }
}
