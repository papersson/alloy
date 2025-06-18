//! Level of Detail (LOD) system for vegetation rendering

use crate::math::{Vec2, Vec3};
use crate::scene::{Mesh, Vertex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodLevel {
    Full = 0,      // Full geometric detail (0-10m)
    Reduced = 1,   // Reduced vertices (10-30m)
    Billboard = 2, // Billboard representation (30-50m)
    Fade = 3,      // Fading out (50-60m)
}

impl LodLevel {
    pub fn from_distance(distance: f32) -> Self {
        match distance {
            d if d < 10.0 => Self::Full,
            d if d < 30.0 => Self::Reduced,
            d if d < 50.0 => Self::Billboard,
            _ => Self::Fade,
        }
    }

    pub fn max_distance(&self) -> f32 {
        match self {
            Self::Full => 10.0,
            Self::Reduced => 30.0,
            Self::Billboard => 50.0,
            Self::Fade => 60.0,
        }
    }

    pub fn fade_factor(&self, distance: f32) -> f32 {
        let start_distance = match self {
            Self::Full => 0.0,
            Self::Reduced => 10.0,
            Self::Billboard => 30.0,
            Self::Fade => 50.0,
        };

        let range = self.max_distance() - start_distance;
        let factor = (distance - start_distance) / range;
        factor.clamp(0.0, 1.0)
    }
}

pub struct GrassLodMeshes {
    pub lod_levels: [Mesh; 4],
}

impl GrassLodMeshes {
    pub fn generate() -> Self {
        let lod_levels = [
            Self::generate_full_mesh(),
            Self::generate_reduced_mesh(),
            Self::generate_billboard_mesh(),
            Self::generate_fade_mesh(),
        ];

        Self { lod_levels }
    }

    fn generate_full_mesh() -> Mesh {
        // Full detail grass blade with curved geometry
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate curved grass blade with 5 segments
        let segments = 5;
        let width = 0.05;
        let height = 0.6;
        let curve_amount = 0.1;

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let y = t * height;

            // Apply curve to x position
            let curve_x = curve_amount * t * t;

            // Taper width towards top
            let current_width = width * (1.0 - t * 0.8);

            // Left vertex
            vertices.push(Vertex {
                position: Vec3::new(-current_width + curve_x, y, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0 - t),
            });

            // Right vertex
            vertices.push(Vertex {
                position: Vec3::new(current_width + curve_x, y, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0 - t),
            });
        }

        // Generate indices for triangle strip
        for i in 0..segments {
            let base = i * 2;

            // First triangle
            indices.push(base as u16);
            indices.push((base + 1) as u16);
            indices.push((base + 2) as u16);

            // Second triangle
            indices.push((base + 1) as u16);
            indices.push((base + 3) as u16);
            indices.push((base + 2) as u16);
        }

        Mesh { vertices, indices }
    }

    fn generate_reduced_mesh() -> Mesh {
        // Simplified grass blade with 2 segments
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let segments = 2;
        let width = 0.05;
        let height = 0.6;
        let curve_amount = 0.08;

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let y = t * height;
            let curve_x = curve_amount * t * t;
            let current_width = width * (1.0 - t * 0.8);

            vertices.push(Vertex {
                position: Vec3::new(-current_width + curve_x, y, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0 - t),
            });

            vertices.push(Vertex {
                position: Vec3::new(current_width + curve_x, y, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0 - t),
            });
        }

        for i in 0..segments {
            let base = i * 2;
            indices.push(base as u16);
            indices.push((base + 1) as u16);
            indices.push((base + 2) as u16);
            indices.push((base + 1) as u16);
            indices.push((base + 3) as u16);
            indices.push((base + 2) as u16);
        }

        Mesh { vertices, indices }
    }

    fn generate_billboard_mesh() -> Mesh {
        // Simple quad billboard
        let width = 0.1;
        let height = 0.6;

        let vertices = vec![
            Vertex {
                position: Vec3::new(-width * 0.5, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(width * 0.5, 0.0, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-width * 0.5, height, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(width * 0.5, height, 0.0),
                normal: Vec3::new(0.0, 0.0, 1.0),
                tex_coord: Vec2::new(1.0, 0.0),
            },
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        Mesh { vertices, indices }
    }

    fn generate_fade_mesh() -> Mesh {
        // Same as billboard but will be rendered with transparency
        Self::generate_billboard_mesh()
    }

    pub fn get_mesh(&self, lod_level: LodLevel) -> &Mesh {
        &self.lod_levels[lod_level as usize]
    }
}

#[derive(Debug, Clone)]
pub struct VegetationInstance {
    pub transform: crate::math::Mat4,
    pub color_variation: Vec3,
    pub lod_level: LodLevel,
    pub fade_alpha: f32,
}

pub struct VegetationLodSystem {
    pub grass_lods: GrassLodMeshes,
    pub view_position: Vec3,
}

impl VegetationLodSystem {
    pub fn new() -> Self {
        Self {
            grass_lods: GrassLodMeshes::generate(),
            view_position: Vec3::zero(),
        }
    }

    pub fn update_view_position(&mut self, position: Vec3) {
        self.view_position = position;
    }

    pub fn calculate_lod_level(&self, instance_position: Vec3) -> (LodLevel, f32) {
        let distance = instance_position.sub(&self.view_position).length();
        let lod_level = LodLevel::from_distance(distance);

        // Calculate fade factor for smooth transitions
        let fade_factor = lod_level.fade_factor(distance);

        (lod_level, fade_factor)
    }
}
