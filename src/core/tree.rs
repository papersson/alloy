//! Tree system for rendering low-poly trees on the spherical world

use crate::math::{Mat4, Vec2, Vec3, Vec4};
use crate::scene::{InstanceData, InstancedMesh, Mesh, Vertex};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct TreeSystem {
    instanced_mesh: InstancedMesh,
    planet_radius: f32,
}

impl TreeSystem {
    pub fn new(
        planet_radius: f32,
        tree_count: usize,
        road_start_angle: f32,
        road_end_angle: f32,
    ) -> Self {
        let base_mesh = Self::create_tree_mesh();
        let instances = Self::generate_tree_instances(
            planet_radius,
            tree_count,
            road_start_angle,
            road_end_angle,
        );

        let instanced_mesh = InstancedMesh {
            base_mesh,
            instances,
        };

        Self {
            instanced_mesh,
            planet_radius,
        }
    }

    fn create_tree_mesh() -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Create trunk (tapered cylinder approximation using 6 sides)
        let trunk_sides = 6;
        let trunk_radius_bottom = 0.4; // Increased from 0.15
        let trunk_radius_top = 0.2; // Increased from 0.08
        let trunk_height = 3.0; // Increased from 1.0

        // Trunk vertices
        for i in 0..trunk_sides {
            let angle = (i as f32 / trunk_sides as f32) * 2.0 * std::f32::consts::PI;
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();

            // Bottom vertex
            vertices.push(Vertex {
                position: Vec3::new(
                    trunk_radius_bottom * cos_angle,
                    0.0,
                    trunk_radius_bottom * sin_angle,
                ),
                tex_coord: Vec2::new(i as f32 / trunk_sides as f32, 1.0),
                normal: Vec3::new(cos_angle, 0.0, sin_angle),
            });

            // Top vertex
            vertices.push(Vertex {
                position: Vec3::new(
                    trunk_radius_top * cos_angle,
                    trunk_height,
                    trunk_radius_top * sin_angle,
                ),
                tex_coord: Vec2::new(i as f32 / trunk_sides as f32, 0.0),
                normal: Vec3::new(cos_angle, 0.0, sin_angle),
            });
        }

        // Trunk indices
        for i in 0..trunk_sides {
            let current_bottom = (i * 2) as u16;
            let current_top = current_bottom + 1;
            let next_bottom = ((i + 1) % trunk_sides * 2) as u16;
            let next_top = next_bottom + 1;

            // Two triangles per quad
            indices.push(current_bottom);
            indices.push(next_bottom);
            indices.push(current_top);

            indices.push(current_top);
            indices.push(next_bottom);
            indices.push(next_top);
        }

        // Create foliage (simple cone)
        let foliage_base_radius = 2.5; // Increased from 0.8
        let foliage_height = 4.0; // Increased from 1.5
        let foliage_y_offset = trunk_height - 0.5; // Overlap slightly with trunk
        let foliage_sides = 8;

        let vertex_offset = vertices.len() as u16;

        // Foliage tip vertex
        vertices.push(Vertex {
            position: Vec3::new(0.0, foliage_y_offset + foliage_height, 0.0),
            tex_coord: Vec2::new(0.5, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
        });

        // Foliage base vertices
        for i in 0..foliage_sides {
            let angle = (i as f32 / foliage_sides as f32) * 2.0 * std::f32::consts::PI;
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();

            vertices.push(Vertex {
                position: Vec3::new(
                    foliage_base_radius * cos_angle,
                    foliage_y_offset,
                    foliage_base_radius * sin_angle,
                ),
                tex_coord: Vec2::new(i as f32 / foliage_sides as f32, 1.0),
                normal: Vec3::new(cos_angle, 0.5, sin_angle).normalize(),
            });
        }

        // Foliage indices
        for i in 0..foliage_sides {
            let tip = vertex_offset;
            let current_base = vertex_offset + 1 + i as u16;
            let next_base = vertex_offset + 1 + ((i + 1) % foliage_sides) as u16;

            indices.push(tip);
            indices.push(next_base);
            indices.push(current_base);
        }

        Mesh { vertices, indices }
    }

    #[allow(clippy::many_single_char_names)]
    fn generate_tree_instances(
        planet_radius: f32,
        tree_count: usize,
        road_start_angle: f32,
        road_end_angle: f32,
    ) -> Vec<InstanceData> {
        let mut instances = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(123); // Different seed than grass

        // Generate trees avoiding the road path
        let mut attempts = 0;
        while instances.len() < tree_count && attempts < tree_count * 10 {
            attempts += 1;

            // Generate random spherical coordinates
            let u: f32 = rng.gen();
            let v: f32 = rng.gen();

            let theta = 2.0 * std::f32::consts::PI * u;
            let phi = (1.0 - 2.0 * v).acos();

            // Convert to cartesian coordinates on sphere surface
            let x = planet_radius * phi.sin() * theta.cos();
            let y = planet_radius * phi.sin() * theta.sin();
            let z = planet_radius * phi.cos();

            let position = Vec3::new(x, y, z);

            // Check if tree is too close to road (simple check for equator road)
            if y.abs() < 2.5 {
                // Near equator (reduced from 5.0 for smaller planet)
                let angle = x.atan2(z);
                let normalized_angle = if angle < 0.0 {
                    angle + 2.0 * std::f32::consts::PI
                } else {
                    angle
                };

                // Check if within road bounds
                if normalized_angle >= road_start_angle && normalized_angle <= road_end_angle {
                    continue; // Skip this position, it's on the road
                }
            }

            // Calculate up vector (radial from planet center)
            let up = position.normalize();

            // Create a random forward direction in the tangent plane
            let world_up = Vec3::new(0.0, 1.0, 0.0);
            let right = if (up.dot(&world_up).abs() - 1.0).abs() < 0.01 {
                Vec3::new(1.0, 0.0, 0.0)
            } else {
                world_up.cross(&up).normalize()
            };
            let forward = up.cross(&right).normalize();

            // Add some random rotation around the up axis
            let angle: f32 = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();
            let rotated_forward = forward.scale(cos_angle).add(&right.scale(sin_angle));
            let rotated_right = forward.scale(-sin_angle).add(&right.scale(cos_angle));

            // Create transformation matrix
            let mut transform = Mat4::identity();

            // Add some size variation
            let scale = 1.0 + rng.gen::<f32>() * 0.5; // 1.0 to 1.5

            // Set rotation columns (Metal uses column-major)
            transform.cols[0] = Vec4::new(
                rotated_right.x * scale,
                rotated_right.y * scale,
                rotated_right.z * scale,
                0.0,
            );

            transform.cols[1] = Vec4::new(up.x * scale, up.y * scale, up.z * scale, 0.0);

            transform.cols[2] = Vec4::new(
                rotated_forward.x * scale,
                rotated_forward.y * scale,
                rotated_forward.z * scale,
                0.0,
            );

            // Set position column
            transform.cols[3] = Vec4::new(position.x, position.y, position.z, 1.0);

            // Color variation - trunk brown, foliage green
            // We'll use the color_variation to indicate trunk vs foliage in the shader
            let color_variation = Vec3::new(0.0, 0.0, 0.0); // Will be handled in shader

            instances.push(InstanceData {
                transform,
                color_variation,
                _padding: 0.0,
            });
        }

        instances
    }

    pub fn instanced_mesh(&self) -> &InstancedMesh {
        &self.instanced_mesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_creation() {
        let tree_system = TreeSystem::new(50.0, 10, 0.0, std::f32::consts::PI / 2.0);
        assert_eq!(tree_system.planet_radius, 50.0);

        // Check that instances were created
        assert!(!tree_system.instanced_mesh.instances.is_empty());
        assert!(tree_system.instanced_mesh.instances.len() <= 10);
    }

    #[test]
    fn test_tree_mesh_creation() {
        let mesh = TreeSystem::create_tree_mesh();

        // Check that mesh has vertices and indices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());

        // Should have trunk + foliage vertices
        assert!(mesh.vertices.len() > 12); // At least trunk vertices
    }
}
