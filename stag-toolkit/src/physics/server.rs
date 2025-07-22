use crate::physics::body::PhysicsBody;
use crate::physics::identity::Identity;
use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::atomic::AtomicU64;

#[derive(Copy, Clone, Default, Debug)]
pub struct PhysicsServerSettings {
    /// How many physics frames to keep a hold of.
    /// Set to 0 for no history recording, enabling better performance.
    history_count: u32,
}

/// A "frame" or slice of time in the physics server.
pub struct PhysicsFrame {
    bodies: RwLock<HashMap<Identity, PhysicsBody>>,
}

pub struct PhysicsServer {
    settings: PhysicsServerSettings,
    allocations: AtomicU64,

    /// Current physics "frame" or tick.
    /// https://rust-guide.com/en/documentation/concurrency/RwLock
    current: PhysicsFrame,
    /// Recorded history of physics frames.
    /// TODO: use a queue system like FloatQueue
    history: RwLock<Vec<PhysicsFrame>>,
}

impl PhysicsFrame {
    pub fn default() -> Self {
        Self {
            bodies: RwLock::new(HashMap::new()),
        }
    }
}

impl PhysicsServer {
    pub fn new(settings: PhysicsServerSettings) -> Self {
        Self {
            settings,
            allocations: AtomicU64::new(0),
            current: PhysicsFrame::default(),
            history: RwLock::new(Vec::new()),
        }
    }

    pub fn register_body(&mut self, mut body: PhysicsBody) {
        // Assign a unique ID to the body if it doesn't already have one
        if body.id == 0 {
            body.id = self.get_allocation_id();
        }

        let mut frame_bodies = self.current.bodies.write().unwrap();
        if frame_bodies.contains_key(&body.id) {
            // error: body already included!
            return;
        }

        frame_bodies.insert(body.id, body);
    }

    pub fn get_allocation_id(&self) -> Identity {
        let prev = self
            .allocations
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        prev + 1
    }
}
