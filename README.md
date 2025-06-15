# Rust 3D Game Engine

A minimal 3D game engine built from scratch in Rust, using raw Metal API for macOS Apple Silicon.

## Features

- **Native Metal Rendering**: Direct Metal API integration without abstraction layers
- **Custom Math Library**: SIMD-aligned math types (Vec2/3/4, Mat4) without external dependencies
- **First-Person Controls**: WASD movement with mouse-look camera
- **Scene Graph**: Hierarchical node-based scene management
- **Phong Lighting**: Ambient, diffuse, and specular lighting
- **UI System**: Bitmap font rendering with FPS counter
- **60 FPS Target**: Optimized for smooth performance on Apple Silicon

## Architecture

### Core Modules

- **Core** (`src/core/`): Timer, texture loading, logging
- **Math** (`src/math/`): Custom SIMD-aligned vector and matrix types
- **Input** (`src/input/`): Keyboard and mouse input handling
- **Scene** (`src/scene/`): Scene graph, camera, mesh, and lighting
- **Renderer** (`src/renderer/`): Metal rendering pipeline
- **UI** (`src/ui/`): 2D overlay rendering system
- **App** (`src/app.rs`): Main application loop and window management

### Key Design Decisions

- **No External Math Libraries**: Custom implementation for learning and control
- **Raw Metal API**: Direct GPU programming without wgpu abstractions
- **Minimal Dependencies**: Only winit for windowing and objc2-metal for Metal bindings
- **Type-Driven Development**: Strong typing with Result-based error handling

## Building and Running

### Prerequisites

- macOS with Apple Silicon (M1/M2/M3)
- Rust 1.75 or later
- Xcode Command Line Tools

### Build

```bash
# Debug build
cargo build

# Release build (recommended for performance)
cargo build --release
```

### Run

```bash
# Run in release mode
cargo run --release
```

### Controls

- **WASD**: Move forward/backward/left/right
- **Mouse**: Look around
- **ESC**: Exit application

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with clippy lints
cargo clippy -- -W clippy::pedantic
```

### Code Style

The project follows Rust standard formatting:

```bash
cargo fmt
```

### Performance Profiling

For macOS profiling with Instruments:

```bash
cargo install cargo-instruments
cargo instruments -t "Time Profiler" --release
```

## Project Structure

```
game-engine/
├── src/
│   ├── core/           # Core utilities
│   │   ├── mod.rs
│   │   ├── timer.rs    # Frame timing
│   │   └── texture.rs  # Texture loading
│   ├── math/           # Math library
│   │   └── mod.rs      # Vec2/3/4, Mat4, Transform
│   ├── input/          # Input handling
│   │   └── mod.rs      # Keyboard/mouse state
│   ├── scene/          # Scene management
│   │   └── mod.rs      # Camera, Mesh, Node, Light
│   ├── renderer/       # Rendering
│   │   ├── mod.rs
│   │   ├── scene_renderer.rs  # Main renderer
│   │   └── cube_renderer.rs   # Legacy renderer
│   ├── ui/             # UI system
│   │   ├── mod.rs      # UI renderer
│   │   └── font.rs     # Bitmap font
│   ├── app.rs          # Application loop
│   ├── lib.rs          # Library root
│   └── main.rs         # Entry point
├── assets/
│   └── shaders/
│       ├── scene.metal # 3D scene shaders
│       └── ui.metal    # UI shaders
├── tests/              # Integration tests
└── .claude/            # Project documentation
    └── project/
        └── progress.md # Development progress

```

## Safety

The engine uses unsafe code for Metal API interactions. All unsafe blocks are documented with safety invariants. Key areas:

- Metal object creation and configuration
- GPU buffer memory management
- Objective-C message passing
- Raw pointer casts for drawables

## Future Enhancements

- Shader hot-reloading
- Advanced materials system
- Shadow mapping
- Post-processing effects
- Model loading (.obj, .gltf)
- Audio system
- Physics integration

## License

This project is for educational purposes. Feel free to use it as a learning resource.