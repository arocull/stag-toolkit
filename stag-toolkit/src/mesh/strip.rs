use glam::{Vec2, Vec3};

/// A mesh optimized for [triangle strips](https://en.wikipedia.org/wiki/Triangle_strip).
#[derive(Clone)]
pub struct TriangleStripMesh {
    pub buffer_position: usize,
    pub index: Vec<usize>,
    pub position: Vec<Vec3>,
    pub normal: Vec<Vec3>,
    pub uv1: Vec<Vec2>,
    // uv2: Vec<Vec2>,
}

impl TriangleStripMesh {
    /// Creates a new triangle strip mesh with buffers pre-allocated to the given size.
    pub fn new(tricount: usize) -> Self {
        let buffer_size = tricount + 2;
        Self {
            buffer_position: 0,
            index: Vec::<usize>::with_capacity(buffer_size),
            position: Vec::<Vec3>::with_capacity(buffer_size),
            normal: Vec::<Vec3>::with_capacity(buffer_size),
            uv1: Vec::<Vec2>::with_capacity(buffer_size),
        }
    }

    /// Push the given data for a line.
    pub fn push(&mut self, position: Vec3, normal: Vec3, uv: Vec2) {
        self.index.push(self.buffer_position);
        self.position.push(position);
        self.normal.push(normal);
        self.uv1.push(uv);
        self.buffer_position += 1;
    }
}
