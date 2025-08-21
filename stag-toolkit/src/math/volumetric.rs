use crate::math::noise::Perlin1D;
use glam::{FloatExt, Mat4, Vec3, Vec4};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

/// A container for storing volume data
pub struct VolumeData<T> {
    /// Internal data for voxel grid.
    pub data: Vec<T>,
    dim: [usize; 3],
    strides: [usize; 3],
    size: usize,
}

/// Utilized for handling
pub struct VolumeWorker<T> {
    /// Volume data being utilized inside the worker.
    pub data: Vec<T>,
    /// Minimum bound of the worker data.
    pub range_min: usize,
    /// Maximum bound of the worker data.
    pub range_max: usize,
    /// Width of the worker data.
    pub range_width: usize,
}

impl<T: Clone + Copy + Default> VolumeData<T> {
    /// Create a new volumetric of the given size with the default value.
    pub fn new(default: T, dim: [usize; 3]) -> Self {
        let size = dim[0] * dim[1] * dim[2];

        Self {
            data: vec![default; size],
            dim,
            strides: [1, dim[0], dim[0] * dim[1]],
            size,
        }
    }

    /// Sets the value at the given linear index
    pub fn set_linear(&mut self, i: usize, val: T) {
        self.data[i] = val;
    }

    /// Returns the value at the given linear index
    pub fn get_linear(&self, i: usize) -> T {
        self.data[i]
    }

    /// Returns the linearized index of the given value.
    pub fn linearize(&self, x: usize, y: usize, z: usize) -> usize {
        x.min(self.dim[0] - 1)
            + self.strides[1].wrapping_mul(y.min(self.dim[1] - 1))
            + self.strides[2].wrapping_mul(z.min(self.dim[2] - 1))
    }

    /// Returns the linearized index of the given value.
    /// **Does not perform checks to ensure the coordinates are within bounds.**
    pub fn linearize_fast(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.strides[1].wrapping_mul(y) + self.strides[2].wrapping_mul(z)
    }

    /// Returns true if the given coordinates are within the cell padding margin.
    /// False otherwise.
    /// Zero padding will always return false.
    pub fn is_margin(&self, x: usize, y: usize, z: usize, cell_padding: usize) -> bool {
        (x < cell_padding || x >= self.dim[0] - cell_padding)
            || (y < cell_padding || y >= self.dim[1] - cell_padding)
            || (z < cell_padding || z >= self.dim[2] - cell_padding)
    }

    /// Returns the delinearized coordinates of the given index.
    pub fn delinearize(&self, mut i: usize) -> [usize; 3] {
        let z = i / self.strides[2];
        i -= z * self.strides[2];
        let y = i / self.strides[1];
        let x = i % self.strides[1];
        [x, y, z]
    }

    /// Sets the value at the bordering cell margin to the given value.
    /// With the Island Builder, this should be minimum SDF distance (suggested: +10.0).
    pub fn set_padding(&mut self, cell_padding: usize, to: T) {
        for i in 0usize..self.size {
            let [x, y, z] = self.delinearize(i);

            if self.is_margin(x, y, z, cell_padding) {
                self.set_linear(i, to);
            }
        }
    }

    /// Returns the dimensions of this Volume.
    pub fn get_dimensions(&self) -> [usize; 3] {
        self.dim
    }

    /// Splits the Volume into a set of worker data for parallel operations.
    /// If `preserve_data` is true, the data of the volume is copied into the vector.
    pub fn to_workers(&self, group_size: usize, preserve_data: bool) -> Vec<VolumeWorker<T>> {
        let worker_count = (self.size as f64 / group_size as f64).ceil() as usize;

        let mut workers: Vec<VolumeWorker<T>> = Vec::with_capacity(worker_count);

        for i in 0..worker_count {
            let range_min: usize = i * group_size;
            let range_max: usize = ((i + 1) * group_size).min(self.size);
            let range_width: usize = range_max - range_min;

            let worker_data: Vec<T>;
            if preserve_data {
                worker_data = Vec::from_iter(self.data[range_min..range_max].iter().copied());
            } else {
                worker_data = vec![T::default(); range_width];
            }

            workers.push(VolumeWorker {
                data: worker_data,
                range_min,
                range_max,
                range_width,
            });
        }

        workers
    }
}

impl<T: Clone + Copy + Default> VolumeWorker<T> {
    /// Copies the corresponding slice from the worker volume.
    pub fn copy_from(&mut self, volume: &VolumeData<T>) {
        #[cfg(debug_assertions)]
        assert!(self.range_max <= volume.size); // Ensure that the worker is within the volume size

        for i in 0..self.range_width {
            self.data[i] = volume.data[i + self.range_min];
        }
    }
}

impl VolumeData<f32> {
    /// Outputs box-blurred data into the given volume grid with the given blur radius.
    pub fn blur(&self, radius: usize, weight: f32, group_size: usize, out: &mut Self) {
        let coverage = radius * 2 + 1;
        let inv_cvg_cubed = 1.0 / (coverage * coverage * coverage) as f32;

        let max_x = self.dim[0] - 1;
        let max_y = self.dim[1] - 1;
        let max_z = self.dim[2] - 1;

        let mut workers = self.to_workers(group_size, false);

        out.data = workers
            .par_iter_mut()
            .flat_map(|worker| -> Vec<f32> {
                for i in 0..worker.range_width {
                    let idx = i + worker.range_min;
                    let [x, y, z] = self.delinearize(idx);

                    let mut avg: f32 = 0.0;
                    for tx in x.saturating_sub(radius)..=(x + radius).min(max_x) {
                        for ty in y.saturating_sub(radius)..=(y + radius).min(max_y) {
                            for tz in z.saturating_sub(radius)..=(z + radius).min(max_z) {
                                avg += self.data[self.linearize_fast(tx, ty, tz)];
                            }
                        }
                    }

                    worker.data[i] = self.data[idx].lerp(avg * inv_cvg_cubed, weight);
                }

                worker.data.clone()
            })
            .collect();
    }

    /// In-place adds noise to the volumetric.
    pub fn noise_add(&mut self, noise: &Perlin1D, transform: Mat4, w: f32) {
        for i in 0usize..self.size {
            let [x, y, z] = self.delinearize(i);

            let sample_pos = transform.transform_point3(Vec3::new(x as f32, y as f32, z as f32));

            self.data[i] += noise.sample(Vec4::from((sample_pos, w))) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeData;

    #[test]
    fn test_volume_data_indexing() {
        let vol = VolumeData::new(1.0f32, [4, 4, 4]);
        assert_eq!(vol.size, 4 * 4 * 4, "volume size matches expected");

        // Index zero
        assert_eq!(vol.linearize(0, 0, 0), 0, "Linearize at 0,0,0");
        assert_eq!(vol.delinearize(0), [0, 0, 0], "Delinearize index 0");

        // Max index
        let idx_max = vol.size - 1;
        assert_eq!(vol.linearize(3, 3, 3), idx_max, "Linearize at 3,3,3");
        assert_eq!(
            vol.delinearize(idx_max),
            [3, 3, 3],
            "Delinearize at index {idx_max}"
        );

        // Test clamping
        assert_eq!(vol.linearize(0, 0, 0), 0, "Linearize at -1,-1,-1");
        assert_eq!(vol.linearize(4, 4, 4), idx_max, "Linearize at 4,4,4");
    }
}
