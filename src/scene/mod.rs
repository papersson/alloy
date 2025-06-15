use crate::math::{Mat4, Transform, Vec2, Vec3};
use std::cell::RefCell;
use std::rc::Rc;

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

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn plane(width: f32, depth: f32) -> Self {
        let half_width = width / 2.0;
        let half_depth = depth / 2.0;

        let vertices = vec![
            Vertex {
                position: Vec3::new(-half_width, 0.0, -half_depth),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(half_width, 0.0, -half_depth),
                tex_coord: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(half_width, 0.0, half_depth),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-half_width, 0.0, half_depth),
                tex_coord: Vec2::new(0.0, 1.0),
            },
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        Self { vertices, indices }
    }
}

pub type NodeRef = Rc<RefCell<Node>>;

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub mesh: Option<Mesh>,
    pub children: Vec<NodeRef>,
    parent: Option<NodeRef>,
}

impl Node {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            transform: Transform::identity(),
            mesh: None,
            children: Vec::new(),
            parent: None,
        }
    }

    #[must_use]
    pub fn with_mesh(name: String, mesh: Mesh) -> Self {
        Self {
            name,
            transform: Transform::identity(),
            mesh: Some(mesh),
            children: Vec::new(),
            parent: None,
        }
    }

    pub fn add_child(&mut self, child: NodeRef) {
        self.children.push(child);
    }

    #[must_use]
    pub fn world_transform(&self) -> Mat4 {
        let local_transform = self.transform.to_matrix();
        if let Some(parent) = &self.parent {
            parent.borrow().world_transform().multiply(&local_transform)
        } else {
            local_transform
        }
    }
}

pub struct Scene {
    pub root_nodes: Vec<NodeRef>,
}

impl Scene {
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: NodeRef) {
        self.root_nodes.push(node);
    }

    pub fn traverse<F>(&self, mut callback: F)
    where
        F: FnMut(&Node, &Mat4),
    {
        for root in &self.root_nodes {
            self.traverse_node(&root.borrow(), &Mat4::identity(), &mut callback);
        }
    }

    fn traverse_node<F>(&self, node: &Node, parent_transform: &Mat4, callback: &mut F)
    where
        F: FnMut(&Node, &Mat4),
    {
        let world_transform = parent_transform.multiply(&node.transform.to_matrix());
        callback(node, &world_transform);

        for child in &node.children {
            self.traverse_node(&child.borrow(), &world_transform, callback);
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
