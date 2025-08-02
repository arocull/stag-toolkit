use crate::mesh::trimesh::TriangleMesh;
use crate::physics::body_state::BodyState;
use crate::physics::identity::Identity;
use glam::Vec3;
use std::sync::Arc;

pub struct PhysicsBody {
    /// Identity for the physics server.
    pub id: Identity,

    /// Collision primitives for the body.
    pub collision: Vec<Arc<TriangleMesh>>,

    /// This body exists in these layers, and other objects colliding with this layer can collide with this body.
    pub layers_existing: u32,
    /// This body will collide with bodies that exist in these layers.
    /// If both this body and the colliding body collide with this layer, there is conservation of momentum.
    pub layers_colliding: u32,

    /// Mass of the body, in kilograms.
    pub mass: f32,

    /// TODO: Computed center-of-mass of the physics body.
    pub center_of_mass: Vec3,
    /// TODO: Computed moment-of-inertia of the physics body.
    pub moment_of_inertia: Vec3,
    /// TODO: Computed inverse inertia of the physics body.
    pub inverse_inertia: Vec3,

    pub state: BodyState,
}

impl PhysicsBody {
    pub fn new(
        collision: Vec<Arc<TriangleMesh>>,
        mass: f32,
        layers_existing: u32,
        layers_colliding: u32,
    ) -> Self {
        Self {
            id: 0,
            collision,
            layers_existing,
            layers_colliding,
            mass,
            center_of_mass: Vec3::ZERO,
            moment_of_inertia: Vec3::ZERO,
            inverse_inertia: Vec3::splat(1.0),
            state: BodyState::default(),
        }
    }
}

impl Default for PhysicsBody {
    fn default() -> Self {
        Self::new(vec![], 1.0, u32::MAX, u32::MAX)
    }
}
