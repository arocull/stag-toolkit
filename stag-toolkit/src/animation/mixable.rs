use glam::{FloatExt, Quat, Vec3};
use std::{cmp::Eq, collections::HashMap, hash::Hash};

/// Helpers for various animation mixing procedures for HashMaps.
pub trait Mixable {
    /// Linearly interpolates two HashMaps, filling the left-hand side.
    /// If a channel does not exist in one pose or the other, uses whatever existing value there is.
    fn interpolate(&mut self, rhs: &Self, blend: f32);
    /// Adds the right-hand side values (scaled by a weight) to the left-hand side ones.
    /// Right-hand side keys that do not exist in the left-hand-side will be added in.
    fn add(&mut self, rhs: &Self, weight: f32);
    /// Multiplies the left-hand side values by the right-hand side values.
    /// Any right-hand side keys without left-hand side counterparts are ignored.
    fn multiply(&mut self, rhs: &Self);
    /// Scales all values by the given factor.
    fn scale(&mut self, scale: f32);
}

impl<T: Hash + Eq + Copy> Mixable for HashMap<T, f32> {
    fn interpolate(&mut self, rhs: &Self, blend: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Interpolate value if it already exists
                self.insert(*key, orig.lerp(*val, blend));
            } else {
                // Otherwise, insert value
                self.insert(*key, *val);
            }
        }
    }

    fn add(&mut self, rhs: &Self, weight: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Add value if it already exists
                self.insert(*key, orig + (*val) * weight);
            } else {
                // Otherwise, insert value
                self.insert(*key, (*val) * weight);
            }
        }
    }

    fn multiply(&mut self, rhs: &Self) {
        for (key, val) in rhs.iter() {
            // Only multiple value if it exists
            if let Some(orig) = self.get(key) {
                self.insert(*key, orig * (*val));
            }
        }
    }

    fn scale(&mut self, scale: f32) {
        for (_, val) in self.iter_mut() {
            *val *= scale;
        }
    }
}

impl<T: Hash + Eq + Copy> Mixable for HashMap<T, Vec3> {
    fn interpolate(&mut self, rhs: &Self, blend: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Interpolate value if it already exists
                self.insert(*key, orig.lerp(*val, blend));
            } else {
                // Otherwise, insert value
                self.insert(*key, *val);
            }
        }
    }

    fn add(&mut self, rhs: &Self, weight: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Add value if it already exists
                self.insert(*key, orig + (*val) * weight);
            } else {
                // Otherwise, insert value
                self.insert(*key, (*val) * weight);
            }
        }
    }

    fn multiply(&mut self, rhs: &Self) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Only multiply value if it exists
                self.insert(*key, orig * (*val));
            }
        }
    }

    fn scale(&mut self, scale: f32) {
        for (_, val) in self.iter_mut() {
            *val *= scale;
        }
    }
}

// TODO: should we be normalizing these?
impl<T: Hash + Eq + Copy> Mixable for HashMap<T, Quat> {
    fn interpolate(&mut self, rhs: &Self, blend: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Interpolate value if it already exists
                self.insert(*key, orig.slerp(*val, blend));
            } else {
                // Otherwise, insert value
                self.insert(*key, *val);
            }
        }
    }

    fn add(&mut self, rhs: &Self, weight: f32) {
        for (key, val) in rhs.iter() {
            if let Some(orig) = self.get(key) {
                // Combine rotations if both sides already exist
                self.insert(*key, (*orig) * ((*val) * weight));
            } else {
                // Otherwise, insert value
                self.insert(*key, *val);
            }
        }
    }

    /// Performs a rotation multiply.
    fn multiply(&mut self, rhs: &Self) {
        for (key, val) in rhs.iter() {
            // Only multiple value if it exists
            if let Some(orig) = self.get(key) {
                self.insert(*key, (*orig) * (*val));
            }
        }
    }

    fn scale(&mut self, scale: f32) {
        for (_, val) in self.iter_mut() {
            *val *= scale;
        }
    }
}

// TODO: unit tests
