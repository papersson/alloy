# Rust 3D Graphics Engine

A minimal 3D graphics engine built from scratch in Rust, using the Metal API for macOS.

## Features

- Direct Metal rendering pipeline
- Custom math library (Vec3, Mat4, transforms)
- First-person camera with WASD + mouse controls
- Phong lighting model
- Scene graph with hierarchical transforms
- FPS counter overlay

## Requirements

- macOS with Apple Silicon (M1/M2/M3)
- Rust 1.75+
- Xcode Command Line Tools

## Building and Running

### Prerequisites

- macOS with Apple Silicon (M1/M2/M3)
- Rust 1.75 or later
- Xcode Command Line Tools

## Quick Start

```bash
cargo run --release
```

Controls:
- WASD: Move
- Mouse: Look  
- ESC: Exit

## Development

```bash
cargo fmt && cargo clippy -- -W clippy::pedantic && cargo test
```

## Implementation Notes

The engine uses unsafe Metal API calls for GPU interaction. All unsafe blocks document their safety invariants.

This is an educational project focused on understanding 3D graphics programming fundamentals.