#![doc(html_favicon_url = "https://alanocull.com/favicon.ico")]
//! Game development utilities addon for Godot, written in Rust for speed and stability.
//!
//! Contains these features:
//! - IslandBuilder
//! - (WIP) AnimationSoup

// MODULE DECLARATION //

/// General math utilities and conversions.
pub mod math {
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
    /// Ineroperable mesh data with Godot Engine.
    pub mod godot;
    /// Convex Hull algorithms like Quick Hull and related functions.
    pub mod hull;
    /// Net algorithms like Naive Surface Nets.
    pub mod nets;
    /// TriangleMesh and related classes for handling and operating on 3D geometry.
    pub mod trimesh;
}

/// Internal implementation and Godot interfaces for Abyss' IslandBuilder tool.
pub mod island;

// IMPORTS //
use godot::prelude::*;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}
