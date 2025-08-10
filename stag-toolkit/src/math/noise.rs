use glam::{Vec3, Vec4};
use noise::{NoiseFn, Perlin, Seedable};

#[derive(Clone)]
pub struct Perlin1D {
    pub frequency: [f64; 4],
    pub amplitude: f64,
    perlin: Perlin,
}

/// Generates a 1D noise value from a 4D input.
impl Perlin1D {
    pub fn new(seed: u32, frequency: [f64; 4], amplitude: f64) -> Self {
        Self {
            frequency,
            amplitude,
            perlin: Perlin::new(seed),
        }
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.perlin = self.perlin.set_seed(seed);
    }

    pub fn seed(&self) -> u32 {
        self.perlin.seed()
    }

    /// Returns a value based on the sampling position,
    /// applying frequency and amplitude automatically.
    pub fn sample_precise(&self, position: [f64; 4]) -> f64 {
        self.perlin.get([
            position[0] * self.frequency[0],
            position[1] * self.frequency[1],
            position[2] * self.frequency[2],
            position[3] * self.frequency[3],
        ]) * self.amplitude
    }

    /// Returns a value based on the sampling position,
    /// applying frequency and amplitude automatically.
    pub fn sample(&self, position: Vec4) -> f64 {
        self.perlin.get([
            position.x as f64 * self.frequency[0],
            position.y as f64 * self.frequency[1],
            position.z as f64 * self.frequency[2],
            position.w as f64 * self.frequency[3],
        ]) * self.amplitude
    }
}

impl Default for Perlin1D {
    fn default() -> Self {
        Self::new(0, [1.0; 4], 1.0)
    }
}

#[derive(Clone)]
pub struct Perlin3D {
    pub frequency: [f64; 4],
    pub amplitude: [f64; 3],
    x: Perlin,
    y: Perlin,
    z: Perlin,
}

/// Generates a 3D noise value from a 4D input.
impl Perlin3D {
    pub fn new(seed: u32, frequency: [f64; 4], amplitude: [f64; 3]) -> Self {
        Self {
            frequency,
            amplitude,
            x: Perlin::new(seed),
            y: Perlin::new(seed + 1),
            z: Perlin::new(seed + 2),
        }
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.x = self.x.set_seed(seed);
        self.y = self.y.set_seed(seed);
        self.z = self.z.set_seed(seed);
    }

    pub fn seed(&self) -> u32 {
        self.x.seed()
    }

    /// Returns a value based on the sampling position,
    /// applying frequency and amplitude automatically.
    pub fn sample_precise(&self, position: [f64; 4]) -> [f64; 3] {
        let pos = [
            position[0] * self.frequency[0],
            position[1] * self.frequency[1],
            position[2] * self.frequency[2],
            position[3] * self.frequency[3],
        ];

        [
            self.x.get(pos) * self.amplitude[0],
            self.y.get(pos) * self.amplitude[1],
            self.z.get(pos) * self.amplitude[2],
        ]
    }

    /// Returns a value based on the sampling position,
    /// applying frequency and amplitude automatically.
    pub fn sample(&self, position: Vec4) -> Vec3 {
        let res = self.sample_precise([
            position.x as f64,
            position.y as f64,
            position.z as f64,
            position.z as f64,
        ]);
        Vec3::new(res[0] as f32, res[1] as f32, res[2] as f32)
    }
}

impl Default for Perlin3D {
    fn default() -> Self {
        Self::new(0, [1.0; 4], [1.0, 1.0, 1.0])
    }
}
