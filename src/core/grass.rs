//! Grass system for rendering instanced grass blades on the spherical world

use crate::math::{Mat4, Vec3, Vec4};
use crate::scene::{InstanceData, InstancedMesh, Mesh};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct GrassSystem {
    instanced_mesh: InstancedMesh,
    planet_radius: f32,
}

impl GrassSystem {
    pub fn new(planet_radius: f32, density: f32) -> Self {
        let base_mesh = Mesh::grass_blade();
        let instances = Self::generate_grass_instances(planet_radius, density);

        let instanced_mesh = InstancedMesh {
            base_mesh,
            instances,
        };

        Self {
            instanced_mesh,
            planet_radius,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn generate_grass_instances(planet_radius: f32, density: f32) -> Vec<InstanceData> {
        let mut instances = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Calculate number of grass blades based on surface area and density
        let surface_area = 4.0 * std::f32::consts::PI * planet_radius * planet_radius;
        let num_blades = (surface_area * density) as usize;

        // Generate grass in clusters for more natural distribution
        let num_clusters = (num_blades / 20).max(50); // Average 20 blades per cluster
        let blades_per_cluster = num_blades / num_clusters;

        for _ in 0..num_clusters {
            // Generate cluster center
            let cluster_u: f32 = rng.gen();
            let cluster_v: f32 = rng.gen();
            let cluster_theta = 2.0 * std::f32::consts::PI * cluster_u;
            let cluster_phi = (1.0 - 2.0 * cluster_v).acos();

            // Generate blades within this cluster
            let cluster_spread = 0.1; // Spread radius for cluster

            for _ in 0..blades_per_cluster {
                // Add small offset from cluster center
                let offset_theta = cluster_theta + (rng.gen::<f32>() - 0.5) * cluster_spread;
                let offset_phi = cluster_phi + (rng.gen::<f32>() - 0.5) * cluster_spread;

                // Convert to cartesian coordinates on sphere surface
                let x = planet_radius * offset_phi.sin() * offset_theta.cos();
                let y = planet_radius * offset_phi.sin() * offset_theta.sin();
                let z = planet_radius * offset_phi.cos();

                let position = Vec3::new(x, y, z);

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

                // Add more size variation
                let scale = 0.5 + rng.gen::<f32>() * 1.0; // 0.5 to 1.5 - wider range

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

                // More natural color variation
                let color_var = rng.gen::<f32>();
                let color_variation = Vec3::new(
                    -0.05 + color_var * 0.1,          // Slight red/yellow tint
                    -0.1 + rng.gen::<f32>() * 0.3,    // -0.1 to 0.2 green variation
                    -0.05 + (1.0 - color_var) * 0.05, // Inverse correlation with red
                );

                instances.push(InstanceData {
                    transform,
                    color_variation,
                    _padding: 0.0,
                });
            }
        }

        instances
    }

    pub fn instanced_mesh(&self) -> &InstancedMesh {
        &self.instanced_mesh
    }
}
