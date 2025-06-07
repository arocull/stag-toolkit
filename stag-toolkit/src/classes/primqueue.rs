use crate::math::primqueue::FloatQueue;
use crate::math::types::gdmath::{ToVector2, Vec2Godot, packed_float32_array};
use godot::{prelude::GodotClass, prelude::*};

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

    /// Returns the allocated queue length.
    #[func]
    pub fn size(&self) -> i64 {
        self.queue.len() as i64
    }

    /// Returns the length of queue that has been pushed to.
    #[func]
    pub fn size_used(&self) -> i64 {
        self.queue.len_used() as i64
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
