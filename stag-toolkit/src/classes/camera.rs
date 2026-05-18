use glam::{EulerRot, Mat4, Quat, Vec3};
use godot::builtin::{Aabb, Array, Transform3D};
use godot::classes::Engine;
use godot::prelude::*;

use crate::math::bounding_box::BoundingBox;
use crate::math::types::ToVector3;
use crate::math::types::gdmath::{ToQuaternion, ToTransform3D};
use stag_toolkit_codegen::camera_process_toggles;

/// A node that slowly rotates to face the center of a target group.
///
/// This node must be a child of another [Node3D] in order to track properly.
/// The parent [Node3D]'s up-vector is used to determine the horizon.
#[camera_process_toggles]
#[derive(GodotClass)]
#[class(init,base=Node3D,tool,rename=StagCameraRotator)]
pub struct Rotator {
    // /// If enabled, processes on the physics tick instead.
    // #[export]
    // #[init(val = false)]
    // process_in_physics: bool,
    #[export_group(name = "Interpolation")]
    /// How quickly to rotate the camera toward the target, inversely proportional to the interpolation time.
    /// For example, when 60, it takes `1/60 = 0.1667` seconds to reach the target (1 frame at 60 FPS).
    #[export]
    #[init(val = 10.0f32)]
    pub rate: f32,

    /// If true, completely removes the roll component of the quaternion after interpolating,
    /// so it is always level with the horizon (assuming your up vector is Y+).
    #[export]
    #[init(val = true)]
    pub never_roll: bool,

    #[export_group(name = "Tracking", prefix = "tracking_")]
    /// A list of [Node3D] targets to track automatically.
    /// The node will follow the center of an [Aabb] containing all the nodes.
    ///
    /// The target nodes should be within the same [SceneTree] as this node.
    /// If empty, attempts to reset the local orientation back to the parent forward vector.
    /// Null targets are ignored.
    #[export]
    #[init(val=Array::<Option<Gd<Node3D>>>::new())]
    pub tracking_targets: Array<Option<Gd<Node3D>>>,

    /// If true, disables tracking automatically using `tracking_targets`,
    /// and instead follows the `tracking_override_position`.
    #[export]
    #[init(val = false)]
    pub tracking_override: bool,

    /// Position, in the parent [Node3D] local-space,
    /// to follow when `tracking_override` is true.
    #[export]
    #[init(val = Vector3::FORWARD)]
    pub tracking_override_position: Vector3,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Rotator {
    fn ready(&mut self) {
        self.update_tracking();
    }

    fn process(&mut self, delta: f32) {
        // First get target postiion
        let target_position: Vec3 = if self.tracking_override {
            self.tracking_override_position.to_vector3()
        } else {
            self.calculate_target_position().to_vector3()
        };

        // Normalize it into a direction
        let target_vector: Vec3 = if let Some(normalized) = target_position.try_normalize() {
            normalized
        } else {
            Vec3::NEG_Z
        };

        // Get current and goal orientations
        let current: Quat = self.base().get_quaternion().to_quaternion().normalize();
        // Not sure why this needs to be inversed, but it does!
        let mut goal: Quat = Quat::look_to_rh(target_vector, Vec3::Y).inverse();

        // TODO: fancy interpolation code here?
        goal = current
            .slerp(goal, (self.rate * delta).clamp(0.0f32, 1.0f32))
            .normalize();

        // Forcibly remove roll component to prevent "leaning"
        if self.never_roll {
            let (yaw, pitch, _roll) = goal.to_euler(EulerRot::YXZ);
            goal = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0f32);
        }

        self.base_mut().set_quaternion(goal.to_quaternion());
    }
}

#[godot_api]
impl Rotator {
    #[func]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.update_tracking();
    }
    #[func]
    pub fn set_editor_preview(&mut self, editor_preview: bool) {
        self.editor_preview = editor_preview;
        self.update_tracking();
    }

    /// Computes the tracking position for this current frame,
    /// by creating a bounding box around all tracking targets,
    /// and returning the center of it, in the parent [Node3D] local-space.
    /// Returns [Vector3.FORWARD] if there are no targets.
    ///
    /// This can be overridden in GDScript for custom tracking.
    #[func(virtual)]
    pub fn calculate_target_position(&self) -> Vector3 {
        let aabb = get_target_bounds(self.base().get_parent_node_3d(), &self.tracking_targets);

        if let Some(bounds) = aabb {
            bounds.center().to_vector3()
        } else {
            Vector3::FORWARD
        }
    }
}

/// A node that slides along the local Z axis to fit an [Aabb] in frame.
///
/// This node must not be rotated in local space in order to function properly.
/// This node must be a child of another [Node3D] in order to track properly.
///
/// Orthographic camera projections are not supported,
/// as distance is irrelevant in those cases anyway.
#[camera_process_toggles]
#[derive(GodotClass)]
#[class(init,base=Node3D,tool,rename=StagCameraDolly)]
pub struct Dolly {
    #[export_group(name = "Tracking", prefix = "tracking_")]

    /// A list of [Node3D] targets to track automatically.
    /// The dolly will attempt to contain an [Aabb] containing all the nodes.
    ///
    /// The target nodes should be within the same [SceneTree] as this node.
    /// If empty, attempts to reset the local orientation back to the parent forward vector.
    /// Null targets are ignored.
    #[export]
    #[init(val=Array::<Option<Gd<Node3D>>>::new())]
    pub tracking_targets: Array<Option<Gd<Node3D>>>,

    /// Additional radius around the targets that should be included when computing the tracking [Aabb].
    #[export]
    #[init(val = 0.5f32)]
    pub tracking_target_radius: f32,

    /// If true, disables tracking automatically using `tracking_targets`,
    /// and instead follows the `tracking_override_bounds`.
    #[export]
    #[init(val = false)]
    pub tracking_override: bool,

    /// Axis-aligned bounding-box to try and fit in the camera.
    /// This should be in the local space of the parent [Node3D].
    #[export]
    #[init(val=Aabb::default())]
    pub tracking_override_bounds: Aabb,

    #[export_group(name = "Camera Settings")]
    /// Camera field-of-view, in radians.
    /// Used to determine the dolly's distance from target bounding box.
    /// You can add or subtract a few degrees to adjust the margin.
    #[export(range=(0.1,179.9,radians_as_degrees))]
    // #[var(hint_string = "radians_as_degrees")]
    #[init(val = 75.0f32.to_radians())]
    pub fov: f32,

    // /// Aspect ratio of the camera (long axis divided by short axis).
    // /// This determines how much the bounding box fills the frame,
    // /// by allowing it to be cut off at the edges of the short axis.
    // ///
    // /// Leave this `1.0` if you wish for the entire bounding box to stay in frame.
    // #[export(range=(0.001,1.77777777777,or_greater))]
    // #[init(val = 1.0f32)]
    // pub aspect: f32,
    #[export_group(name = "Depth")]
    /// Maximum Z-depth backward this node can move.
    #[export(range=(0.0, 25.0, or_less, or_greater, suffix = "m"))]
    #[init(val = 10.0)]
    pub depth_max: f32,

    /// Maximum Z-depth forward this node can move.
    #[export(range=(-25.0, 0.0, or_less, or_greater, suffix = "m"))]
    #[init(val = -10.0)]
    pub depth_min: f32,

    /// Maximum acceleration for the dolly.
    #[export_group(name = "Motion")]
    #[export(range=(0.01, 100.0, or_greater, suffix = "m/s^2"))]
    #[init(val = 20.0)]
    pub acceleration_max: f32,

    /// Base velocity (distance / time) to apply based on the remaining travel distance.
    #[export(range=(0.0, 100.0, or_greater))]
    #[init(val = 1.0)]
    pub ramp_linear: f32,

    /// Acceleration (distance / time^2) to apply based on the remaining travel distance.
    #[export(range=(0.0, 100.0, or_greater))]
    #[init(val = 3.0)]
    pub ramp_quadratic: f32,

    /// Jerk (distance / time^3) to apply based on the remaining travel distance.
    #[export(range=(0.0, 100.0, or_greater))]
    #[init(val = 6.0)]
    pub ramp_cubic: f32,

    /// Clamp movement to this maximum speed.
    #[export(range=(0.01, 25.0, or_greater, suffix = "m/s"))]
    #[init(val = 10.0)]
    pub speed_max: f32,

    #[init(val = 0.0)]
    depth: f32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Dolly {
    fn ready(&mut self) {
        self.update_tracking();
    }

    fn process(&mut self, delta: f32) {
        let bounds: Aabb = if self.tracking_override {
            self.tracking_override_bounds
        } else {
            self.calculate_tracking_bounds()
        };

        let bounds_center = bounds.center();

        // Find ideal distance to camera
        let goal_distance = (self.goal_depth(bounds) + (bounds.size.z * 0.5) + bounds_center.z)
            .clamp(self.depth_min, self.depth_max);
        let velocity = self.ease_velocity(goal_distance - self.depth);

        let depth = (self.depth + velocity * delta).clamp(self.depth_min, self.depth_max);
        self.depth = depth;

        self.base_mut().set_transform(Transform3D::new(
            Basis::IDENTITY,
            Vector3::new(0.0, 0.0, depth),
        ));

        // println!("GOAL DISTANCE: {goal_distance}\tDEPTH: {depth}\tVELOCITY: {velocity}");
    }
}

#[godot_api]
impl Dolly {
    #[func]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.update_tracking();
    }
    #[func]
    pub fn set_editor_preview(&mut self, editor_preview: bool) {
        self.editor_preview = editor_preview;
        self.update_tracking();
    }

    /// Returns the easing velocity to use for this frame, when given a `distance` to cover.
    /// `distance` is a signed float in meters/engine units.
    /// This can be overridden in GDScript for custom easing formulas.
    ///
    /// This method cannot be called manually inside GDScript.
    #[func(virtual)]
    fn ease_velocity(&self, distance: f32) -> f32 {
        let d = distance.abs();

        ((self.ramp_linear * d)
            + (0.5 * self.ramp_quadratic * d * d)
            + ((1.0f32 / 6.0f32) * self.ramp_cubic * d * d * d))
            .min(self.speed_max)
            * distance.signum()
    }

    /// Returns the goal distance from the given bounding box, in meters/engine units.
    /// This can be overridden in GDScript for custom camera fitting.
    ///
    /// This method cannot be called manually inside GDScript.
    #[func(virtual)]
    fn goal_depth(&self, bounds: Aabb) -> f32 {
        // https://stackoverflow.com/questions/14614252/how-to-fit-camera-to-object/14614736#14614736

        // https://stackoverflow.com/a/2866471
        (bounds.size.x.max(bounds.size.y) * 0.5) / (self.fov * 0.5).tan()
    }

    /// Computes the tracking bounds for this current frame,
    /// by creating a bounding box around all tracking targets
    /// in the parent [Node3D] local-space.
    /// Returns a zero-value [Aabb] grown by the target radius if there are no targets.
    ///
    /// This can be overridden in GDScript for custom tracking.
    #[func(virtual)]
    fn calculate_tracking_bounds(&self) -> Aabb {
        let aabb = get_target_bounds(self.base().get_parent_node_3d(), &self.tracking_targets);

        if let Some(a) = aabb {
            a.expand_margin(self.tracking_target_radius).to_aabb()
        } else {
            Aabb::default().grow(self.tracking_target_radius)
        }
    }
}

/// A node that moves to align with the center of a target group.
///
/// This node must be a child of another [Node3D] in order to track properly.
/// The parent [Node3D]'s up-vector is used to determine the horizon.
#[camera_process_toggles]
#[derive(GodotClass)]
#[class(init,base=Node3D,tool,rename=StagCameraPositioner)]
pub struct Positioner {
    #[export_group(name = "Interpolation")]
    /// How quickly to move the camera toward the target, inversely proportional to the interpolation time.
    /// For example, when 60, it takes `1/60 = 0.1667` seconds to reach the target (1 frame at 60 FPS).
    #[export]
    #[init(val = 20.0f32)]
    pub rate: f32,

    #[export_group(name = "Tracking", prefix = "tracking_")]
    /// A list of [Node3D] targets to track automatically.
    /// The node will follow the center of an [Aabb] containing all the nodes.
    ///
    /// The target nodes should be within the same [SceneTree] as this node.
    /// If empty, aligns to a zero vector.
    /// Null targets are ignored.
    #[export]
    #[init(val=Array::<Option<Gd<Node3D>>>::new())]
    pub tracking_targets: Array<Option<Gd<Node3D>>>,

    /// If true, disables tracking automatically using `tracking_targets`,
    /// and instead follows the `tracking_override_position`.
    #[export]
    #[init(val = false)]
    pub tracking_override: bool,

    /// Position, in the parent [Node3D] local-space,
    /// to follow when `tracking_override` is true.
    #[export]
    #[init(val = Vector3::ZERO)]
    pub tracking_override_position: Vector3,

    #[export_group(name = "Axis Lock", prefix = "axis_lock_")]
    /// If true, forces this axis to zero when tracking.
    #[export]
    #[init(val = false)]
    pub axis_lock_x: bool,
    /// If true, forces this axis to zero when tracking.
    #[export]
    #[init(val = false)]
    pub axis_lock_y: bool,
    /// If true, forces this axis to zero when tracking.
    #[export]
    #[init(val = false)]
    pub axis_lock_z: bool,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for Positioner {
    fn ready(&mut self) {
        self.update_tracking();
    }

    fn process(&mut self, delta: f32) {
        // First get target postiion
        let goal: Vec3 = if self.tracking_override {
            self.tracking_override_position.to_vector3()
        } else {
            self.calculate_target_position().to_vector3()
        };

        // Get current position
        let current: Vec3 = self.base().get_position().to_vector3();

        // TODO: fancier interpolation code here?
        let new_position: Vec3 = current.lerp(goal, (self.rate * delta).clamp(0.0f32, 1.0f32));
        self.base_mut().set_position(new_position.to_vector3());
    }
}

#[godot_api]
impl Positioner {
    #[func]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.update_tracking();
    }
    #[func]
    pub fn set_editor_preview(&mut self, editor_preview: bool) {
        self.editor_preview = editor_preview;
        self.update_tracking();
    }

    /// Computes the tracking position for this current frame,
    /// by creating a bounding box around all tracking targets,
    /// and returning the center of it, in the parent [Node3D] local-space,
    /// with any locked axii having their component set to zero.
    /// Returns [Vector3.ZERO] if there are no targets.
    ///
    /// This can be overridden in GDScript for custom tracking.
    #[func(virtual)]
    pub fn calculate_target_position(&self) -> Vector3 {
        let aabb = get_target_bounds(self.base().get_parent_node_3d(), &self.tracking_targets);

        if let Some(bounds) = aabb {
            (bounds.center()
                * Vec3::new(
                    (!self.axis_lock_x).into(),
                    (!self.axis_lock_y).into(),
                    (!self.axis_lock_z).into(),
                ))
            .to_vector3()
        } else {
            Vector3::ZERO
        }
    }
}

/// Returns a bounding box for the given list of targets and parent node.
fn get_target_bounds(
    parent: Option<Gd<Node3D>>,
    targets: &Array<Option<Gd<Node3D>>>,
) -> Option<BoundingBox> {
    let parent_transform: Mat4 = if let Some(parent) = parent {
        parent.get_global_transform().to_transform3d().inverse()
    } else {
        Mat4::IDENTITY
    };

    let mut aabb: Option<BoundingBox> = None;
    for target_option in targets.iter_shared() {
        // target.global_transform();
        if let Some(target) = target_option {
            let pos: Vec3 =
                parent_transform.transform_point3(target.get_global_position().to_vector3());

            if let Some(bounds) = aabb.take() {
                aabb = Some(bounds.enclose(pos));
            } else {
                aabb = Some(BoundingBox::new(pos, pos));
            }
        }
    }

    aabb
}
