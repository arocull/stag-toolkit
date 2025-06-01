use std::collections::HashMap;

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
    fn length(&self, positions: &[Vec3]) -> f32;
}

impl EdgeOperations for Edge {
    fn flip(&self) -> Self {
        [self[1], self[0]]
    }

    fn length(&self, positions: &[Vec3]) -> f32 {
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
    fn orientation(&self, positions: &[Vec3]) -> f32;
    /// Returns the calculated normal of the given face using a counter-clockwise wound triangle.
    fn normal(&self, positions: &[Vec3]) -> Vec3;
    /// Returns the face plane.
    fn plane(&self, positions: &[Vec3]) -> Vec4;
    /// Projects the given point onto the triangle.
    fn project(&self, positions: &[Vec3], point: Vec3) -> Vec3;
    /// Calculates the projected barycentric coordinates of a point `p` relative to this triangle.
    fn barycentric(&self, positions: &[Vec3], project: Vec3) -> Vec3;
    /// Returns true if the given Barycentric point is contained by the triangle.
    fn contains_barycentric(&self, barycentric_point: Vec3) -> bool;
    /// Returns true if the given point is behind the surface of the triangle.
    fn is_point_behind(&self, positions: &[Vec3], project: Vec3) -> bool;
    /// Returns true if two triangles are the same.
    fn equals(&self, other: &Triangle) -> bool;
    /// Returns a new, flipped triangle by changing vertex order.
    fn flip(&self) -> Self;
    /// Returns true if the triangle has this edge in its specified direction. False otherwise.
    fn has_edge(&self, edge: &Edge) -> bool;
    /// Returns the centerpoint of the triangle.
    fn centerpoint(&self, positions: &[Vec3]) -> Vec3;
    /// Returns a face-winded list of edges on this triangle.
    fn edges(&self) -> [Edge; 3];
}

impl TriangleOperations for Triangle {
    fn orientation(&self, positions: &[Vec3]) -> f32 {
        (positions[self[1]] - positions[self[0]])
            .cross(positions[self[2]] - positions[self[0]])
            .dot(Vec3::ONE)
    }

    fn normal(&self, positions: &[Vec3]) -> Vec3 {
        let u = positions[self[1]] - positions[self[0]];
        let v = positions[self[2]] - positions[self[0]];
        let c = u.cross(v);

        let len = c.length_squared();

        if len <= 1e-6 {
            // Make sure vector isn't zero-length
            return Vec3::Y; // Default to up if so
        }

        c / len.sqrt() // Return normalized vector
    }

    fn plane(&self, positions: &[Vec3]) -> Vec4 {
        plane(positions[self[0]], self.normal(positions))
    }

    fn project(&self, positions: &[Vec3], point: Vec3) -> Vec3 {
        // Get plane normal
        let norm = self.normal(positions);
        // Create a plane
        let pl = plane(positions[self[0]], norm);
        // Project point onto plane, using opposite of plane's normal.
        // Projection should never fail as ray is always antiparallel to the normal.
        pl.ray_intersection(point, -norm).0
    }

    fn barycentric(&self, positions: &[Vec3], project: Vec3) -> Vec3 {
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

    fn is_point_behind(&self, positions: &[Vec3], project: Vec3) -> bool {
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

    fn centerpoint(&self, positions: &[Vec3]) -> Vec3 {
        (positions[self[0]] + positions[self[1]] + positions[self[2]]) * Vec3::splat(1.0 / 3.0)
    }

    fn edges(&self) -> [Edge; 3] {
        [[self[0], self[1]], [self[1], self[2]], [self[2], self[0]]]
    }
}

// MESHES //

/// Container for triangle mesh data.
#[derive(Clone)]
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
        Self {
            triangles,
            positions,
            // Default normals to an empty vector if not included
            normals: normals.unwrap_or_default(),
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

        Self {
            triangles: tris,
            positions,
            // Default normals to an empty vector if not included
            normals: normals.unwrap_or_default(),
        }
    }

    /// Joins the given mesh with this one, in place.
    /// Does not merge points or optimize the mesh in any way.
    pub fn join(&mut self, mesh: &Self) {
        let idx_count = self.positions.len();

        // Glomp in other mesh's positions and normals
        self.positions.append(&mut mesh.positions.clone());
        self.normals.append(&mut mesh.normals.clone());

        // Prepare to add a ton of triangles
        self.triangles.reserve_exact(mesh.triangles.len());

        // Shift each triangle index based on our current number of points
        for tri in mesh.triangles.iter() {
            self.triangles
                .push([tri[0] + idx_count, tri[1] + idx_count, tri[2] + idx_count]);
        }
    }

    /// Returns the first left and right faces of an edge, if they exist.
    /// Note: very slow, prefer using `edge_map` instead if handling many edges.
    pub fn tris_for_edge(&self, edge: &Edge) -> (Option<Triangle>, Option<Triangle>) {
        let mut left: Option<Triangle> = None;
        let mut right: Option<Triangle> = None;
        let flip = edge.flip();

        for tri in self.triangles.iter() {
            if tri.has_edge(edge) {
                right = Some(*tri);
                break;
            }
        }
        for tri in self.triangles.iter() {
            if tri.has_edge(&flip) {
                left = Some(*tri);
                break;
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

    /// Returns a hash map of edges.
    /// For each edge, the left and right face index is returned, in order.
    /// The mesh is assumed to be manifold (each edge has exactly two faces).
    pub fn edge_map(&self) -> HashMap<Edge, [usize; 2]> {
        let mut edges = HashMap::<Edge, [usize; 2]>::new();

        for (idx, tri) in self.triangles.iter().enumerate() {
            for edge in tri.edges() {
                // Check for reverse-edge first
                let flip = edge.flip();

                // If we found a key already existing for our reverse edge,
                // fill in our face index as the right face
                if let Some(faces) = edges.get_mut(&flip) {
                    faces[1] = idx;
                } else {
                    // Otherwise, insert a key with this face for our edge
                    edges.insert(edge, [idx, 0]);
                }
            }
        }

        edges
    }

    /// Calculates the angle between two faces.
    pub fn face_angle(&self, a: &Triangle, b: &Triangle) -> f32 {
        a.normal(&self.positions)
            .angle_between(b.normal(&self.positions))
    }

    /// Removes an edge from the mesh by merging both vertices into a centerpoint.
    /// Does not remove degenerate geometry.
    pub fn edge_collapse(&mut self, edge: &Edge) {
        // Create a new vertex at the center of the edge
        let center = (self.positions[edge[0]] + self.positions[edge[1]]) * 0.5;

        // Append vertex to end of positions list
        let new_idx = self.positions.len();
        self.positions.push(center);

        // Swap out old vertex indices for new one
        self.swap_indices(vec![(edge[0], new_idx), (edge[1], new_idx)]);
    }

    /// Decimates the mesh by removing all immediate edges with an angle less than the given threshold.
    /// When the amount of triangles removed per decimation falls under the `minimum_dropout` threshold,
    /// the algorithm stops decimating triangles.
    pub fn decimate_planar(&mut self, threshold: f32, iterations: i32, minimum_dropout: u32) {
        // Do nothing if invalid.
        if iterations <= 0 {
            return;
        }

        for _ in 0..iterations {
            // Get a list of all edges in the trimesh
            let edges = self.edge_map();

            // Collapse all edges below the threshold
            let mut count = 0;
            for (edge, [left_idx, right_idx]) in edges.iter() {
                if self.face_angle(&self.triangles[*left_idx], &self.triangles[*right_idx])
                    < threshold
                {
                    self.edge_collapse(edge);
                    count += 1;
                }
            }

            // Clean up mesh after decimation
            self.remove_degenerate();

            // End decimation if nothing changed
            if count <= minimum_dropout {
                break;
            }
        }

        self.remove_unused();
    }

    /// Merges all vertices within the given threshold distance of each other, merging later vertices into earlier ones.
    /// This operation occurs in-place.
    /// Does not remove degenerate triangles.
    pub fn merge_by_distance(&mut self, threshold: f32) {
        let thresh_squared = threshold * threshold;

        // Array of new, merged vertices
        let mut new_verts = self.positions.clone();
        // List of vertex indices: (replace, new)
        let mut replace: Vec<(usize, usize)> = vec![];

        // Start from back of array
        for (i, vert) in self.positions.iter().enumerate().rev() {
            // ...read forward until we hit our current index
            for j in 0..i {
                if vert.distance_squared(new_verts[j]) <= thresh_squared {
                    // Remove vertices at the back of the new list
                    new_verts.remove(i);
                    // ...and modify the vertices at the front to be the midpoint
                    new_verts[j] = (vert + new_verts[j]) * 0.5;

                    // ...and note what vertices to replace
                    replace.push((i, j));

                    break;
                }
            }
        }

        // Finally, update triangle indices
        self.swap_indices(replace);
    }

    /// Iterates over all triangles, replacing each vertex index value using the given tuple: (old, new).
    /// Does not remove degenerate triangles.
    pub fn swap_indices(&mut self, replace: Vec<(usize, usize)>) {
        if replace.is_empty() {
            return;
        }

        // Iterate over every swap item
        for idx_swap in replace.iter() {
            for tri in self.triangles.iter_mut() {
                // Update the triangle indices
                for idx in tri.iter_mut() {
                    if idx_swap.0 == *idx {
                        *idx = idx_swap.1;
                    }
                }
            }
        }
    }

    /// Removes degenerate triangles from the mesh.
    pub fn remove_degenerate(&mut self) {
        // Ensure no vertex indices on the triangle match
        self.triangles
            .retain(|tri| !(tri[0] == tri[1] || tri[0] == tri[2] || tri[1] == tri[2]));
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
        let mut remapped: Vec<usize> = vec![0; used.len()];
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

    /// Performs all existing optimization steps on the triangle mesh.
    pub fn optimize(&mut self, merge_distance: f32) {
        self.merge_by_distance(merge_distance);
        self.remove_degenerate();
        self.remove_unused();
    }
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
    use super::TriangleMesh;
    use crate::mesh::trimesh::{Triangle, TriangleOperations};
    use glam::{vec3, Vec3};

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
            // Degenerate/non-manifold geometry simply returns an up vector
            TestFaceNormal {
                vertices: vec![Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
                normal: Vec3::Y,
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

        for (idx, case) in test_cases.iter().enumerate() {
            let tri: Triangle = [0, 1, 2];
            let was_behind = tri.is_point_behind(&case.tri, case.pt);

            assert_eq!(
                was_behind,
                case.behind,
                "case {0}: triangle normal {1}",
                idx,
                tri.normal(&case.tri)
            );
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

    #[test]
    fn test_planar_decimation() {
        let positions: Vec<Vec3> = vec![
            vec3(1.0, 0.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, -1.0),
        ];
        let triangles: Vec<Triangle> = vec![[0, 1, 2], [0, 3, 1]];
        let mut mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None);

        assert_eq!(
            1.0,
            triangles[0].orientation(&positions).signum(),
            "first triangle should be facing up"
        );
        assert_eq!(
            1.0,
            triangles[1].orientation(&positions).signum(),
            "second triangle should be facing up"
        );

        assert_eq!(
            0.0,
            mesh.face_angle(&triangles[0], &triangles[1]),
            "faces sharing same plane should have an angle of zero"
        );

        let edges = mesh.edge_map();
        assert_eq!(5, edges.len());

        mesh.decimate_planar(0.1, 10, 0);
        assert_eq!(0, mesh.triangles.len());
    }

    // TODO: edge map test using a manifold cube
}
