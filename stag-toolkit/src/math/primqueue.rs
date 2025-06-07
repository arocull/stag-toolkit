use glam::vec2;
use std::cmp::Ordering;

/// A queue of floats, used for quickly storing and iterating through a set of data.
/// Can also perform analysis on the data set.
pub struct FloatQueue {
    /// A vector of float values.
    vals: Vec<f32>,
    /// Current index of queue for storing at.
    idx: usize,
    /// Amount of items inside the queue that have been used up.
    used: usize,
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
            used: 1,
        }
    }

    /// Resizes the float queue, resetting its contents.
    pub fn allocate(&mut self, new_max_size: usize) {
        self.vals.resize(new_max_size, 0.0);
        self.idx = 0;
        self.used = 1; // Reset use count
    }

    /// Returns the allocated queue length.
    pub fn len(&self) -> usize {
        self.vals.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.vals.is_empty()
    }

    /// Returns the actual length of the queue that is in use.
    pub fn len_used(&self) -> usize {
        self.used
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
        self.used = self.used.max(self.idx + 1);
        self.increment(1);
    }

    /// Returns the minimum and maximum values of the queue.
    pub fn range(&self) -> glam::Vec2 {
        let mut min: f32 = self.vals[0];
        let mut max: f32 = self.vals[0];

        for (i, val) in self.vals.iter().enumerate() {
            // Don't include unused values
            if i >= self.used {
                break;
            }

            min = min.min(*val);
            max = max.max(*val);
        }

        vec2(min, max)
    }

    /// Returns the queue's contents, sorted in ascending order from smallest to greatest.
    pub fn sorted(&self) -> Vec<f32> {
        let mut vect = self.vals.clone();
        vect.truncate(self.used); // Don't include unused vals
        vect.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        vect
    }

    /// Returns the average of the queue.
    pub fn mean(&self) -> f32 {
        let mut avg = 0.0;
        for (i, val) in self.vals.iter().enumerate() {
            if i >= self.used {
                break;
            }

            avg += *val;
        }
        avg / (self.used as f32)
    }

    /// Returns the median of the queue.
    pub fn median(&self) -> f32 {
        let sorted = self.sorted();
        sorted[self.used / 2]
    }

    /// Returns the standard deviation of the queue, using the given average.
    pub fn standard_deviation(&self, average: f32) -> f32 {
        let mut sum = 0.0;
        for (i, val) in self.vals.iter().enumerate() {
            if i >= self.used {
                break;
            }

            let diff = *val - average;
            sum += diff * diff;
        }
        (sum / (self.used as f32)).sqrt()
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
        assert_eq!(5, queue.len_used());

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

        queue.push(1.0);
        queue.push(1.0);
        assert_eq!(vec![1.0, 1.0, 2.0, -1.5, 17.0], queue.vals);
        assert_eq!(5, queue.len_used());

        // Test that queue analysis considers half-used queue length
        queue.allocate(5);
        queue.push(5.0);
        queue.push(3.0);
        queue.push(4.0);
        assert_eq!(3, queue.len_used(), "queue should only be half-used");
        assert_eq!(4.0, queue.mean());
        assert_eq!(4.0, queue.median());
        assert_eq!(vec![3.0, 4.0, 5.0], queue.sorted());
        assert_eq!(vec2(3.0, 5.0), queue.range());
        assert_eq!(0.816_496_6, queue.standard_deviation(queue.mean()));
    }
}
