use crate::math::Vec3;

pub struct GravitySystem {
    pub planet_center: Vec3,
    pub gravity_strength: f32,
}

impl GravitySystem {
    pub fn new(planet_center: Vec3, gravity_strength: f32) -> Self {
        Self {
            planet_center,
            gravity_strength,
        }
    }

    pub fn get_gravity_vector(&self, position: Vec3) -> Vec3 {
        // Vector from position to planet center
        let to_center = self.planet_center.sub(&position);

        // Normalize and scale by gravity strength
        if to_center.length() > 0.0 {
            to_center.normalize().scale(self.gravity_strength)
        } else {
            Vec3::zero() // At center, no gravity
        }
    }

    pub fn get_up_vector(&self, position: Vec3) -> Vec3 {
        // Up vector is opposite of gravity direction
        let from_center = position.sub(&self.planet_center);

        if from_center.length() > 0.0 {
            from_center.normalize()
        } else {
            Vec3::new(0.0, 1.0, 0.0) // Default up if at center
        }
    }

    pub fn get_surface_distance(&self, position: Vec3, planet_radius: f32) -> f32 {
        // Distance from planet center minus radius
        position.sub(&self.planet_center).length() - planet_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gravity_system_creation() {
        let center = Vec3::new(0.0, -50.0, 0.0);
        let gravity = GravitySystem::new(center, 9.8);
        assert_eq!(gravity.planet_center, center);
        assert_eq!(gravity.gravity_strength, 9.8);
    }

    #[test]
    fn test_gravity_vector() {
        let gravity = GravitySystem::new(Vec3::zero(), 10.0);

        // Test gravity from above
        let pos_above = Vec3::new(0.0, 10.0, 0.0);
        let grav_vec = gravity.get_gravity_vector(pos_above);
        assert!((grav_vec.x - 0.0).abs() < 0.01);
        assert!((grav_vec.y - -10.0).abs() < 0.01);
        assert!((grav_vec.z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_up_vector() {
        let gravity = GravitySystem::new(Vec3::zero(), 10.0);

        // Test up vector from above
        let pos_above = Vec3::new(0.0, 10.0, 0.0);
        let up_vec = gravity.get_up_vector(pos_above);
        assert!((up_vec.x - 0.0).abs() < 0.01);
        assert!((up_vec.y - 1.0).abs() < 0.01);
        assert!((up_vec.z - 0.0).abs() < 0.01);

        // Test up vector from side
        let pos_side = Vec3::new(10.0, 0.0, 0.0);
        let up_vec_side = gravity.get_up_vector(pos_side);
        assert!((up_vec_side.x - 1.0).abs() < 0.01);
        assert!((up_vec_side.y - 0.0).abs() < 0.01);
        assert!((up_vec_side.z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_surface_distance() {
        let gravity = GravitySystem::new(Vec3::zero(), 10.0);
        let planet_radius = 50.0;

        // Test on surface
        let on_surface = Vec3::new(50.0, 0.0, 0.0);
        let dist = gravity.get_surface_distance(on_surface, planet_radius);
        assert!((dist - 0.0).abs() < 0.01);

        // Test above surface
        let above = Vec3::new(60.0, 0.0, 0.0);
        let dist_above = gravity.get_surface_distance(above, planet_radius);
        assert!((dist_above - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_gravity_at_center() {
        let gravity = GravitySystem::new(Vec3::zero(), 10.0);
        let grav_vec = gravity.get_gravity_vector(Vec3::zero());
        assert_eq!(grav_vec, Vec3::zero());
    }

    #[test]
    fn test_up_vector_at_center() {
        let gravity = GravitySystem::new(Vec3::zero(), 10.0);
        let up_vec = gravity.get_up_vector(Vec3::zero());
        // Should return default up vector
        assert_eq!(up_vec, Vec3::new(0.0, 1.0, 0.0));
    }
}
