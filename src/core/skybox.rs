//! Skybox rendering for atmospheric effects

use crate::math::{Vec2, Vec3};
use crate::scene::{Mesh, Vertex};

pub struct Skybox {
    pub mesh: Mesh,
}

impl Skybox {
    #[must_use]
    pub fn new() -> Self {
        let mesh = Self::create_skybox_mesh();
        Self { mesh }
    }

    fn create_skybox_mesh() -> Mesh {
        // Create an inverted cube that surrounds the camera
        // The cube is large enough to always be behind everything else
        let size = 500.0;

        let vertices = vec![
            // Front face (looking towards +Z, but inverted so it faces inward)
            Vertex {
                position: Vec3::new(-size, -size, size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
            },
            Vertex {
                position: Vec3::new(size, -size, size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
            },
            Vertex {
                position: Vec3::new(size, size, size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
            },
            Vertex {
                position: Vec3::new(-size, size, size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, -1.0),
            },
            // Back face (looking towards -Z, but inverted)
            Vertex {
                position: Vec3::new(size, -size, -size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-size, -size, -size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-size, size, -size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(size, size, -size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
            },
            // Top face (looking down from +Y, but inverted)
            Vertex {
                position: Vec3::new(-size, size, size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, size, size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, size, -size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-size, size, -size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
            },
            // Bottom face (looking up from -Y, but inverted)
            Vertex {
                position: Vec3::new(-size, -size, -size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, -size, -size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, -size, size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-size, -size, size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
            },
            // Right face (looking left from +X, but inverted)
            Vertex {
                position: Vec3::new(size, -size, size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, -size, -size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, size, -size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(-1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(size, size, size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(-1.0, 0.0, 0.0),
            },
            // Left face (looking right from -X, but inverted)
            Vertex {
                position: Vec3::new(-size, -size, -size),
                tex_coord: Vec2::new(0.0, 1.0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-size, -size, size),
                tex_coord: Vec2::new(1.0, 1.0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-size, size, size),
                tex_coord: Vec2::new(1.0, 0.0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-size, size, -size),
                tex_coord: Vec2::new(0.0, 0.0),
                normal: Vec3::new(1.0, 0.0, 0.0),
            },
        ];

        // Inverted winding order for inward-facing triangles
        let indices = vec![
            // Front face
            0, 2, 1, 0, 3, 2, // Back face
            4, 6, 5, 4, 7, 6, // Top face
            8, 10, 9, 8, 11, 10, // Bottom face
            12, 14, 13, 12, 15, 14, // Right face
            16, 18, 17, 16, 19, 18, // Left face
            20, 22, 21, 20, 23, 22,
        ];

        Mesh { vertices, indices }
    }
}

impl Default for Skybox {
    fn default() -> Self {
        Self::new()
    }
}
