#![doc(html_favicon_url = "https://alanocull.com/favicon.ico")]
//! Game development utilities addon for Godot, written in Rust for speed and stability.
//!
//! Contains these nodes:
//! - IslandBuilder
//! - SimulatedRope
//! - QueueFloat
//!
//! Crate Features:
//! - **`godot`** (default) - Enables Godot Engine classes using [godot-rust/gdext](https://github.com/godot-rust/gdext) crate. Disable if you just want to use this as a library.
//! - **`nothreads`** (WIP) - Experimental feature for single-threaded Web exports.

// MODULE DECLARATION //

/// All-purpose utility.
pub mod utils;

/// General math utilities and conversions.
pub mod math {
    /// Rust-only implementation of an Axis-Aligned Bounding Box.
    pub mod bounding_box;
    /// Methods for asserting values are within a given delta, for unit tests.
    pub mod delta;
    /// 3D noise types.
    pub mod noise;
    /// Internal implementation for primitive queues.
    pub mod primqueue;
    /// Ray, plane and point projections.
    pub mod projection;
    /// Types and traits for implementing raycast functions on objects.
    pub mod raycast;
    /// Signed Distance Field math and shape sampling.
    pub mod sdf;
    /// Type conversions for ineroperability between libraries.
    pub mod types;
    /// Volumetric data handling.
    pub mod volumetric;
}
/// Mesh data handling and operating with Godot.
pub mod mesh {
    // Convex Hull algorithms like Quick Hull and related functions.
    // pub mod hull;
    /// Net algorithms like Naive Surface Nets.
    pub mod nets;
    /// TriangleMesh and related classes for handling and operating on 3D geometry.
    pub mod trimesh;

    /// Godot-agnostic Island Builder utilities.
    pub mod island;

    /// Ineroperable mesh data with Godot Engine.
    #[cfg(feature = "godot")]
    pub mod godot;
    /// Point Cloud trait and associated functions for managing large sets of point data.
    /// TODO: decouple this from Godot.
    #[cfg(feature = "godot")]
    pub mod pointcloud;
}
/// Data structures and tools for simulated systems.
pub mod simulation {
    /// Data structures for rope simulation.
    /// Godot-agnostic.
    pub mod rope;
}
/// Physics-server related classes.
#[cfg(feature = "physics_server")]
pub mod physics {
    /// Physics bodies with collision, mass, and intertia.
    pub mod body;
    /// State of physics bodies transform, velocity, and angular velocity.
    pub mod body_state;
    /// Physics object identity types.
    pub mod identity;
    /// Utility structures and functions for raycasting.
    pub mod raycast;
    /// Custom physics server implementation for general use.
    /// Experimental.
    pub mod server;
}
/// Custom animation system for Godot Engine.
pub mod animation {
    /// Animation-focused mixing implementation for HashMaps.
    /// Godot-agnostic.
    pub mod mixable;
    /// Pose data structure.
    /// Godot-agnostic.
    pub mod pose;

    /// Godot-related AnimationSoup classes.
    #[cfg(feature = "godot")]
    pub mod soup;
}

/// Godot-specific tools and classes, including the extension itself.
#[cfg(feature = "godot")]
pub mod classes {
    // IMPORTS //
    use godot::prelude::*;

    /// Island Builder data handling.
    pub mod island_settings;

    /// Godot interfaces for Abyss' IslandBuilder tool.
    pub mod island;

    /// Godot interfaces for primitive queues.
    pub mod primqueue;

    /// Godot interfaces for rope simulations.
    pub mod rope;

    /// Custom physics server implementation for use in StagToolkit simulations.
    /// Highly experimental.
    #[cfg(feature = "physics_server")]
    pub mod physics_server;

    /// Utility functions for managing Godot classes.
    pub mod utils;

    struct StagToolkit;

    #[gdextension]
    #[allow(non_snake_case)]
    unsafe impl ExtensionLibrary for StagToolkit {}
}
