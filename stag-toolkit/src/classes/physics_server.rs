use crate::math::raycast::RaycastParameters;
use crate::math::types::ToVector3;
use crate::math::types::gdmath::ToTransform3D;
use crate::mesh::trimesh::{Triangle, TriangleMesh};
use crate::physics::body::PhysicsBody;
use crate::physics::body_state::BodyState;
use crate::physics::identity::Identity;
use crate::physics::raycast::PhysicsRaycastParameters;
use crate::physics::server::{PhysicsServer, PhysicsServerSettings};
use glam::Vec3;
use godot::classes::ConvexPolygonShape3D;
use godot::prelude::*;
use std::sync::Arc;

/// A StagToolkit physics runtime tied to a single node.
/// This server can be interfaced with
/// @experimental: This implementation is extremely crude.
#[derive(GodotClass)]
#[class(base=Node,tool)]
pub struct StagPhysicsServer {
    server: PhysicsServer,

    base: Base<Node>,
}

#[godot_api]
impl INode for StagPhysicsServer {
    fn init(base: Base<Node>) -> Self {
        Self {
            server: PhysicsServer::new(PhysicsServerSettings {
                simulate_bodies: false,
            }),
            base,
        }
    }
}

#[godot_api]
impl StagPhysicsServer {
    /// Registers a physics body with the physics server, and returns the registerd body ID.
    /// @experimental: Currently, collision cannot be changed after registering.
    #[func]
    fn register_body(
        &mut self,
        collision_shapes: Array<Gd<ConvexPolygonShape3D>>,
        mass: f32,
        collision_exist: u32,
        collision_mask: u32,
    ) -> u64 {
        let mut meshes: Vec<Arc<TriangleMesh>> = Vec::with_capacity(collision_shapes.len());

        // Convert convex collision shapes into meshes
        for mut shape in collision_shapes.iter_shared() {
            if let Some(debug_mesh) = shape.get_debug_mesh() {
                // Get vertices with face winding
                let vertices: Vec<Vec3> = debug_mesh.get_faces().to_vector3();

                // Create an array of triangles
                let mut tris: Vec<Triangle> = Vec::with_capacity(vertices.len() / 3);
                for i in 0..vertices.len() / 3 {
                    tris.push([i * 3, i * 3 + 1, i * 3 + 2]);
                }

                // Build and optimize a collision mesh
                let mut mesh = TriangleMesh::new(tris, vertices, None, None);
                mesh.optimize(1e-6);

                meshes.push(Arc::new(mesh));
            }
        }

        let body = PhysicsBody::new(meshes, mass, collision_exist, collision_mask);

        if let Some(id) = self.server.register_body(body) {
            return id;
        }

        godot_error!("Already registered with StagPhysicsServer node");
        0
    }

    /// Steps the physics simulation forward by `delta` seconds.
    #[func]
    fn tick(&self, delta: f32) {
        self.server.tick(delta);
    }

    /// Sets the physics state of the given physics body.
    #[func]
    fn set_body_state(
        &mut self,
        id: u64,
        global_transform: Transform3D,
        linear_velocity: Vector3,
        angular_velocity: Vector3,
    ) {
        let failed = self.server.set_body_state(
            id as Identity,
            BodyState::new(
                global_transform.to_transform3d(),
                linear_velocity.to_vector3(),
                angular_velocity.to_vector3(),
            ),
        );

        if failed {
            godot_error!("Failed to set body state for {id}");
        }
    }

    /// Performs a raycast on all bodies registered in the physics server.
    /// Returns a dictionary which contains the following data:
    /// - `point` [Vector3] collision point in global space
    /// - `normal` [Vector3] collision normal in global space
    /// - `depth` float raycast distance
    /// - `body` integer identifier for the colliding physics body
    ///
    /// If the raycast did not hit, the dictionary is empty.
    #[func]
    fn raycast(
        &self,
        origin: Vector3,
        direction: Vector3,
        max_depth: f32,
        hit_backfaces: bool,
        collision_mask: u32,
    ) -> Dictionary {
        let params = PhysicsRaycastParameters::new(
            RaycastParameters::new(
                origin.to_vector3(),
                direction.to_vector3(),
                max_depth,
                hit_backfaces,
            ),
            collision_mask,
        );
        let result = self.server.raycast(params);

        if let Some(result) = result {
            let mut dictionary = Dictionary::new();
            dictionary.set(
                "point",
                <Vec3 as ToVector3<Vector3>>::to_vector3(&result.raycast_result.point),
            );
            dictionary.set(
                "normal",
                <Vec3 as ToVector3<Vector3>>::to_vector3(&result.raycast_result.normal),
            );
            dictionary.set("depth", result.raycast_result.depth);
            dictionary.set("body", result.body_identifier);

            return dictionary;
        }

        Dictionary::new()
    }
}
