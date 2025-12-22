#![doc(html_favicon_url = "https://alanocull.com/favicon.ico")]
//! Real-time solutions for 3D games, art, and simulations.
//!
//! This library provides utility data structures for handling 3D data, such as meshes and volumetrics.
//! Additional utilities like projection and SDF libraries are also provided.
//!
//! ## Module Layout
//!
//! - [`animation`]: under `animation` feature flag, contains experimental animation data structures.
//! - [`classes`]: under `godot` feature flag, contains [Godot-Rust](https://github.com/godot-rust/gdext) classes for use inside [Godot Engine](https://godotengine.org/)'s [Stag Toolkit](https://github.com/arocull/stag-toolkit) addon
//! - [`math`]: Math utilities including projection, volumetrics, Signed Distance Fields, and raycasting.
//! - [`mesh`]: Mesh utilities including the TriangleMesh data structure, and an SDF-based terrain generation system.
//! - [`physics`]: under the `physics_server` feature flag, contains experimental physics-related data structures
//! - [`simulation`]: includes simple a rope simulation
//! - [`utils`]: various utilities used globally across the crate
//!
//! ## Feature Flags
//! - **`animation`** - Enables experimental animation library. Not solidified and will see breaking changes over time.
//! - **`physics_server`** - Enables experimental physics server utilities. Not solidified and will see breaking changes over time.
//! - **`godot`** - Enables [Godot Engine](https://godotengine.org/) classes using [godot-rust/gdext](https://github.com/godot-rust/gdext) crate.
// - **`nothreads`** (WIP) - Experimental feature for single-threaded Web exports.

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
    /// PointCloud trait for managing large sets of point data.
    pub mod pointcloud;
    /// TriangleMesh and related classes for handling and operating on 3D geometry.
    pub mod trimesh;

    /// Godot-agnostic Island Builder utilities.
    pub mod island;

    /// Ineroperable mesh data with Godot Engine.
    ///
    /// Requires `godot` feature flag.
    #[cfg(feature = "godot")]
    pub mod godot;
}
/// Data structures and tools for simulated systems.
pub mod simulation {
    /// Data structures for rope simulation.
    /// Godot-agnostic.
    pub mod rope;
}
/// Physics-server related classes.
///
/// Requires `physics_server` feature flag.
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
#[cfg(feature = "animation")]
pub mod animation {
    /// Animation-focused mixing implementation for HashMaps.
    /// Godot-agnostic.
    pub mod mixable;
    /// Pose data structure.
    /// Godot-agnostic.
    pub mod pose;

    /// Godot-related AnimationSoup classes.
    ///
    /// Requires `godot` feature flag.
    #[cfg(feature = "godot")]
    pub mod soup;
}

/// Godot-specific tools and classes, including the extension itself.
///
/// Requires `godot` feature flag.
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
