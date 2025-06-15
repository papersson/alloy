#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _padding: f32,
}

impl Vec3 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            _padding: 0.0,
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::zero()
    }
}
