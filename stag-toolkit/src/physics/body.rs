use crate::mesh::trimesh::TriangleMesh;
use crate::physics::identity::Identity;

pub struct PhysicsBody {
    pub id: Identity,
    pub collision: Vec<TriangleMesh>,
}

impl PhysicsBody {
    pub fn new(collision: Vec<TriangleMesh>) -> Self {
        Self { id: 0, collision }
    }
}
