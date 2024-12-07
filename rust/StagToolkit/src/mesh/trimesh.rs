use crate::math::{
    projection::{plane, Plane},
    types::*,
};

/// A mesh triangle of vertex indices. In counter-clockwise face winding order.
pub type Triangle = [usize; 3];
/// A mesh edge of vertex indices. In counter-clockwise winding order.
pub type Edge = [usize; 2];

/// Adds math utility functions for triangles
pub trait TriangleOperations {
    /// Returns a positive value if the triangle's points are oriented counter-clockwise,
    /// negative if clockwise, and zero if they are collinear.
    fn orientation(&self, positions: &Vec<Vec3>) -> f32;
    /// Returns the calculated normal of the given face using a counter-clockwise wound triangle.
    fn normal(&self, positions: &Vec<Vec3>) -> Vec3;
    /// Returns the face plane.
    fn plane(&self, positions: &Vec<Vec3>) -> Vec4;
    /// Projects the given point onto the triangle.
    fn project(&self, positions: &Vec<Vec3>, point: Vec3) -> Vec3;
    /// Calculates the projected barycentric coordinates of a point `p` relative to this triangle.
    fn barycentric(&self, positions: &Vec<Vec3>, project: Vec3) -> Vec3;
    /// Returns true if the given Barycentric point is contained by the triangle.
    fn contains_barycentric(&self, barycentric_point: Vec3) -> bool;
    /// Returns true if the given point is behind the surface of the triangle.
    fn is_point_behind(&self, positions: &Vec<Vec3>, project: Vec3) -> bool;
    /// Returns true if two triangles are the same.
    fn equals(&self, other: &Triangle) -> bool;
    /// Returns a new, flipped triangle by changing vertex order.
    fn flip(&self) -> Self;
}

impl TriangleOperations for Triangle {
    fn orientation(&self, positions: &Vec<Vec3>) -> f32 {
        (positions[self[1]] - positions[self[0]])
            .cross(positions[self[2]] - positions[self[0]])
            .dot(Vec3::ONE)
    }

    fn normal(&self, positions: &Vec<Vec3>) -> Vec3 {
        let u = positions[self[1]] - positions[self[0]];
        let v = positions[self[2]] - positions[self[0]];
        u.cross(v).normalize()
    }

    fn plane(&self, positions: &Vec<Vec3>) -> Vec4 {
        plane(positions[self[0]], self.normal(positions))
    }

    fn project(&self, positions: &Vec<Vec3>, point: Vec3) -> Vec3 {
        // Get plane normal
        let norm = self.normal(positions);
        // Create a plane
        let pl = plane(positions[self[0]], norm);
        // Project point onto plane, using opposite of plane's normal.
        // Projection should never fail as ray is always antiparallel to the normal.
        pl.ray_intersection(point, -norm).0
    }

    fn barycentric(&self, positions: &Vec<Vec3>, project: Vec3) -> Vec3 {
        let a = positions[self[0]];
        let b = positions[self[1]];
        let c = positions[self[2]];

        // Compute vectors and determinants for barycentric calculation
        let v0 = c - a;
        let v1 = b - a;
        let v2 = project - a;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        // Calculate barycentric coordinates
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;

        Vec3::new(1.0 - v - w, v, w) // Return UVW coordinates
    }

    fn contains_barycentric(&self, barycentric_point: Vec3) -> bool {
        barycentric_point.x >= 0.0
            && barycentric_point.x <= 1.0
            && barycentric_point.y >= 0.0
            && barycentric_point.y <= 1.0
            && barycentric_point.z >= 0.0
            && barycentric_point.z <= 1.0
    }

    fn is_point_behind(&self, positions: &Vec<Vec3>, project: Vec3) -> bool {
        let p = self.plane(positions);
        // If our point is above the plane or parallel to it, it is not in front of us
        let d = p.signed_distance(project);
        if d >= 0.0 {
            return false;
        }

        // If our point is behind us, confirm it's behind the triangle itself
        self.contains_barycentric(self.barycentric(positions, self.project(positions, project)))
    }

    fn equals(&self, other: &Triangle) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }

    fn flip(&self) -> Self {
        [self[1], self[0], self[2]]
    }
}

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
    /// Creates a new TriangleMesh from a list of triangles.
    pub fn from_triangles(triangles: Vec<Triangle>, positions: Vec<Vec3>) -> Self {
        let mut indices: Vec<usize> = vec![];
        indices.reserve_exact(triangles.len());
        for t in triangles.iter() {
            indices.push(t[0]);
            indices.push(t[1]);
            indices.push(t[2]);
        }

        let mut mesh = Self {
            indices,
            positions,
            normals: vec![],
        };

        mesh.optimize();

        mesh
    }

    /// Returns the calculated length of an edge.
    pub fn calculate_edge_length(&self, edge: Edge) -> f32 {
        self.positions[edge[0]].distance(self.positions[edge[1]])
    }

    /// Returns the left and right faces of an edge.
    // pub fn fetch_edge_faces() -> (Triangle, Triangle) { }

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

    /// Removes duplicate and unused vertices
    pub fn optimize(&mut self) {
        let mut index = 0;

        // Drop irrelevant positions
        // TODO: drop duplicate positions, could maybe use dedup?
        self.positions.retain(|_item| {
            index += 1;

            // Search through all indices. If we're being used somewhere, keep ourself
            for idx in self.indices.iter() {
                if *idx == index {
                    return true;
                }
            }
            false
        });
    }

    /// TODO: Bakes all normals into mesh data.
    pub fn bake_normals(&mut self) {}
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
    use crate::mesh::trimesh::{Triangle, TriangleOperations};

    use glam::Vec3;

    const MAX_DIFFERENCE: f32 = 1e-7;

    #[test]
    fn test_calculate_normal() {
        struct TestFaceNormal {
            vertices: Vec<Vec3>,
            normal: Vec3,
        }

        let test_cases: Vec<TestFaceNormal> = vec![
            TestFaceNormal {
                vertices: vec![Vec3::NEG_X, Vec3::X, Vec3::Y],
                normal: Vec3::Z,
            },
            // Flipping the vertex order flips the face
            TestFaceNormal {
                vertices: vec![Vec3::Y, Vec3::X, Vec3::NEG_X],
                normal: Vec3::NEG_Z,
            },
            // A raised face still has the same normal
            TestFaceNormal {
                vertices: vec![
                    Vec3::new(-1.0, 0.0, 1.0),
                    Vec3::new(1.0, 0.0, 1.0),
                    Vec3::new(0.0, 1.0, 1.0),
                ],
                normal: Vec3::Z,
            },
            TestFaceNormal {
                vertices: vec![Vec3::NEG_X, Vec3::X, Vec3::Z],
                normal: Vec3::NEG_Y,
            },
        ];

        let mut idx = 0;
        for case in test_cases.iter() {
            let tri: Triangle = [0, 1, 2];
            let normal = tri.normal(&case.vertices);

            let dot = case.normal.dot(normal);
            assert!(
                dot >= 1.0 - MAX_DIFFERENCE,
                "Case {0}, expected {1} but got {2}, with dot product {3}",
                idx,
                case.normal,
                normal,
                dot
            );

            assert!(
                normal.length() > 1.0 - MAX_DIFFERENCE,
                "Case {0}, {1} isn't normalized ( >=1.0 )",
                idx,
                normal
            );
            assert!(
                normal.length() < 1.0 + MAX_DIFFERENCE,
                "Case {0}, {1} isn't normalized ( <=1.0 )",
                idx,
                normal
            );

            idx += 1;
        }
    }

    #[test]
    fn test_is_point_behind() {
        struct TestFaceIsPointBehind {
            tri: Vec<Vec3>,
            pt: Vec3,
            behind: bool,
        }
        let test_cases: Vec<TestFaceIsPointBehind> = vec![
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(0.0, 1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                ],
                pt: Vec3::Z,
                behind: true,
            },
            // Face normal is flipped, point should be in front of face
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(-1.0, -1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(0.0, 1.0, -1.0),
                ],
                pt: Vec3::Z,
                behind: false,
            },
            // Tested point is behind, but out of face's bounds
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(0.0, 1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                ],
                pt: Vec3::new(10.0, 10.0, 10.0),
                behind: false,
            },
        ];

        let mut idx = 0;
        for case in test_cases.iter() {
            let tri: Triangle = [0, 1, 2];
            let was_behind = tri.is_point_behind(&case.tri, case.pt);

            assert_eq!(
                was_behind,
                case.behind,
                "case {0}: triangle normal {1}",
                idx,
                tri.normal(&case.tri)
            );

            idx += 1;
        }
    }
}
