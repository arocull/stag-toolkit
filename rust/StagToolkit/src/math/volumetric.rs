use glam::{FloatExt, Mat4, Vec3};
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
        let sample_pt = self.get_sample_point(position, w);
        Vec3::new(
            self.x.get(sample_pt) as f32,
            self.y.get(sample_pt) as f32,
            self.z.get(sample_pt) as f32,
        )
    }

    /// Samples the noise field and returns a single value between -1 and 1
    pub fn sample_single(&self, position: Vec3, w: f64) -> f64 {
        let sample_pt = self.get_sample_point(position, w);
        self.x.get(sample_pt)
    }

    fn get_sample_point(&self, position: Vec3, w: f64) -> [f64; 4] {
        [
            position.x as f64 * self.frequency[0],
            position.y as f64 * self.frequency[1],
            position.z as f64 * self.frequency[2],
            w * self.frequency[3],
        ]
    }
}

// TODO: Figure out a good method for managing volumes of data
// ...possibly need dynamic in some cases (simulations)
// ...but constant in others (island builder)
// ...maybe we make a generic specific for dynamic ones,
// ...but constant-size ones are case dependent?
/// A container for storing volume data
pub struct VolumeData<T> {
    data: Vec<T>,
    dim: [u32; 3],
    strides: [u32; 3],
    size: usize,
}

impl VolumeData<f32> {
    /// Creates a new volumetric of the given size with the default value.
    pub fn new(default: f32, dim: [u32; 3]) -> Self {
        let size = (dim[0] * dim[1] * dim[2]) as usize;

        let mut dat = vec![];
        dat.resize(size, default);

        Self {
            data: dat,
            dim,
            strides: [1, dim[0], dim[0] * dim[1]],
            size,
        }
    }

    /// Sets the value at the given linear index
    pub fn set_linear(&mut self, i: usize, val: f32) {
        self.data[i] = val;
    }

    /// Returns the value at the given linear index
    pub fn get_linear(&self, i: usize) -> f32 {
        self.data[i]
    }

    /// Returns the linearized index of the given value.
    pub fn linearize(&self, x: u32, y: u32, z: u32) -> u32 {
        let x_clamp = x.clamp(0, self.dim[0] - 1);
        let y_clamp = y.clamp(0, self.dim[1] - 1);
        let z_clamp = z.clamp(0, self.dim[2] - 1);

        x_clamp + self.strides[1].wrapping_mul(y_clamp) + self.strides[2].wrapping_mul(z_clamp)
    }

    /// Returns true if the given coordinates are within the cell padding margin.
    /// False otherwise.
    /// Zero padding will always return false.
    pub fn is_margin(&self, x: u32, y: u32, z: u32, cell_padding: u32) -> bool {
        (x < cell_padding || x >= self.dim[0] - cell_padding)
            || (y < cell_padding || y >= self.dim[1] - cell_padding)
            || (z < cell_padding || z >= self.dim[2] - cell_padding)
    }

    /// Returns the delinearized coordinates of the given index.
    pub fn delinearize(&self, mut i: u32) -> [u32; 3] {
        let z = i / self.strides[2];
        i -= z * self.strides[2];
        let y = i / self.strides[1];
        let x = i % self.strides[1];
        [x, y, z]
    }

    /// Creates box-blurred data in a new volume grid with the given blur radius.
    pub fn blur(&self, radius: u32, weight: f32) -> Self {
        let mut out = Self::new(1.0, self.dim);

        for i in 0usize..self.size {
            let [x, y, z] = self.delinearize(i as u32);

            let mut avg: f32 = 0.0;
            for tx in x - radius..=x + radius {
                for ty in y - radius..=y + radius {
                    for tz in z - radius..=z + radius {
                        let j = self.linearize(tx, ty, tz);
                        avg += self.data[j as usize];
                    }
                }
            }

            let coverage = radius * 2 + 1;
            avg /= (coverage * coverage * coverage) as f32;

            out.data[i] = self.data[i].lerp(avg, weight);
        }

        out
    }

    /// Sets the minimum SDF distance at the bordering cell margin to +10.0.
    pub fn trim_padding(&mut self, cell_padding: u32) {
        for i in 0usize..self.size {
            let [x, y, z] = self.delinearize(i as u32);

            if self.is_margin(x, y, z, cell_padding) {
                self.set_linear(i, 10.0);
            }
        }
    }

    /// In-place adds noise to the volumetric.
    pub fn noise_add(&mut self, noise: &PerlinField, transform: Mat4, w: f64, amplitude: f32) {
        for i in 0usize..self.size {
            let [x, y, z] = self.delinearize(i as u32);

            let sample_pos = transform.transform_point3(Vec3::new(x as f32, y as f32, z as f32));

            self.data[i] += (noise.sample_single(sample_pos, w) as f32) * amplitude;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeData;

    #[test]
    fn test_volumedata_indexxing() {
        let vol = VolumeData::new(1.0f32, [4, 4, 4]);
        assert_eq!(vol.size, 4 * 4 * 4, "volume size matches expected");

        // Index zero
        assert_eq!(vol.linearize(0, 0, 0), 0, "Linearize at 0,0,0");
        assert_eq!(vol.delinearize(0), [0, 0, 0], "Delinearize index 0");

        // Max index
        let idx_max = (vol.size as u32) - 1;
        assert_eq!(vol.linearize(3, 3, 3), idx_max, "Linearize at 3,3,3");
        assert_eq!(
            vol.delinearize(idx_max),
            [3, 3, 3],
            "Delinearize at index {0}",
            idx_max
        );

        // Test clamping
        assert_eq!(vol.linearize(0, 0, 0), 0, "Linearize at -1,-1,-1");
        assert_eq!(vol.linearize(4, 4, 4), idx_max, "Linearize at 4,4,4");
    }
}
