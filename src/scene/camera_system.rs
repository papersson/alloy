//! Camera system supporting both first-person and third-person cameras

use crate::core::ThirdPersonCamera;
use crate::math::{Mat4, Vec3};
use crate::scene::Camera;

/// Camera system that can switch between different camera modes
pub enum CameraSystem {
    FirstPerson(Camera),
    ThirdPerson(ThirdPersonCamera),
}

impl CameraSystem {
    pub fn new_first_person(position: Vec3, target: Vec3, aspect_ratio: f32) -> Self {
        Self::FirstPerson(Camera::new(position, target, aspect_ratio))
    }

    pub fn new_third_person(character_position: Vec3, aspect_ratio: f32) -> Self {
        Self::ThirdPerson(ThirdPersonCamera::new(character_position, aspect_ratio))
    }

    pub fn view_matrix(&self) -> Mat4 {
        match self {
            Self::FirstPerson(camera) => camera.view_matrix(),
            Self::ThirdPerson(camera) => camera.view_matrix(),
        }
    }

    pub fn projection_matrix(&self) -> Mat4 {
        match self {
            Self::FirstPerson(camera) => camera.projection_matrix(),
            Self::ThirdPerson(camera) => camera.projection_matrix(),
        }
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        match self {
            Self::FirstPerson(camera) => camera.view_projection_matrix(),
            Self::ThirdPerson(camera) => camera.view_projection_matrix(),
        }
    }

    pub fn position(&self) -> Vec3 {
        match self {
            Self::FirstPerson(camera) => camera.position(),
            Self::ThirdPerson(camera) => camera.camera_position,
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        match self {
            Self::FirstPerson(camera) => camera.set_aspect_ratio(aspect_ratio),
            Self::ThirdPerson(camera) => camera.set_aspect_ratio(aspect_ratio),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        match self {
            Self::FirstPerson(camera) => camera.update(delta_time),
            Self::ThirdPerson(camera) => camera.update(delta_time),
        }
    }
}