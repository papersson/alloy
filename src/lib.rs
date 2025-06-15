//! A minimal 3D game engine for macOS using Metal
//!
//! This crate provides a complete 3D rendering engine built from scratch using
//! Apple's Metal API. It includes custom math types, scene management, input handling,
//! and a UI system.
//!
//! # Example
//! ```no_run
//! use game_engine::app::App;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     App::run()
//! }
//! ```

pub mod app;
pub mod core;
pub mod input;
pub mod math;
pub mod renderer;
pub mod scene;
pub mod ui;
