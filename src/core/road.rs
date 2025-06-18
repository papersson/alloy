//! Road system for rendering curved paths on the spherical world

use crate::math::{Vec2, Vec3};
use crate::scene::{Mesh, Vertex};

pub struct RoadSystem {
    mesh: Mesh,
    planet_radius: f32,
}

impl RoadSystem {
    pub fn new(planet_radius: f32, start_angle: f32, end_angle: f32, width: f32) -> Self {
        let mesh = Self::generate_road_mesh(planet_radius, start_angle, end_angle, width);

        Self {
            mesh,
            planet_radius,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn generate_road_mesh(
        planet_radius: f32,
        start_angle: f32,
        end_angle: f32,
        width: f32,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Number of segments for the road
        let segments = 50;
        let half_width = width / 2.0;

        // Generate vertices along the path
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = start_angle + (end_angle - start_angle) * t;

            // Center point on sphere at this angle (assuming path on equator for now)
            let center_x = planet_radius * angle.cos();
            let center_y = 0.0; // Path follows equator
            let center_z = planet_radius * angle.sin();
            let center = Vec3::new(center_x, center_y, center_z);

            // Calculate up vector (radial from planet center)
            let up = center.normalize();

            // Calculate tangent direction (perpendicular to radial and forward)
            let forward = Vec3::new(-angle.sin(), 0.0, angle.cos());
            let right = forward.cross(&up).normalize();

            // Create vertices at road edges
            let left_pos = center.add(&right.scale(-half_width));
            let right_pos = center.add(&right.scale(half_width));

            // UV coordinates
            let u = t;

            // Left vertex
            vertices.push(Vertex {
                position: left_pos,
                tex_coord: Vec2::new(0.0, u),
                normal: up,
            });

            // Right vertex
            vertices.push(Vertex {
                position: right_pos,
                tex_coord: Vec2::new(1.0, u),
                normal: up,
            });
        }

        // Generate triangles
        for i in 0..segments {
            let base_idx = (i * 2) as u16;

            // First triangle
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 1);

            // Second triangle
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }

        Mesh { vertices, indices }
    }

    pub fn generate_curved_road(
        planet_radius: f32,
        start_pos: Vec3,
        end_pos: Vec3,
        width: f32,
        segments: usize,
    ) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_width = width / 2.0;

        // Normalize positions to sphere surface
        let start_normalized = start_pos.normalize();
        let end_normalized = end_pos.normalize();

        // Generate vertices along the great circle path
        for i in 0..=segments {
            let t = i as f32 / segments as f32;

            // Spherical linear interpolation between start and end
            let angle = start_normalized.dot(&end_normalized).acos();
            let sin_angle = angle.sin();

            let center_normalized = if sin_angle.abs() < 0.001 {
                // Points are too close, use linear interpolation
                start_normalized
                    .scale(1.0 - t)
                    .add(&end_normalized.scale(t))
                    .normalize()
            } else {
                // Standard slerp
                let a = ((1.0 - t) * angle).sin() / sin_angle;
                let b = (t * angle).sin() / sin_angle;
                start_normalized.scale(a).add(&end_normalized.scale(b))
            };

            let center = center_normalized.scale(planet_radius);

            // Calculate up vector (radial from planet center)
            let up = center_normalized;

            // Calculate forward direction (tangent to the path)
            let forward = if i == segments {
                // At the end, use the previous forward direction
                let prev_t = (i - 1) as f32 / segments as f32;
                let prev_angle = ((1.0 - prev_t) * angle).sin() / sin_angle;
                let prev_b = (prev_t * angle).sin() / sin_angle;
                let prev_center = start_normalized
                    .scale(prev_angle)
                    .add(&end_normalized.scale(prev_b));
                center_normalized.sub(&prev_center).normalize()
            } else {
                // Calculate forward as derivative of the slerp
                let next_t = (i + 1) as f32 / segments as f32;
                let next_a = ((1.0 - next_t) * angle).sin() / sin_angle;
                let next_b = (next_t * angle).sin() / sin_angle;
                let next_center = start_normalized
                    .scale(next_a)
                    .add(&end_normalized.scale(next_b));
                next_center.sub(&center_normalized).normalize()
            };

            // Calculate right vector
            let right = forward.cross(&up).normalize();

            // Create vertices at road edges
            let left_pos = center.add(&right.scale(-half_width));
            let right_pos = center.add(&right.scale(half_width));

            // UV coordinates
            let u = t;

            // Left vertex
            vertices.push(Vertex {
                position: left_pos,
                tex_coord: Vec2::new(0.0, u),
                normal: up,
            });

            // Right vertex
            vertices.push(Vertex {
                position: right_pos,
                tex_coord: Vec2::new(1.0, u),
                normal: up,
            });
        }

        // Generate triangles
        for i in 0..segments {
            let base_idx = (i * 2) as u16;

            // First triangle
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 1);

            // Second triangle
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }

        Mesh { vertices, indices }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_road_creation() {
        let road = RoadSystem::new(50.0, 0.0, std::f32::consts::PI / 2.0, 3.0);
        assert_eq!(road.planet_radius, 50.0);

        // Check that mesh has vertices and indices
        assert!(!road.mesh.vertices.is_empty());
        assert!(!road.mesh.indices.is_empty());
    }

    #[test]
    fn test_curved_road_generation() {
        let start = Vec3::new(50.0, 0.0, 0.0);
        let end = Vec3::new(0.0, 0.0, 50.0);
        let mesh = RoadSystem::generate_curved_road(50.0, start, end, 3.0, 20);

        // Should have (segments + 1) * 2 vertices
        assert_eq!(mesh.vertices.len(), 42);

        // Should have segments * 2 triangles * 3 indices
        assert_eq!(mesh.indices.len(), 120);
    }
}
