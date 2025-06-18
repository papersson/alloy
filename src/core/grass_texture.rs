//! Procedural grass texture generation for texture arrays

use crate::core::TextureFormat;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Generates procedural grass blade textures with variations
pub struct GrassTextureGenerator {
    width: u32,
    height: u32,
    variations: u32,
}

impl GrassTextureGenerator {
    /// Creates a new grass texture generator
    #[must_use]
    pub fn new(width: u32, height: u32, variations: u32) -> Self {
        Self {
            width,
            height,
            variations,
        }
    }

    /// Generates all grass texture variations as a single concatenated array
    #[must_use]
    pub fn generate_texture_array_data(&self) -> Vec<u8> {
        let mut all_data =
            Vec::with_capacity((self.width * self.height * 4 * self.variations) as usize);

        // Use a seeded RNG for consistent results
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for variation in 0..self.variations {
            let texture_data = self.generate_single_texture(variation, &mut rng);
            all_data.extend_from_slice(&texture_data);
        }

        all_data
    }

    fn generate_single_texture(&self, variation: u32, rng: &mut impl Rng) -> Vec<u8> {
        let mut data = vec![0u8; (self.width * self.height * 4) as usize];

        // Base color variations
        let base_colors = [
            (76, 153, 51),  // Standard green
            (102, 153, 51), // Yellow-green
            (51, 153, 76),  // Blue-green
            (89, 140, 64),  // Darker green
            (115, 161, 82), // Lighter green
            (64, 128, 48),  // Deep green
            (96, 156, 72),  // Mid green
            (83, 145, 58),  // Natural green
        ];

        let (base_r, base_g, base_b) = base_colors[variation as usize % base_colors.len()];

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = ((y * self.width + x) * 4) as usize;

                // Calculate normalized coordinates
                let u = x as f32 / (self.width - 1) as f32;
                let v = y as f32 / (self.height - 1) as f32;

                // Generate grass blade shape (wider at bottom, narrow at top)
                let blade_width = (1.0 - v * 0.8).max(0.2);
                let center_dist = (u - 0.5).abs() * 2.0;

                // Alpha channel defines blade shape
                let alpha = if center_dist <= blade_width {
                    // Smooth edges
                    let edge_fade = 1.0 - (center_dist / blade_width).powi(2);
                    (edge_fade * 255.0) as u8
                } else {
                    0
                };

                // Color variations along the blade
                let color_variation = 1.0 + (v - 0.5) * 0.3; // Lighter at tips

                // Add subtle noise for realism
                let noise = rng.gen_range(-10..=10);

                // Calculate final color
                let r = ((base_r as f32 * color_variation) as i32 + noise).clamp(0, 255) as u8;
                let g = ((base_g as f32 * color_variation) as i32 + noise).clamp(0, 255) as u8;
                let b = ((base_b as f32 * color_variation) as i32 + noise).clamp(0, 255) as u8;

                // Add vein pattern for variations 4-7
                let has_vein = variation >= 4;
                let vein_strength = if has_vein && (u - 0.5).abs() < 0.05 {
                    0.8
                } else {
                    1.0
                };

                data[idx] = (r as f32 * vein_strength) as u8;
                data[idx + 1] = (g as f32 * vein_strength) as u8;
                data[idx + 2] = (b as f32 * vein_strength) as u8;
                data[idx + 3] = alpha;
            }
        }

        // Add spots/wear for some variations
        if variation % 3 == 2 {
            self.add_wear_spots(&mut data, rng);
        }

        data
    }

    fn add_wear_spots(&self, data: &mut [u8], rng: &mut impl Rng) {
        // Add a few random darker spots to simulate wear/damage
        let num_spots = rng.gen_range(3..8);

        for _ in 0..num_spots {
            let spot_x = rng.gen_range(0..self.width);
            let spot_y = rng.gen_range(self.height / 3..self.height); // More wear on upper parts
            let spot_radius = rng.gen_range(2..5);

            for dy in 0..spot_radius * 2 {
                for dx in 0..spot_radius * 2 {
                    let x = (spot_x + dx).saturating_sub(spot_radius);
                    let y = (spot_y + dy).saturating_sub(spot_radius);

                    if x < self.width && y < self.height {
                        let idx = ((y * self.width + x) * 4) as usize;

                        // Check if within circle
                        let dist_sq = ((dx as i32 - spot_radius as i32).pow(2)
                            + (dy as i32 - spot_radius as i32).pow(2))
                            as f32;
                        if dist_sq <= (spot_radius as f32).powi(2) && data[idx + 3] > 128 {
                            // Darken the spot
                            data[idx] = (data[idx] as f32 * 0.7) as u8;
                            data[idx + 1] = (data[idx + 1] as f32 * 0.6) as u8;
                            data[idx + 2] = (data[idx + 2] as f32 * 0.7) as u8;
                        }
                    }
                }
            }
        }
    }

    /// Gets the texture format for the generated textures
    #[must_use]
    pub const fn format() -> TextureFormat {
        TextureFormat::Rgba8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_generation() {
        let generator = GrassTextureGenerator::new(64, 64, 4);
        let data = generator.generate_texture_array_data();

        // Check data size
        assert_eq!(data.len(), 64 * 64 * 4 * 4);

        // Check that we have non-zero alpha values (blade shapes)
        let has_content = data.chunks(4).any(|pixel| pixel[3] > 0);
        assert!(
            has_content,
            "Generated textures should have visible content"
        );
    }
}
