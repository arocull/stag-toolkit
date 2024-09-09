use crate::math::types::*;

/// A mesh triangle of vertex indices. In counter-clockwise face winding order.
pub type Triangle = [usize; 3];
/// A mesh edge of vertex indices. In counter-clockwise winding order.
pub type Edge = [usize; 2];

/// Container for triangle mesh data.
pub struct TriangleMesh {
    /// Primary mesh buffer, listing the index of corresponding vertex positions and normals, in counter-clockwise face winding.
    pub indices: Vec<usize>,
    /// Individual vertices of the mesh.
    pub positions: Vec<Vec3>,
    /// Normals of the mesh, assigned to vertices of corresponding index.
    pub normals: Vec<Vec3>,
}
impl TriangleMesh {
    /// Creates a new TriangleMesh from the given mesh data.
    pub fn new(indices: Vec<usize>, positions: Vec<Vec3>, normals: Vec<Vec3>) -> Self {
        Self {
            indices,
            positions,
            normals,
        }
    }

    /// Returns the calculated normal of the given face using a counter-clockwise wound triangle.
    pub fn calculate_face_normal(&self, tri: Triangle) -> Vec3 {
        let u = self.positions[tri[1]] - self.positions[tri[0]];
        let v = self.positions[tri[2]] - self.positions[tri[0]];
        -u.cross(v).normalize()
    }

    /// Returns the calculated length of an edge.
    pub fn calculate_edge_length(&self, edge: Edge) -> f32 {
        self.positions[edge[0]].distance(self.positions[edge[1]])
    }

    /// Returns the left and right faces of an edge.
    // pub fn fetch_edge_faces() -> (Triangle, Triangle) {

    // }

    /// Returns an iterator for iterating over this mesh's triangles.
    pub fn walk_triangles(&self) -> WalkTriangles {
        WalkTriangles {
            mesh: self.indices.clone(),
            curr: 0,
        }
    }

    /// Returns the number of vertices in the mesh
    pub fn count_vertices(&self) -> usize {
        self.positions.len()
    }
}

/// Describes a list of triangles. Can be iterated.
pub struct WalkTriangles {
    mesh: Vec<usize>,
    curr: usize,
}
impl Iterator for WalkTriangles {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr < self.mesh.len() {
            let current: Triangle = [
                self.mesh[self.curr],
                self.mesh[self.curr + 1],
                self.mesh[self.curr + 2],
            ];
            self.curr += 3;
            return Some(current);
        }
        None
    }
}

// UNIT TESTS //
#[cfg(test)]
mod tests {

    #[test]
    fn calculate_normal() {}
}
