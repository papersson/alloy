//! Density map system for controlling vegetation distribution

use crate::math::Vec3;

/// Density map for controlling vegetation placement
pub struct DensityMap {
    data: Vec<f32>,
    width: u32,
    height: u32,
}

impl DensityMap {
    /// Creates a new density map with the given dimensions
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![1.0; (width * height) as usize],
            width,
            height,
        }
    }

    /// Creates a procedural density map with natural distribution patterns
    #[must_use]
    pub fn generate_natural(width: u32, height: u32) -> Self {
        let mut data = vec![0.0; (width * height) as usize];

        // Generate using multiple octaves of noise for natural patterns
        for y in 0..height {
            for x in 0..width {
                let u = x as f32 / width as f32;
                let v = y as f32 / height as f32;

                // Base density with multiple noise octaves
                let mut density = 0.5;

                // Large scale variation (continent scale)
                density += Self::noise_2d(u * 4.0, v * 4.0) * 0.3;

                // Medium scale (region scale)
                density += Self::noise_2d(u * 8.0, v * 8.0) * 0.2;

                // Small scale (local variation)
                density += Self::noise_2d(u * 16.0, v * 16.0) * 0.1;

                // Create bare patches
                let patch_noise = Self::noise_2d(u * 6.0 + 100.0, v * 6.0 + 100.0);
                if patch_noise > 0.7 {
                    density *= 0.1; // Sparse area
                }

                // Create dense clusters
                let cluster_noise = Self::noise_2d(u * 5.0 + 200.0, v * 5.0 + 200.0);
                if cluster_noise > 0.6 {
                    density = density.max(0.8); // Dense area
                }

                // Clamp to valid range
                density = density.clamp(0.0, 1.0);

                data[(y * width + x) as usize] = density;
            }
        }

        Self {
            data,
            width,
            height,
        }
    }

    /// Simple 2D noise function for procedural generation
    fn noise_2d(x: f32, y: f32) -> f32 {
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;

        // Smooth interpolation
        let u = xf * xf * (3.0 - 2.0 * xf);
        let v = yf * yf * (3.0 - 2.0 * yf);

        // Hash corners
        let a = Self::hash_2d(xi, yi);
        let b = Self::hash_2d(xi + 1, yi);
        let c = Self::hash_2d(xi, yi + 1);
        let d = Self::hash_2d(xi + 1, yi + 1);

        // Interpolate
        let k1 = a * (1.0 - u) + b * u;
        let k2 = c * (1.0 - u) + d * u;

        k1 * (1.0 - v) + k2 * v
    }

    /// Hash function for noise generation
    fn hash_2d(x: i32, y: i32) -> f32 {
        let mut n = x + y * 57;
        n = (n << 13) ^ n;
        let nn = (n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221)))
            .wrapping_add(1376312589);
        1.0 - (nn & 0x7fffffff) as f32 / 1073741824.0
    }

    /// Samples the density map at spherical coordinates
    pub fn sample_spherical(&self, position: &Vec3, planet_radius: f32) -> f32 {
        // Convert 3D position on sphere to 2D UV coordinates
        let normalized = position.scale(1.0 / planet_radius);

        // Convert to spherical coordinates
        let theta = normalized.y.atan2(normalized.x);
        let phi = normalized.z.acos();

        // Convert to UV (0-1 range)
        let u = (theta + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
        let v = phi / std::f32::consts::PI;

        self.sample_uv(u, v)
    }

    /// Samples the density map at UV coordinates with bilinear filtering
    pub fn sample_uv(&self, u: f32, v: f32) -> f32 {
        // Wrap UV coordinates
        let u = u.fract();
        let v = v.fract();

        // Convert to pixel coordinates
        let x = u * (self.width - 1) as f32;
        let y = v * (self.height - 1) as f32;

        // Get integer coordinates
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        // Get fractional parts
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;

        // Sample four corners
        let v00 = self.data[(y0 * self.width + x0) as usize];
        let v10 = self.data[(y0 * self.width + x1) as usize];
        let v01 = self.data[(y1 * self.width + x0) as usize];
        let v11 = self.data[(y1 * self.width + x1) as usize];

        // Bilinear interpolation
        let v0 = v00 * (1.0 - fx) + v10 * fx;
        let v1 = v01 * (1.0 - fx) + v11 * fx;

        v0 * (1.0 - fy) + v1 * fy
    }

    /// Returns the raw density data for texture creation
    pub fn as_texture_data(&self) -> Vec<u8> {
        // Convert to single-channel 8-bit texture
        self.data
            .iter()
            .map(|&density| (density * 255.0) as u8)
            .collect()
    }

    /// Returns the dimensions of the density map
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_map_creation() {
        let map = DensityMap::new(128, 128);
        assert_eq!(map.dimensions(), (128, 128));
        assert_eq!(map.data.len(), 128 * 128);
    }

    #[test]
    fn test_uv_sampling() {
        let map = DensityMap::new(10, 10);

        // Sample at origin
        let density = map.sample_uv(0.0, 0.0);
        assert!((density - 1.0).abs() < 0.001);

        // Sample at center
        let density = map.sample_uv(0.5, 0.5);
        assert!((density - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_natural_generation() {
        let map = DensityMap::generate_natural(64, 64);

        // Check that we have variation
        let min = map.data.iter().fold(1.0f32, |a, &b| a.min(b));
        let max = map.data.iter().fold(0.0f32, |a, &b| a.max(b));

        assert!(max - min > 0.1, "Natural density map should have variation");
    }
}
