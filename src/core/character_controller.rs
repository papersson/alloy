//! Character controller for third-person movement on spherical world

use crate::math::Vec3;

/// Character controller for spherical movement
pub struct CharacterController {
    /// Character position on sphere surface
    pub position: Vec3,
    /// Character forward direction (in tangent plane)
    pub forward: Vec3,
    /// Character up vector (radial from planet center)
    pub up: Vec3,
    /// Movement speed
    pub move_speed: f32,
    /// Run speed multiplier
    pub run_multiplier: f32,
    /// Rotation speed
    pub rotation_speed: f32,
    /// Current velocity for smooth movement
    pub velocity: Vec3,
    /// Acceleration for smooth starts/stops
    pub acceleration: f32,
    /// Deceleration for smooth stops
    pub deceleration: f32,
}

impl CharacterController {
    pub fn new(position: Vec3) -> Self {
        let up = position.normalize();
        // Default forward perpendicular to up
        let forward = if up.y.abs() > 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0).cross(&up).normalize()
        };

        Self {
            position,
            forward,
            up,
            move_speed: 5.0,
            run_multiplier: 2.0,
            rotation_speed: 2.0,
            velocity: Vec3::zero(),
            acceleration: 10.0,
            deceleration: 15.0,
        }
    }

    pub fn update(
        &mut self,
        input_forward: f32,
        input_right: f32,
        is_running: bool,
        delta_time: f32,
        planet_center: Vec3,
        planet_radius: f32,
    ) {
        // Update up vector based on position
        self.up = self.position.sub(&planet_center).normalize();

        // Calculate right vector
        let right = self.forward.cross(&self.up).normalize();

        // Calculate desired movement direction in world space
        let mut desired_direction = Vec3::zero();
        if input_forward.abs() > 0.01 || input_right.abs() > 0.01 {
            desired_direction = self
                .forward
                .scale(input_forward)
                .add(&right.scale(input_right));

            // Project onto tangent plane
            desired_direction = desired_direction
                .sub(&self.up.scale(desired_direction.dot(&self.up)))
                .normalize();

            // Rotate character to face movement direction
            if desired_direction.length() > 0.1 {
                // Smooth rotation towards movement direction
                let target_forward = desired_direction;
                let angle = self.forward.dot(&target_forward).clamp(-1.0, 1.0).acos();
                if angle > 0.01 {
                    let rotation_amount = (self.rotation_speed * delta_time).min(angle);
                    let rotation_axis = self.forward.cross(&target_forward).normalize();

                    // Rotate forward vector
                    let cos_angle = rotation_amount.cos();
                    let sin_angle = rotation_amount.sin();
                    let one_minus_cos = 1.0 - cos_angle;

                    let x = rotation_axis.x;
                    let y = rotation_axis.y;
                    let z = rotation_axis.z;

                    // Rodrigues' rotation formula
                    self.forward = Vec3::new(
                        self.forward.x * (cos_angle + x * x * one_minus_cos)
                            + self.forward.y * (x * y * one_minus_cos - z * sin_angle)
                            + self.forward.z * (x * z * one_minus_cos + y * sin_angle),
                        self.forward.x * (y * x * one_minus_cos + z * sin_angle)
                            + self.forward.y * (cos_angle + y * y * one_minus_cos)
                            + self.forward.z * (y * z * one_minus_cos - x * sin_angle),
                        self.forward.x * (z * x * one_minus_cos - y * sin_angle)
                            + self.forward.y * (z * y * one_minus_cos + x * sin_angle)
                            + self.forward.z * (cos_angle + z * z * one_minus_cos),
                    )
                    .normalize();
                }
            }
        }

        // Update velocity with acceleration/deceleration
        let speed = self.move_speed * if is_running { self.run_multiplier } else { 1.0 };

        if desired_direction.length() > 0.1 {
            // Accelerate towards desired velocity
            let target_velocity = desired_direction.scale(speed);
            let velocity_diff = target_velocity.sub(&self.velocity);
            let accel_step = self.acceleration * delta_time;

            if velocity_diff.length() > accel_step {
                self.velocity = self
                    .velocity
                    .add(&velocity_diff.normalize().scale(accel_step));
            } else {
                self.velocity = target_velocity;
            }
        } else {
            // Decelerate to stop
            let decel_step = self.deceleration * delta_time;
            if self.velocity.length() > decel_step {
                self.velocity = self
                    .velocity
                    .sub(&self.velocity.normalize().scale(decel_step));
            } else {
                self.velocity = Vec3::zero();
            }
        }

        // Update position
        if self.velocity.length() > 0.01 {
            self.position = self.position.add(&self.velocity.scale(delta_time));

            // Constrain to sphere surface
            let from_center = self.position.sub(&planet_center);
            let distance = from_center.length();
            if distance > 0.0 {
                // Character height above surface
                let character_height = 1.0;
                let desired_distance = planet_radius + character_height;
                self.position = planet_center.add(&from_center.scale(desired_distance / distance));
            }
        }

        // Ensure forward is perpendicular to up
        self.forward = self
            .forward
            .sub(&self.up.scale(self.forward.dot(&self.up)))
            .normalize();
    }

    pub fn get_transform_vectors(&self) -> (Vec3, Vec3, Vec3) {
        (self.position, self.forward, self.up)
    }
}
