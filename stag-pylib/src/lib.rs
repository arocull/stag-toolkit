use pyo3::prelude::*;

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod _core {
    use std::fs::File;

    use glam::Vec3;
    use pyo3::prelude::*;
    use stag_toolkit::mesh::trimesh::{Triangle, TriangleMesh};

    /// Converts a flat list of points to vectors.
    fn floats_to_points(points: &[f32]) -> Vec<Vec3> {
        assert!(!points.is_empty(), "points buffer must not be empty");
        let mut res = Vec::<Vec3>::with_capacity(points.len() / 3);
        for i in 0..(points.len() / 3) {
            res[i] = Vec3::new(points[i * 3], points[i * 3 + 1], points[i * 3 + 2]);
        }
        res
    }

    /// Converts a flat list of indices to triangles.
    fn indices_to_triangles(indices: &[usize]) -> Vec<Triangle> {
        assert!(!indices.is_empty(), "indices buffer must not be empty");
        let mut res = Vec::<Triangle>::with_capacity(indices.len() / 3);
        for i in 0..(indices.len() / 3) {
            res[i] = [indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]];
        }
        res
    }

    /// A triangle mesh.
    #[pyclass]
    pub struct Mesh {
        trimesh: TriangleMesh,
    }

    #[pymethods]
    impl Mesh {
        #[new]
        /// Creates a new mesh with the given data.
        pub fn new(indices: Vec<usize>, vertices: Vec<f32>) -> Self {
            Self {
                trimesh: TriangleMesh::new(
                    indices_to_triangles(&indices),
                    floats_to_points(&vertices),
                    None,
                    None,
                ),
            }
        }

        /// Exports the mesh to the given directory.
        pub fn export(&self, filepath: String) {
            let mut file = File::create(filepath).expect("file should be created/opened");
            self.trimesh
                .export_obj(&mut file)
                .expect("mesh should export");
        }
    }
}
