//! Third-person camera system with spring arm and collision detection

use crate::math::{Mat4, Vec3};

/// Settings for camera behavior and motion sickness prevention
#[derive(Debug, Clone)]
pub struct CameraSettings {
    /// Field of view in radians (default: 90 degrees)
    pub fov: f32,
    /// Camera smoothing factor (0.1-0.25 for motion sickness prevention)
    pub smoothing_factor: f32,
    /// Maximum rotation speed in radians per second
    pub max_rotation_speed: f32,
    /// Invert horizontal camera movement
    pub invert_x: bool,
    /// Invert vertical camera movement
    pub invert_y: bool,
    /// Spring arm length (distance from character)
    pub arm_length: f32,
    /// Camera elevation angle in radians (above character)
    pub elevation_angle: f32,
    /// Camera lag for smooth following
    pub position_lag: f32,
    /// Rotation lag for smooth turning
    pub rotation_lag: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::FRAC_PI_2, // 90 degrees
            smoothing_factor: 0.15,
            max_rotation_speed: 2.0,
            invert_x: false,
            invert_y: false,
            arm_length: 8.0,
            elevation_angle: 0.523, // 30 degrees
            position_lag: 0.1,
            rotation_lag: 0.15,
        }
    }
}

/// Spring arm component for third-person camera
pub struct SpringArm {
    /// Current arm length (may be shorter due to collisions)
    pub current_length: f32,
    /// Target arm length
    pub target_length: f32,
    /// Smoothing factor for length changes
    pub length_smoothing: f32,
}

impl SpringArm {
    pub fn new(length: f32) -> Self {
        Self {
            current_length: length,
            target_length: length,
            length_smoothing: 0.2,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Smooth interpolation of arm length
        let interpolation = 1.0 - (-self.length_smoothing * delta_time).exp();
        self.current_length += (self.target_length - self.current_length) * interpolation;
    }

    pub fn set_collision_distance(&mut self, distance: f32) {
        // Clamp the target length based on collision
        self.target_length = distance.min(self.current_length);
    }

    pub fn reset(&mut self, base_length: f32) {
        self.target_length = base_length;
    }
}

/// Third-person camera with spring arm and smooth following
pub struct ThirdPersonCamera {
    /// Character position (what the camera follows)
    pub character_position: Vec3,
    /// Character forward direction
    pub character_forward: Vec3,
    /// Character up vector (for spherical world)
    pub character_up: Vec3,
    /// Current camera position
    pub camera_position: Vec3,
    /// Target camera position (before smoothing)
    pub target_camera_position: Vec3,
    /// Camera yaw (horizontal rotation)
    pub yaw: f32,
    /// Camera pitch (vertical rotation)
    pub pitch: f32,
    /// Target yaw for smooth rotation
    pub target_yaw: f32,
    /// Target pitch for smooth rotation
    pub target_pitch: f32,
    /// Spring arm for camera distance
    pub spring_arm: SpringArm,
    /// Camera settings
    pub settings: CameraSettings,
    /// Aspect ratio for projection
    pub aspect_ratio: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl ThirdPersonCamera {
    pub fn new(character_position: Vec3, aspect_ratio: f32) -> Self {
        let settings = CameraSettings::default();
        let spring_arm = SpringArm::new(settings.arm_length);

        // Initialize camera behind character
        let character_up = character_position.normalize();
        let character_forward = Vec3::new(1.0, 0.0, 0.0); // Default forward
        let camera_position = character_position
            .sub(&character_forward.scale(settings.arm_length))
            .add(&character_up.scale(settings.elevation_angle.sin() * settings.arm_length));

        Self {
            character_position,
            character_forward,
            character_up,
            camera_position,
            target_camera_position: camera_position,
            yaw: 0.0,
            pitch: settings.elevation_angle,
            target_yaw: 0.0,
            target_pitch: settings.elevation_angle,
            spring_arm,
            settings,
            aspect_ratio,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update spring arm
        self.spring_arm.update(delta_time);

        // Smooth rotation interpolation
        let rotation_interpolation = 1.0 - (-self.settings.rotation_lag * delta_time).exp();
        self.yaw += (self.target_yaw - self.yaw) * rotation_interpolation;
        self.pitch += (self.target_pitch - self.pitch) * rotation_interpolation;

        // Calculate camera offset in spherical coordinates
        let horizontal_distance = self.pitch.cos() * self.spring_arm.current_length;
        let vertical_distance = self.pitch.sin() * self.spring_arm.current_length;

        // Calculate camera position relative to character
        // For spherical world, we need to work in the character's local space
        let right = self.character_forward.cross(&self.character_up).normalize();
        let forward_flat = self.character_up.cross(&right).normalize();

        // Apply yaw rotation
        let yawed_forward = forward_flat
            .scale(self.yaw.cos())
            .add(&right.scale(self.yaw.sin()));

        // Calculate target camera position
        self.target_camera_position = self
            .character_position
            .sub(&yawed_forward.scale(horizontal_distance))
            .add(&self.character_up.scale(vertical_distance));

        // Smooth position interpolation
        let position_interpolation = 1.0 - (-self.settings.position_lag * delta_time).exp();
        let position_diff = self.target_camera_position.sub(&self.camera_position);
        self.camera_position = self
            .camera_position
            .add(&position_diff.scale(position_interpolation));
    }

    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        // Apply inversion settings
        let yaw_mult = if self.settings.invert_x { -1.0 } else { 1.0 };
        let pitch_mult = if self.settings.invert_y { -1.0 } else { 1.0 };

        // Apply rotation with max speed clamping
        let max_delta = self.settings.max_rotation_speed * 0.016; // Assume ~60fps
        self.target_yaw += (yaw_delta * yaw_mult).clamp(-max_delta, max_delta);

        // Clamp pitch to prevent camera flipping
        self.target_pitch = (self.target_pitch + pitch_delta * pitch_mult).clamp(
            -0.1,                              // Just below horizontal
            std::f32::consts::FRAC_PI_2 - 0.1, // Just below vertical
        );
    }

    pub fn set_character_transform(&mut self, position: Vec3, forward: Vec3, up: Vec3) {
        self.character_position = position;
        self.character_forward = forward.normalize();
        self.character_up = up.normalize();
    }

    pub fn view_matrix(&self) -> Mat4 {
        let look_at_target = self.character_position;
        Mat4::look_at(&self.camera_position, &look_at_target, &self.character_up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(self.settings.fov, self.aspect_ratio, self.near, self.far)
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix().multiply(&self.view_matrix())
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn get_forward_direction(&self) -> Vec3 {
        // Camera forward is from camera to character
        self.character_position
            .sub(&self.camera_position)
            .normalize()
    }

    pub fn get_right_direction(&self) -> Vec3 {
        self.get_forward_direction()
            .cross(&self.character_up)
            .normalize()
    }
}
