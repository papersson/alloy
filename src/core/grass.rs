//! Grass system for rendering instanced grass blades on the spherical world

use crate::core::{DensityMap, LodLevel, VegetationInstance, VegetationLodSystem};
use crate::math::{Mat4, Vec3, Vec4};
use crate::scene::{InstanceData, InstancedMesh, Mesh};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct GrassSystem {
    lod_system: VegetationLodSystem,
    instances: Vec<VegetationInstance>,
    #[allow(dead_code)]
    planet_radius: f32,
    #[allow(dead_code)]
    density_map: DensityMap,
}

impl GrassSystem {
    pub fn new(planet_radius: f32, density: f32) -> Self {
        let lod_system = VegetationLodSystem::new();
        let density_map = DensityMap::generate_natural(256, 128);
        let instances = Self::generate_grass_instances(planet_radius, density, &density_map);

        Self {
            lod_system,
            instances,
            planet_radius,
            density_map,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn generate_grass_instances(
        planet_radius: f32,
        density: f32,
        density_map: &DensityMap,
    ) -> Vec<VegetationInstance> {
        let mut instances = Vec::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Calculate base number of candidate positions
        let surface_area = 4.0 * std::f32::consts::PI * planet_radius * planet_radius;
        let num_candidates = (surface_area * density * 2.0) as usize; // Generate more candidates, filter by density

        // Generate grass using density map
        for _ in 0..num_candidates {
            // Generate random position on sphere
            let u: f32 = rng.gen();
            let v: f32 = rng.gen();
            let theta = 2.0 * std::f32::consts::PI * u;
            let phi = (1.0 - 2.0 * v).acos();

            // Convert to cartesian coordinates on sphere surface
            let x = planet_radius * phi.sin() * theta.cos();
            let y = planet_radius * phi.sin() * theta.sin();
            let z = planet_radius * phi.cos();

            let position = Vec3::new(x, y, z);

            // Sample density at this position
            let density_value = density_map.sample_spherical(&position, planet_radius);

            // Use density as probability for placing grass
            if rng.gen::<f32>() < density_value {
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

                // Assign random texture index (0-7 for 8 texture variations)
                let texture_index = rng.gen_range(0..8);

                instances.push(VegetationInstance {
                    transform,
                    color_variation,
                    lod_level: LodLevel::Full, // Will be updated based on distance
                    fade_alpha: 1.0,
                    texture_index,
                });
            }
        }

        instances
    }

    pub fn update(&mut self, view_position: Vec3) {
        self.lod_system.update_view_position(view_position);

        // Update LOD levels for all instances
        for instance in &mut self.instances {
            let instance_pos = Vec3::new(
                instance.transform.cols[3].x,
                instance.transform.cols[3].y,
                instance.transform.cols[3].z,
            );

            let (lod_level, fade_factor) = self.lod_system.calculate_lod_level(instance_pos);
            instance.lod_level = lod_level;
            instance.fade_alpha = if lod_level == LodLevel::Fade {
                1.0 - fade_factor
            } else {
                1.0
            };
        }
    }

    pub fn get_instances_by_lod(&self, lod_level: LodLevel) -> Vec<InstanceData> {
        self.instances
            .iter()
            .filter(|inst| inst.lod_level == lod_level)
            .map(|inst| InstanceData {
                transform: inst.transform,
                color_variation: inst.color_variation,
                lod_level: inst.lod_level as u32,
                texture_index: inst.texture_index,
                _padding: [0; 3],
            })
            .collect()
    }

    pub fn get_lod_mesh(&self, lod_level: LodLevel) -> &Mesh {
        self.lod_system.grass_lods.get_mesh(lod_level)
    }

    pub fn lod_system(&self) -> &VegetationLodSystem {
        &self.lod_system
    }

    // Legacy method for compatibility
    pub fn instanced_mesh(&self) -> InstancedMesh {
        // Return full LOD mesh with all instances for now
        let instances: Vec<InstanceData> = self
            .instances
            .iter()
            .map(|inst| InstanceData {
                transform: inst.transform,
                color_variation: inst.color_variation,
                lod_level: inst.lod_level as u32,
                texture_index: inst.texture_index,
                _padding: [0; 3],
            })
            .collect();

        InstancedMesh {
            base_mesh: self.lod_system.grass_lods.get_mesh(LodLevel::Full).clone(),
            instances,
        }
    }
}
