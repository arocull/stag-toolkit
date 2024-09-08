#![doc(html_favicon_url = "https://alanocull.com/favicon.ico")]
//! Game development utilities addon for Godot, written in Rust for speed and stability.
//!
//! Contains these features:
//! - IslandBuilder
//! - (WIP) AnimationSoup

// MODULE DECLARATION //

/// General math utilities and conversions.
pub mod math {
    /// Signed Distance Field math and shape sampling.
    pub mod sdf;
    /// Type conversions for ineroperability between libraries.
    pub mod types;
    /// Volumetric data handling and 3D noise.
    pub mod volumetric;
}
/// Mesh data handling and inoperating with Godot.
pub mod mesh {
    /// Module for making mesh data ineroperable with Godot Engine.
    pub mod godot;
    /// Module for handling net algorithms like Surface Nets.
    pub mod nets;
    /// Module for handling trimesh data.
    pub mod trimesh;
}

/// Internal implementation and Godot interfaces for Abyss' IslandBuilder tool.
pub mod island;

// IMPORTS //
use godot::prelude::*;

struct StagToolkit;

#[gdextension]
unsafe impl ExtensionLibrary for StagToolkit {}
