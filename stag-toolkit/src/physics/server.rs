use crate::math::raycast::{Raycast, RaycastResult, RaycastResultReducer};
use crate::physics::body::PhysicsBody;
use crate::physics::body_state::BodyState;
use crate::physics::identity::Identity;
use crate::physics::raycast::{PhysicsRaycastParameters, PhysicsRaycastResult};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
// https://rust-guide.com/en/documentation/concurrency/Arc
// https://rust-guide.com/en/documentation/concurrency/RwLock

#[derive(Copy, Clone, Default, Debug)]
pub struct PhysicsServerSettings {
    // TODO: How many physics frames to keep a hold of.
    // Set to 0 for no history recording, enabling better performance.
    // pub history_count: u32,
    /// If true, simulates physics bodies moving and colliding.
    pub simulate_bodies: bool,
}

/// A "frame" or slice of time in the physics server.
#[derive(Clone)]
pub struct PhysicsFrame {
    bodies: Arc<RwLock<HashMap<Identity, PhysicsBody>>>,
}

impl Default for PhysicsFrame {
    fn default() -> Self {
        Self {
            bodies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl PhysicsFrame {
    pub fn raycast(
        &self,
        raycast_parameters: PhysicsRaycastParameters,
    ) -> Option<PhysicsRaycastResult> {
        // TODO: potential deadlock, can we limit all these to one mutex?
        match self.bodies.read() {
            Ok(bodies) => {
                let mut results: Vec<RaycastResult> = vec![];
                for (_, body_state) in bodies.iter() {
                    let in_mask = (body_state.layers_colliding & body_state.layers_existing) > 0;

                    // TODO: optimize with an AABB check

                    if in_mask && !body_state.collision.is_empty() {
                        let mut body_tests: Vec<Option<RaycastResult>> =
                            vec![None; body_state.collision.len()];

                        let params = body_state.state.transform.inverse()
                            * raycast_parameters.raycast_parameters;

                        body_tests
                            .par_iter_mut()
                            .enumerate()
                            .for_each(|(idx, result)| {
                                *result = body_state.collision[idx].raycast(params);
                            });

                        if let Some(result) = body_tests.nearest() {
                            results.push(body_state.state.transform * result);
                        }
                    }
                }

                if let Some(result) = results.nearest() {
                    return Some(PhysicsRaycastResult {
                        raycast_result: result,
                        body_identifier: 0,
                    });
                }

                None
            }
            Err(_) => {
                println!("PhysicsFrame: Failed to read mutex.");
                None
            }
        }
    }
}

pub struct PhysicsServer {
    settings: PhysicsServerSettings,
    allocations: AtomicU64,

    /// Current physics "frame" or tick.
    pub current: Arc<PhysicsFrame>,
    // Recorded history of physics frames.
    // TODO: use a queue system like FloatQueue
    // history: Arc<RwLock<Vec<PhysicsFrame>>>,
}

impl PhysicsServer {
    pub fn new(settings: PhysicsServerSettings) -> Self {
        Self {
            settings,
            allocations: AtomicU64::new(0),
            current: Arc::new(PhysicsFrame::default()),
            // history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registers the given physics body with the physics server, returning an identity.
    /// If the ID has already been assigned, returns [None].
    pub fn register_body(&mut self, mut body: PhysicsBody) -> Option<Identity> {
        // Assign a unique ID to the body if it doesn't already have one
        let mut id = body.id;
        if id == 0 {
            id = self.get_allocation_id();
            body.id = id;
        }

        // Insert body
        let mut frame_bodies = self.current.bodies.write().unwrap();
        if frame_bodies.contains_key(&id) {
            // error: body already included!
            return None;
        }
        frame_bodies.insert(id, body);
        Some(id)
    }

    pub fn get_allocation_id(&self) -> Identity {
        let prev = self
            .allocations
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        prev + 1
    }

    pub fn tick(&self, _delta: f32) {
        if self.settings.simulate_bodies {
            todo!("Simulate bodies are not yet implemented");
        }
    }

    /// Returns true on failure.
    pub fn set_body_state(&mut self, identity: Identity, state: BodyState) -> bool {
        if let Some(body) = self.current.bodies.write().unwrap().get_mut(&identity) {
            body.state = state;
            return false;
        }
        true
    }

    pub fn raycast(
        &self,
        raycast_parameters: PhysicsRaycastParameters,
    ) -> Option<PhysicsRaycastResult> {
        self.current.raycast(raycast_parameters)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_allocation_id() {
        let server = PhysicsServer::new(PhysicsServerSettings::default());
        assert_eq!(1, server.get_allocation_id());
        assert_eq!(2, server.get_allocation_id());
        assert_eq!(3, server.get_allocation_id());
    }
}
