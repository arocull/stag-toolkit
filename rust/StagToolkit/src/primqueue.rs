use crate::math::types::{packed_float32_array, ToVector2, Vec2Godot};
use glam::vec2;
use godot::{prelude::GodotClass, prelude::*};

/// A queue of floats, used for quickly storing and iterating through a set of data.
/// Can also perform analysis on the data set.
pub struct FloatQueue {
    vals: Vec<f32>,
    idx: usize,
}

impl Default for FloatQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl FloatQueue {
    /// Returns a new FloatQueue.
    pub fn new() -> Self {
        Self {
            vals: vec![0.0],
            idx: 0,
        }
    }

    /// Resizes the float queue, resetting its contents.
    pub fn allocate(&mut self, new_max_size: usize) {
        self.vals.resize(new_max_size, 0.0);
        self.idx = 0;
    }

    /// Returns the queue length.
    pub fn len(&self) -> usize {
        self.vals.len()
    }

    /// Returns the current queue index.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Increments the float queue forward by the given amount.
    fn increment(&mut self, amount: usize) {
        self.idx = (self.idx + amount) % self.vals.len();
    }

    /// Pushes a float onto the queue.
    /// Number cannot be NAN.
    pub fn push(&mut self, new_float: f32) {
        self.vals[self.idx] = new_float;
        self.increment(1);
    }

    /// Returns the minimum and maximum values of the queue.
    pub fn range(&self) -> glam::Vec2 {
        let mut min: f32 = self.vals[0];
        let mut max: f32 = self.vals[0];

        for val in self.vals.iter() {
            min = min.min(*val);
            max = max.max(*val);
        }

        vec2(min, max)
    }

    /// Returns the queue's contents, sorted in ascending order from smallest to greatest.
    pub fn sorted(&self) -> Vec<f32> {
        let mut vect = self.vals.clone();
        vect.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vect
    }

    /// Returns the average of the queue.
    pub fn mean(&self) -> f32 {
        let mut avg = 0.0;
        for val in self.vals.iter() {
            avg += *val;
        }
        avg / (self.vals.len() as f32)
    }

    /// Returns the median of the queue.
    pub fn median(&self) -> f32 {
        let sorted = self.sorted();
        sorted[sorted.len() / 2]
    }

    /// Returns the standard deviation of the queue, using the given average.
    pub fn standard_deviation(&self, average: f32) -> f32 {
        let mut sum = 0.0;
        for val in self.vals.iter() {
            let diff = *val - average;
            sum += diff * diff;
        }
        (sum / (self.vals.len() as f32)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use glam::vec2;

    use super::FloatQueue;

    #[test]
    fn test_floatqueue() {
        let mut queue = FloatQueue::new();

        // Test allocation
        queue.allocate(100);
        assert_eq!(100, queue.len());

        // Test indexing
        assert_eq!(0, queue.index());
        queue.increment(1);
        assert_eq!(1, queue.index());
        queue.increment(105);
        assert_eq!(6, queue.index());

        // Reset queue
        queue.allocate(5);
        assert_eq!(5, queue.len());
        assert_eq!(0, queue.index());

        // Test insertion
        queue.push(9.0);
        queue.push(-3.0);
        queue.push(2.0);
        queue.push(-1.5);
        queue.push(17.0);
        assert_eq!(vec![9.0, -3.0, 2.0, -1.5, 17.0], queue.vals);

        // Test sorting
        assert_eq!(vec![-3.0, -1.5, 2.0, 9.0, 17.0], queue.sorted());

        // Test analysis
        assert_eq!(vec2(-3.0, 17.0), queue.range(), "range");
        assert_eq!(2.0, queue.median(), "median");
        assert_eq!(4.7, queue.mean(), "average");
        assert_eq!(
            7.413_501_3,
            queue.standard_deviation(queue.mean()),
            "standard deviation"
        );
    }
}

// GODOT IMPLEMENTATION //

/// A queue of floats, used for quickly storing and iterating through a set of data.
/// Can also perform analysis on the data set.
#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct QueueFloat {
    queue: FloatQueue,
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for QueueFloat {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            queue: FloatQueue::new(),
            base,
        }
    }
}

#[godot_api]
impl QueueFloat {
    /// Resizes the float queue, resetting its contents.
    #[func]
    pub fn allocate(&mut self, size: i64) {
        self.queue.allocate(size as usize);
    }

    /// Returns the queue length.
    #[func]
    pub fn size(&self) -> i64 {
        self.queue.len() as i64
    }

    /// Returns the current queue index.
    #[func]
    pub fn index(&self) -> i64 {
        self.queue.index() as i64
    }

    /// Pushes a float onto the queue.
    /// Value cannot be NAN.
    #[func]
    pub fn push(&mut self, new_float: f32) {
        if new_float.is_nan() {
            godot_error!("cannot push NAN value onto QueueFloat");
            return;
        }
        self.queue.push(new_float);
    }

    /// Returns the minimum and maximum values of the queue.
    #[func]
    pub fn range(&self) -> Vec2Godot {
        self.queue.range().to_vector2()
    }

    /// Returns the queue's contents, sorted in ascending order from smallest to greatest.
    /// Does not modify the queue.
    #[func]
    pub fn sorted(&self) -> PackedFloat32Array {
        packed_float32_array(self.queue.sorted())
    }

    /// Returns the average of the queue.
    #[func]
    pub fn mean(&self) -> f32 {
        self.queue.mean()
    }

    /// Returns the median of the queue.
    #[func]
    pub fn median(&self) -> f32 {
        self.queue.median()
    }

    /// Returns the standard deviation of the queue.
    #[func]
    pub fn standard_deviation(&self) -> f32 {
        self.queue.standard_deviation(self.queue.mean())
    }
}
