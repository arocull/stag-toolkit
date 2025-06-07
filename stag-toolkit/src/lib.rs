#![doc(html_favicon_url = "https://alanocull.com/favicon.ico")]
//! Game development utilities addon for Godot, written in Rust for speed and stability.
//!
//! Contains these features:
//! - IslandBuilder
//! - SimulatedRope
//! - StagTest (GDScript-only)
//! - (WIP) AnimationSoup
//!
//! If you would like to use the Godot-agnostic libraries, simply disable the `godot` feature.

// MODULE DECLARATION //

/// General math utilities and conversions.
pub mod math {
    /// Methods for asserting values are within a given delta, for unit tests.
    pub mod delta;
    /// Internal implementation for primitive queues.
    pub mod primqueue;
    /// Ray, plane and point projections.
    pub mod projection;
    /// Signed Distance Field math and shape sampling.
    pub mod sdf;
    /// Type conversions for ineroperability between libraries.
    pub mod types;
    /// Volumetric data handling and 3D noise.
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

    /// Godot interfaces for Abyss' IslandBuilder tool.
    pub mod island;

    /// Godot interfaces for primitive queues.
    pub mod primqueue;

    /// Godot interfaces for rope simulations.
    pub mod rope;

    /// Sanity tests.
    #[cfg(debug_assertions)]
    pub mod sanity;

    struct StagToolkit;

    #[gdextension]
    #[allow(non_snake_case)]
    unsafe impl ExtensionLibrary for StagToolkit {}
}
