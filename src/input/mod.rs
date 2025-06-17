use std::collections::HashSet;
use winit::keyboard::PhysicalKey;

pub struct InputState {
    pressed_keys: HashSet<PhysicalKey>,
    mouse_delta: (f32, f32),
    mouse_sensitivity: f32,
    movement_speed: f32,
}

impl InputState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            mouse_sensitivity: 0.003,
            movement_speed: 2.0,
        }
    }

    pub fn key_pressed(&mut self, key: PhysicalKey) {
        self.pressed_keys.insert(key);
    }

    pub fn key_released(&mut self, key: PhysicalKey) {
        self.pressed_keys.remove(&key);
    }

    #[must_use]
    pub fn is_key_pressed(&self, key: PhysicalKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn set_mouse_delta(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta = (delta_x, delta_y);
    }

    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    #[must_use]
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    #[must_use]
    pub fn mouse_sensitivity(&self) -> f32 {
        self.mouse_sensitivity
    }

    #[must_use]
    pub fn movement_speed(&self) -> f32 {
        self.movement_speed
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
