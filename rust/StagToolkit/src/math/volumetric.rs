use glam::Vec3;
use noise::{NoiseFn, Perlin, Seedable};

/// Perlin noise implementation that generates a 3D output value
pub struct PerlinField {
    x: Perlin,
    y: Perlin,
    z: Perlin,
    /// Frequency on each noise axis
    pub frequency: [f64; 4],
}

impl PerlinField {
    /// Creates a new 3D perlin noise field
    pub fn new(seed_x: u32, seed_y: u32, seed_z: u32, frequency: f64) -> Self {
        Self {
            x: Perlin::new(seed_x),
            y: Perlin::new(seed_y),
            z: Perlin::new(seed_z),
            frequency: [frequency, frequency, frequency, frequency],
        }
    }

    /// Returns the X, Y, and Z seeds of the respective noise generators
    pub fn get_seed(&self) -> (u32, u32, u32) {
        (self.x.seed(), self.y.seed(), self.z.seed())
    }
    /// Sets the X, Y, and Z seeds of the respective noise generator
    pub fn set_seed(&mut self, seed_x: u32, seed_y: u32, seed_z: u32) {
        self.x.set_seed(seed_x);
        self.y.set_seed(seed_y);
        self.z.set_seed(seed_z);
    }

    /// Samples the noise field and returns a 3D vector, each value between -1 and 1
    pub fn sample(&self, position: Vec3, w: f64) -> Vec3 {
        let sample_pt: [f64; 4] = [
            position.x as f64 * self.frequency[0],
            position.y as f64 * self.frequency[1],
            position.z as f64 * self.frequency[2],
            w * self.frequency[3],
        ];
        Vec3::new(
            self.x.get(sample_pt) as f32,
            self.y.get(sample_pt) as f32,
            self.z.get(sample_pt) as f32,
        )
    }
}

// TODO: Figure out a good method for managing volumes of data
// ...possibly need dynamic in some cases (simulations)
// ...but constant in others (island builder)
// ...maybe we make a generic specific for dynamic ones,
// ...but constant-size ones are case dependent?
// A container for storing volume data
// pub struct VolumeData<T, const VOLUME: usize> {
//     data: [T; VOLUME],
// }

// impl<const VOLUME: usize> VolumeData<f32, { VOLUME }> {
//     pub fn new(default: f32) -> Self {
//         Self {
//             data: [default; VOLUME],
//         }
//     }
// }
