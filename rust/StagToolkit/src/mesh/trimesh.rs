use crate::math::{
    projection::{plane, Plane},
    types::*,
};

// EDGES //

/// A mesh edge of vertex indices. In counter-clockwise winding order.
pub type Edge = [usize; 2];

/// A set of two indices, that can be operated from any set of positions.
pub trait EdgeOperations {
    /// Returns a new, flipped edge by changing vertex order.
    fn flip(&self) -> Self;
    /// Returns the calculated length of an edge.
    fn length(&self, positions: &Vec<Vec3>) -> f32;
}

impl EdgeOperations for Edge {
    fn flip(&self) -> Self {
        [self[1], self[0]]
    }

    fn length(&self, positions: &Vec<Vec3>) -> f32 {
        positions[self[0]].distance(positions[self[1]])
    }
}

// TRIANGLES //

/// A mesh triangle of vertex indices. In counter-clockwise face winding order.
pub type Triangle = [usize; 3];

/// A set of three indices, that can be operated on from any set of positions.
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
    /// Returns true if the triangle has this edge in its specified direction. False otherwise.
    fn has_edge(&self, edge: &Edge) -> bool;
    /// Returns the centerpoint of the triangle.
    fn centerpoint(&self, positions: &Vec<Vec3>) -> Vec3;
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

    fn has_edge(&self, edge: &Edge) -> bool {
        for i in 0..3 {
            // Check if the edge exists along this point or the next wrapped one
            if edge[0] == self[i] && edge[1] == self[(i + 1) % self.len()] {
                return true;
            }
        }

        false
    }

    fn centerpoint(&self, positions: &Vec<Vec3>) -> Vec3 {
        (positions[self[0]] + positions[self[1]] + positions[self[2]]) * Vec3::splat(1.0 / 3.0)
    }
}

// MESHES //

/// Container for triangle mesh data.
pub struct TriangleMesh {
    /// Primary mesh buffer, listing the index of corresponding vertex positions and normals, in counter-clockwise face winding.
    pub triangles: Vec<Triangle>,
    // pub indices: Vec<usize>,
    /// Individual vertices of the mesh.
    pub positions: Vec<Vec3>,
    /// Normals of the mesh, assigned to vertices of corresponding index.
    pub normals: Vec<Vec3>,
}
impl TriangleMesh {
    /// Creates a new TriangleMesh from the given mesh data.
    pub fn new(triangles: Vec<Triangle>, positions: Vec<Vec3>, normals: Option<Vec<Vec3>>) -> Self {
        // Default normals to an empty vector if not included
        let norms: Vec<Vec3>;
        match normals {
            Some(normals_list) => norms = normals_list,
            None => norms = vec![],
        }

        Self {
            triangles,
            positions,
            normals: norms,
        }
    }

    /// Creates a new TriangleMesh from a list of indices.
    /// Every three indices are expected to represent a triangle, with counter-clockwise face winding.
    /// Each index has a corresponding vertex position and normal.
    /// List of normals is optional.
    pub fn from_indices(
        indices: Vec<usize>,
        positions: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
    ) -> Self {
        // Reserve triangles
        let mut tris: Vec<Triangle> = vec![];
        tris.reserve_exact(indices.len() / 3);

        // Create triangles for each index
        for i in 0..(indices.len() / 3) {
            tris.push([indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]]);
        }

        // Default normals to an empty vector if not included
        let norms: Vec<Vec3>;
        match normals {
            Some(normals_list) => norms = normals_list,
            None => norms = vec![],
        }

        Self {
            triangles: tris,
            positions,
            normals: norms,
        }
    }

    /// Returns the first left and right faces of an edge, if they exist.
    pub fn tris_for_edge(&self, edge: &Edge) -> (Option<Triangle>, Option<Triangle>) {
        let mut left: Option<Triangle> = None;
        let mut right: Option<Triangle> = None;

        for tri in self.triangles.iter() {
            if tri.has_edge(edge) {
                right = Some(*tri);

                if left.is_some() {
                    // Return if we finished
                    break;
                }
            } else if tri.has_edge(&edge.flip()) {
                left = Some(*tri);

                if right.is_some() {
                    // Return if we finished.
                    break;
                }
            }
        }

        (left, right)
    }

    /// Returns the number of vertex positions in the mesh.
    pub fn count_vertices(&self) -> usize {
        self.positions.len()
    }

    /// Unwraps the triangles in the mesh and returns them as an ordered list of indices.
    pub fn indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = vec![];
        indices.reserve_exact(self.triangles.len());

        for tri in self.triangles.iter() {
            for idx in tri {
                indices.push(*idx);
            }
        }

        indices
    }

    /// Removes all unused vertex positions in the mesh.
    pub fn remove_unused(&mut self) {
        // Keep track of all used points
        let mut used: Vec<bool> = vec![];
        used.resize(self.positions.len(), false);

        // Figure out what points are used
        for tri in self.triangles.iter() {
            for item in tri {
                used[*item] = true;
            }
        }

        // Drop points that are not associated with anything
        let mut idx: usize = 0;
        self.positions.retain(|_item| {
            let i = idx;
            idx += 1;
            used[i]
        });

        // ...do the same for normals.
        idx = 0;
        self.normals.retain(|_item| {
            let i = idx;
            idx += 1;
            used[i]
        });

        // Create an array for remapping vertex index values
        let mut remapped: Vec<usize> = vec![];
        remapped.resize(used.len(), 0);
        let mut new_idx: usize = 0; // Current available index
        for (idx, used) in used.iter().enumerate() {
            if *used {
                remapped[idx] = new_idx;
                new_idx += 1;
            }
        }

        // Adjust all triangle indices accordingly
        let mut new_tris: Vec<Triangle> = vec![];
        new_tris.reserve_exact(self.triangles.len());

        for tri in self.triangles.iter() {
            new_tris.push([remapped[tri[0]], remapped[tri[1]], remapped[tri[2]]]);
        }

        self.triangles = new_tris;
    }

    /// TODO: Bakes all normals into mesh data.
    pub fn bake_normals(&mut self) {}
}

/*
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
*/

// UNIT TESTS //
#[cfg(test)]
mod tests {
    use crate::mesh::trimesh::{Triangle, TriangleOperations};

    use glam::Vec3;

    use super::TriangleMesh;

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

    #[test]
    fn test_remove_unused() {
        let positions: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0), // 0
            Vec3::new(1.0, 0.0, 0.0), // 1
            Vec3::new(0.0, 1.0, 0.0), // 2
            Vec3::new(0.0, 1.0, 0.0), // 3
            Vec3::new(0.0, 0.0, 1.0), // 4
            Vec3::new(0.0, 0.0, 1.0), // 5
        ];
        let normals: Vec<Vec3> = positions.clone();
        let tris: Vec<Triangle> = vec![[1, 4, 3], [3, 4, 1]];
        let indices: Vec<usize> = vec![1, 4, 3, 3, 4, 1];

        let mut mesh = TriangleMesh::new(tris.clone(), positions.clone(), Some(normals.clone()));
        let mut index_mesh =
            TriangleMesh::from_indices(indices.clone(), positions.clone(), Some(normals.clone()));

        assert_eq!(
            tris, mesh.triangles,
            "created TriangleMesh has the same triangles"
        );
        assert_eq!(
            positions, mesh.positions,
            "created TriangleMesh has the same vertex positions"
        );
        assert_eq!(
            normals, mesh.normals,
            "created TriangleMesh has the same vertex normals"
        );
        assert_eq!(
            indices,
            mesh.indices(),
            "created TriangleMesh has the same indices"
        );

        assert_eq!(
            tris, index_mesh.triangles,
            "INDEXED TriangleMesh has the same triangles"
        );
        assert_eq!(
            positions, index_mesh.positions,
            "INDEXED TriangleMesh has the same vertex positions"
        );
        assert_eq!(
            normals, index_mesh.normals,
            "INDEXED TriangleMesh has the same vertex normals"
        );
        assert_eq!(
            indices,
            mesh.indices(),
            "created TriangleMesh has the same indices"
        );

        // Remove unused positions
        mesh.remove_unused();
        index_mesh.remove_unused();

        // Mesh vertex buffers shrank
        assert_eq!(3, mesh.positions.len(), "only 3 positions remain");
        assert_eq!(3, mesh.normals.len(), "only 3 normals remain");
        assert_eq!(3, index_mesh.positions.len(), "only 3 positions remain");
        assert_eq!(3, index_mesh.normals.len(), "only 3 normals remain");

        // Mesh should only contain our expected positions
        let positions_expected: Vec<Vec3> = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        assert_eq!(positions_expected, mesh.positions);
        assert_eq!(positions_expected, index_mesh.positions);

        // Indices should have been adjusted for resized buffers
        let indices_expected: Vec<usize> = vec![0, 2, 1, 1, 2, 0];
        assert_eq!(
            indices_expected,
            mesh.indices(),
            "should return expected vertex indices"
        );
        assert_eq!(
            indices_expected,
            index_mesh.indices(),
            "should return expected vertex indices"
        );
    }
}
