use crate::math::{Mat4, Vec2, Vec3};

pub struct Camera {
    position: Vec3,
    target: Vec3,
    up: Vec3,
    fov_y: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
}

impl Camera {
    #[must_use]
    pub fn new(position: Vec3, target: Vec3, aspect_ratio: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            fov_y: std::f32::consts::PI / 4.0,
            aspect_ratio,
            near: 0.1,
            far: 100.0,
        }
    }

    #[must_use]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(&self.position, &self.target, &self.up)
    }

    #[must_use]
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    #[must_use]
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix().multiply(&self.view_matrix())
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_target(&mut self, target: Vec3) {
        self.target = target;
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coord: Vec2,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    #[must_use]
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            // Back face
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            // Top face
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            // Bottom face
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            // Right face
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            // Left face
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                tex_coord: Vec2::new(0.0, 0.0),
            },
        ];

        let indices = vec![
            // Front face
            0, 1, 2, 0, 2, 3, // Back face
            4, 5, 6, 4, 6, 7, // Top face
            8, 9, 10, 8, 10, 11, // Bottom face
            12, 13, 14, 12, 14, 15, // Right face
            16, 17, 18, 16, 18, 19, // Left face
            20, 21, 22, 20, 22, 23,
        ];

        Self { vertices, indices }
    }
}
