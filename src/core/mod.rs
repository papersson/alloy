//! Core utilities for the game engine
//!
//! This module provides fundamental utilities including:
//! - High-resolution timing for frame delta calculations
//! - Texture loading and management
//! - Logging macros

mod density_map;
mod grass;
mod grass_texture;
mod gravity;
mod road;
mod skybox;
mod spherical_world;
mod texture;
mod tree;
mod vegetation_lod;

pub use density_map::DensityMap;
pub use grass::GrassSystem;
pub use grass_texture::GrassTextureGenerator;
pub use gravity::GravitySystem;
pub use road::RoadSystem;
pub use skybox::Skybox;
pub use spherical_world::SphericalWorld;
pub use texture::{Texture, TextureArray, TextureFormat};
pub use tree::TreeSystem;
pub use vegetation_lod::{GrassLodMeshes, LodLevel, VegetationInstance, VegetationLodSystem};

use std::time::Instant;

/// High-resolution timer for frame timing
pub struct Timer {
    #[allow(dead_code)]
    start: Instant,
    last_update: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_update: now,
        }
    }

    pub fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        delta
    }

    #[allow(dead_code)]
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        println!("[LOG] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*));
    };
}
