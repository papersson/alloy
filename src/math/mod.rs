//! Custom math library with SIMD-aligned types
//!
//! This module provides mathematical types and operations optimized for 3D graphics:
//! - SIMD-aligned vector types (Vec2, Vec3, Vec4)
//! - 4x4 matrix operations
//! - Transform utilities
//! - Camera projection matrices
//!
//! All types are 16-byte aligned for optimal SIMD performance.

/// 2D vector with 16-byte alignment for SIMD operations
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
    _padding: [f32; 2],
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            _padding: [0.0, 0.0],
        }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::zero()
    }
}

/// 3D vector with 16-byte alignment for SIMD operations
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

    #[must_use]
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[must_use]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[must_use]
    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    #[must_use]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len, self.z / len)
        } else {
            *self
        }
    }

    #[must_use]
    pub fn add(&self, other: &Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    #[must_use]
    pub fn sub(&self, other: &Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[must_use]
    pub fn scale(&self, s: f32) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::zero()
    }
}

/// 4D vector with natural 16-byte alignment
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[must_use]
    pub fn normalize_plane(&self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len > 0.0 {
            Self::new(self.x / len, self.y / len, self.z / len, self.w / len)
        } else {
            *self
        }
    }
}

impl Default for Vec4 {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<Vec3> for Vec4 {
    fn from(v: Vec3) -> Self {
        Self::new(v.x, v.y, v.z, 1.0)
    }
}

/// 4x4 matrix for 3D transformations (column-major order)
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub cols: [Vec4; 4],
}

impl Mat4 {
    #[must_use]
    pub const fn new(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self {
            cols: [col0, col1, col2, col3],
        }
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self::new(Vec4::zero(), Vec4::zero(), Vec4::zero(), Vec4::zero())
    }

    #[must_use]
    pub fn multiply(&self, other: &Self) -> Self {
        // Direct access to avoid bounds checking overhead
        let a = &self.cols;
        let b = &other.cols;

        // Compute each column of the result
        let col0 = Vec4::new(
            a[0].x * b[0].x + a[1].x * b[0].y + a[2].x * b[0].z + a[3].x * b[0].w,
            a[0].y * b[0].x + a[1].y * b[0].y + a[2].y * b[0].z + a[3].y * b[0].w,
            a[0].z * b[0].x + a[1].z * b[0].y + a[2].z * b[0].z + a[3].z * b[0].w,
            a[0].w * b[0].x + a[1].w * b[0].y + a[2].w * b[0].z + a[3].w * b[0].w,
        );

        let col1 = Vec4::new(
            a[0].x * b[1].x + a[1].x * b[1].y + a[2].x * b[1].z + a[3].x * b[1].w,
            a[0].y * b[1].x + a[1].y * b[1].y + a[2].y * b[1].z + a[3].y * b[1].w,
            a[0].z * b[1].x + a[1].z * b[1].y + a[2].z * b[1].z + a[3].z * b[1].w,
            a[0].w * b[1].x + a[1].w * b[1].y + a[2].w * b[1].z + a[3].w * b[1].w,
        );

        let col2 = Vec4::new(
            a[0].x * b[2].x + a[1].x * b[2].y + a[2].x * b[2].z + a[3].x * b[2].w,
            a[0].y * b[2].x + a[1].y * b[2].y + a[2].y * b[2].z + a[3].y * b[2].w,
            a[0].z * b[2].x + a[1].z * b[2].y + a[2].z * b[2].z + a[3].z * b[2].w,
            a[0].w * b[2].x + a[1].w * b[2].y + a[2].w * b[2].z + a[3].w * b[2].w,
        );

        let col3 = Vec4::new(
            a[0].x * b[3].x + a[1].x * b[3].y + a[2].x * b[3].z + a[3].x * b[3].w,
            a[0].y * b[3].x + a[1].y * b[3].y + a[2].y * b[3].z + a[3].y * b[3].w,
            a[0].z * b[3].x + a[1].z * b[3].y + a[2].z * b[3].z + a[3].z * b[3].w,
            a[0].w * b[3].x + a[1].w * b[3].y + a[2].w * b[3].z + a[3].w * b[3].w,
        );

        Self::new(col0, col1, col2, col3)
    }

    #[must_use]
    pub fn multiply_vec4(&self, v: &Vec4) -> Vec4 {
        Vec4::new(
            self.cols[0].x * v.x
                + self.cols[1].x * v.y
                + self.cols[2].x * v.z
                + self.cols[3].x * v.w,
            self.cols[0].y * v.x
                + self.cols[1].y * v.y
                + self.cols[2].y * v.z
                + self.cols[3].y * v.w,
            self.cols[0].z * v.x
                + self.cols[1].z * v.y
                + self.cols[2].z * v.z
                + self.cols[3].z * v.w,
            self.cols[0].w * v.x
                + self.cols[1].w * v.y
                + self.cols[2].w * v.z
                + self.cols[3].w * v.w,
        )
    }

    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Result<f32, String> {
        match (row, col) {
            (0, 0) => Ok(self.cols[0].x),
            (1, 0) => Ok(self.cols[0].y),
            (2, 0) => Ok(self.cols[0].z),
            (3, 0) => Ok(self.cols[0].w),
            (0, 1) => Ok(self.cols[1].x),
            (1, 1) => Ok(self.cols[1].y),
            (2, 1) => Ok(self.cols[1].z),
            (3, 1) => Ok(self.cols[1].w),
            (0, 2) => Ok(self.cols[2].x),
            (1, 2) => Ok(self.cols[2].y),
            (2, 2) => Ok(self.cols[2].z),
            (3, 2) => Ok(self.cols[2].w),
            (0, 3) => Ok(self.cols[3].x),
            (1, 3) => Ok(self.cols[3].y),
            (2, 3) => Ok(self.cols[3].z),
            (3, 3) => Ok(self.cols[3].w),
            _ => Err(format!("Matrix index out of bounds: ({}, {})", row, col)),
        }
    }

    pub fn set(&mut self, row: usize, col: usize, value: f32) -> Result<(), String> {
        match (row, col) {
            (0, 0) => {
                self.cols[0].x = value;
                Ok(())
            }
            (1, 0) => {
                self.cols[0].y = value;
                Ok(())
            }
            (2, 0) => {
                self.cols[0].z = value;
                Ok(())
            }
            (3, 0) => {
                self.cols[0].w = value;
                Ok(())
            }
            (0, 1) => {
                self.cols[1].x = value;
                Ok(())
            }
            (1, 1) => {
                self.cols[1].y = value;
                Ok(())
            }
            (2, 1) => {
                self.cols[1].z = value;
                Ok(())
            }
            (3, 1) => {
                self.cols[1].w = value;
                Ok(())
            }
            (0, 2) => {
                self.cols[2].x = value;
                Ok(())
            }
            (1, 2) => {
                self.cols[2].y = value;
                Ok(())
            }
            (2, 2) => {
                self.cols[2].z = value;
                Ok(())
            }
            (3, 2) => {
                self.cols[2].w = value;
                Ok(())
            }
            (0, 3) => {
                self.cols[3].x = value;
                Ok(())
            }
            (1, 3) => {
                self.cols[3].y = value;
                Ok(())
            }
            (2, 3) => {
                self.cols[3].z = value;
                Ok(())
            }
            (3, 3) => {
                self.cols[3].w = value;
                Ok(())
            }
            _ => Err(format!("Matrix index out of bounds: ({}, {})", row, col)),
        }
    }

    #[must_use]
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(x, y, z, 1.0),
        )
    }

    #[must_use]
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            Vec4::new(x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn rotation_x(angle_rad: f32) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, c, -s, 0.0),
            Vec4::new(0.0, s, c, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn rotation_y(angle_rad: f32) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self::new(
            Vec4::new(c, 0.0, s, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(-s, 0.0, c, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn rotation_z(angle_rad: f32) -> Self {
        let c = angle_rad.cos();
        let s = angle_rad.sin();
        Self::new(
            Vec4::new(c, -s, 0.0, 0.0),
            Vec4::new(s, c, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    #[must_use]
    pub fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let range = far - near;
        Self::new(
            Vec4::new(f / aspect, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -(far + near) / range, -1.0),
            Vec4::new(0.0, 0.0, -(2.0 * far * near) / range, 0.0),
        )
    }

    #[must_use]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let width = right - left;
        let height = top - bottom;
        let depth = far - near;

        Self::new(
            Vec4::new(2.0 / width, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 2.0 / height, 0.0, 0.0),
            Vec4::new(0.0, 0.0, -2.0 / depth, 0.0),
            Vec4::new(
                -(right + left) / width,
                -(top + bottom) / height,
                -(far + near) / depth,
                1.0,
            ),
        )
    }

    #[must_use]
    pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Self {
        let f = center.sub(eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(&f);

        Self::new(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        )
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Transform {
    #[must_use]
    pub const fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    #[must_use]
    pub const fn identity() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Vec3::zero(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    #[must_use]
    pub fn to_matrix(&self) -> Mat4 {
        let translation = Mat4::translation(self.position.x, self.position.y, self.position.z);
        let rotation_x = Mat4::rotation_x(self.rotation.x);
        let rotation_y = Mat4::rotation_y(self.rotation.y);
        let rotation_z = Mat4::rotation_z(self.rotation.z);
        let scale = Mat4::scale(self.scale.x, self.scale.y, self.scale.z);

        // Order: Scale -> Rotation X -> Rotation Y -> Rotation Z -> Translation
        let rotation = rotation_z.multiply(&rotation_y).multiply(&rotation_x);
        translation.multiply(&rotation).multiply(&scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod vec2_tests {
        use super::*;

        #[test]
        fn test_new() {
            let v = Vec2::new(1.0, 2.0);
            assert_eq!(v.x, 1.0);
            assert_eq!(v.y, 2.0);
        }

        #[test]
        fn test_zero() {
            let v = Vec2::zero();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
        }

        #[test]
        fn test_default() {
            let v = Vec2::default();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
        }

        #[test]
        fn test_equality() {
            let v1 = Vec2::new(1.0, 2.0);
            let v2 = Vec2::new(1.0, 2.0);
            let v3 = Vec2::new(1.0, 3.0);
            assert_eq!(v1, v2);
            assert_ne!(v1, v3);
        }
    }

    mod vec3_tests {
        use super::*;

        #[test]
        fn test_new() {
            let v = Vec3::new(1.0, 2.0, 3.0);
            assert_eq!(v.x, 1.0);
            assert_eq!(v.y, 2.0);
            assert_eq!(v.z, 3.0);
        }

        #[test]
        fn test_zero() {
            let v = Vec3::zero();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
            assert_eq!(v.z, 0.0);
        }

        #[test]
        fn test_default() {
            let v = Vec3::default();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
            assert_eq!(v.z, 0.0);
        }

        #[test]
        fn test_add() {
            let v1 = Vec3::new(1.0, 2.0, 3.0);
            let v2 = Vec3::new(4.0, 5.0, 6.0);
            let result = v1.add(&v2);
            assert_eq!(result.x, 5.0);
            assert_eq!(result.y, 7.0);
            assert_eq!(result.z, 9.0);
        }

        #[test]
        fn test_sub() {
            let v1 = Vec3::new(4.0, 5.0, 6.0);
            let v2 = Vec3::new(1.0, 2.0, 3.0);
            let result = v1.sub(&v2);
            assert_eq!(result.x, 3.0);
            assert_eq!(result.y, 3.0);
            assert_eq!(result.z, 3.0);
        }

        #[test]
        fn test_scale() {
            let v = Vec3::new(1.0, 2.0, 3.0);
            let result = v.scale(2.0);
            assert_eq!(result.x, 2.0);
            assert_eq!(result.y, 4.0);
            assert_eq!(result.z, 6.0);
        }

        #[test]
        fn test_dot() {
            let v1 = Vec3::new(1.0, 2.0, 3.0);
            let v2 = Vec3::new(4.0, 5.0, 6.0);
            let result = v1.dot(&v2);
            assert_eq!(result, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        }

        #[test]
        fn test_cross() {
            let v1 = Vec3::new(1.0, 0.0, 0.0);
            let v2 = Vec3::new(0.0, 1.0, 0.0);
            let result = v1.cross(&v2);
            assert_eq!(result.x, 0.0);
            assert_eq!(result.y, 0.0);
            assert_eq!(result.z, 1.0);

            // Test anti-commutativity
            let result2 = v2.cross(&v1);
            assert_eq!(result2.x, 0.0);
            assert_eq!(result2.y, 0.0);
            assert_eq!(result2.z, -1.0);
        }

        #[test]
        fn test_length() {
            let v = Vec3::new(3.0, 4.0, 0.0);
            assert_eq!(v.length(), 5.0); // 3-4-5 triangle

            let v2 = Vec3::new(1.0, 2.0, 2.0);
            assert_eq!(v2.length(), 3.0); // sqrt(1 + 4 + 4) = 3
        }

        #[test]
        fn test_normalize() {
            let v = Vec3::new(3.0, 4.0, 0.0);
            let normalized = v.normalize();
            assert!((normalized.length() - 1.0).abs() < 1e-6);
            assert_eq!(normalized.x, 0.6);
            assert_eq!(normalized.y, 0.8);
            assert_eq!(normalized.z, 0.0);
        }

        #[test]
        fn test_normalize_zero_vector() {
            let v = Vec3::zero();
            let normalized = v.normalize();
            assert_eq!(normalized, v); // Zero vector should remain zero
        }
    }

    mod vec4_tests {
        use super::*;

        #[test]
        fn test_new() {
            let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
            assert_eq!(v.x, 1.0);
            assert_eq!(v.y, 2.0);
            assert_eq!(v.z, 3.0);
            assert_eq!(v.w, 4.0);
        }

        #[test]
        fn test_zero() {
            let v = Vec4::zero();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
            assert_eq!(v.z, 0.0);
            assert_eq!(v.w, 0.0);
        }

        #[test]
        fn test_default() {
            let v = Vec4::default();
            assert_eq!(v.x, 0.0);
            assert_eq!(v.y, 0.0);
            assert_eq!(v.z, 0.0);
            assert_eq!(v.w, 0.0);
        }

        #[test]
        fn test_from_vec3() {
            let v3 = Vec3::new(1.0, 2.0, 3.0);
            let v4: Vec4 = v3.into();
            assert_eq!(v4.x, 1.0);
            assert_eq!(v4.y, 2.0);
            assert_eq!(v4.z, 3.0);
            assert_eq!(v4.w, 1.0);
        }

        #[test]
        fn test_dot() {
            let v1 = Vec4::new(1.0, 2.0, 3.0, 4.0);
            let v2 = Vec4::new(5.0, 6.0, 7.0, 8.0);
            let result = v1.dot(&v2);
            assert_eq!(result, 70.0); // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
        }
    }

    mod mat4_tests {
        use super::*;

        #[test]
        fn test_identity() {
            let m = Mat4::identity();
            assert_eq!(m.get(0, 0).unwrap(), 1.0);
            assert_eq!(m.get(1, 1).unwrap(), 1.0);
            assert_eq!(m.get(2, 2).unwrap(), 1.0);
            assert_eq!(m.get(3, 3).unwrap(), 1.0);
            assert_eq!(m.get(0, 1).unwrap(), 0.0);
            assert_eq!(m.get(1, 0).unwrap(), 0.0);
        }

        #[test]
        fn test_zero() {
            let m = Mat4::zero();
            for i in 0..4 {
                for j in 0..4 {
                    assert_eq!(m.get(i, j).unwrap(), 0.0);
                }
            }
        }

        #[test]
        fn test_default() {
            let m = Mat4::default();
            assert_eq!(m, Mat4::identity());
        }

        #[test]
        fn test_multiply_identity() {
            let m = Mat4::identity();
            let result = m.multiply(&m);
            assert_eq!(result, Mat4::identity());
        }

        #[test]
        fn test_multiply_vec4() {
            let m = Mat4::identity();
            let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
            let result = m.multiply_vec4(&v);
            assert_eq!(result, v); // Identity matrix should not change vector
        }

        #[test]
        fn test_translation() {
            let m = Mat4::translation(1.0, 2.0, 3.0);
            let v = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let result = m.multiply_vec4(&v);
            assert_eq!(result.x, 1.0);
            assert_eq!(result.y, 2.0);
            assert_eq!(result.z, 3.0);
            assert_eq!(result.w, 1.0);
        }

        #[test]
        fn test_scale() {
            let m = Mat4::scale(2.0, 3.0, 4.0);
            let v = Vec4::new(1.0, 1.0, 1.0, 1.0);
            let result = m.multiply_vec4(&v);
            assert_eq!(result.x, 2.0);
            assert_eq!(result.y, 3.0);
            assert_eq!(result.z, 4.0);
            assert_eq!(result.w, 1.0);
        }

        #[test]
        fn test_rotation_x() {
            let angle = std::f32::consts::PI / 2.0; // 90 degrees
            let m = Mat4::rotation_x(angle);
            let v = Vec4::new(0.0, 1.0, 0.0, 1.0);
            let result = m.multiply_vec4(&v);

            // Rotation matrix around X: Y -> -Z
            assert!((result.x - 0.0).abs() < 1e-6);
            assert!((result.y - 0.0).abs() < 1e-6);
            assert!((result.z - -1.0).abs() < 1e-6);
            assert!((result.w - 1.0).abs() < 1e-6);
        }

        #[test]
        fn test_rotation_y() {
            let angle = std::f32::consts::PI / 2.0; // 90 degrees
            let m = Mat4::rotation_y(angle);
            let v = Vec4::new(1.0, 0.0, 0.0, 1.0);
            let result = m.multiply_vec4(&v);
            assert!((result.x - 0.0).abs() < 1e-6);
            assert!((result.y - 0.0).abs() < 1e-6);
            assert!((result.z - 1.0).abs() < 1e-6); // Rotating X by 90 degrees around Y gives Z
            assert!((result.w - 1.0).abs() < 1e-6);
        }

        #[test]
        fn test_rotation_z() {
            let angle = std::f32::consts::PI / 2.0; // 90 degrees
            let m = Mat4::rotation_z(angle);
            let v = Vec4::new(1.0, 0.0, 0.0, 1.0);
            let result = m.multiply_vec4(&v);
            assert!((result.x - 0.0).abs() < 1e-6);
            assert!((result.y - -1.0).abs() < 1e-6); // Rotating X by 90 degrees around Z gives -Y
            assert!((result.z - 0.0).abs() < 1e-6);
            assert!((result.w - 1.0).abs() < 1e-6);
        }

        #[test]
        fn test_perspective() {
            let fov = std::f32::consts::PI / 2.0; // 90 degrees
            let aspect = 16.0 / 9.0;
            let near = 0.1;
            let far = 100.0;
            let m = Mat4::perspective(fov, aspect, near, far);

            // Test that it's not identity or zero
            assert_ne!(m, Mat4::identity());
            assert_ne!(m, Mat4::zero());

            // Basic sanity check
            assert!(m.get(0, 0).unwrap() > 0.0);
            assert!(m.get(1, 1).unwrap() > 0.0);
        }

        #[test]
        fn test_orthographic() {
            let m = Mat4::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);

            // Test that it's not identity or zero
            assert_ne!(m, Mat4::identity());
            assert_ne!(m, Mat4::zero());

            // Basic sanity check
            assert_eq!(m.get(0, 0).unwrap(), 1.0);
            assert_eq!(m.get(1, 1).unwrap(), 1.0);
        }

        #[test]
        fn test_look_at() {
            let eye = Vec3::new(0.0, 0.0, 5.0);
            let center = Vec3::new(0.0, 0.0, 0.0);
            let up = Vec3::new(0.0, 1.0, 0.0);
            let m = Mat4::look_at(&eye, &center, &up);

            // Test that it's not identity or zero
            assert_ne!(m, Mat4::identity());
            assert_ne!(m, Mat4::zero());
        }

        #[test]
        fn test_get_set() {
            let mut m = Mat4::zero();
            assert!(m.set(1, 2, 5.0).is_ok());
            assert_eq!(m.get(1, 2).unwrap(), 5.0);

            assert!(m.set(3, 3, 10.0).is_ok());
            assert_eq!(m.get(3, 3).unwrap(), 10.0);
        }

        #[test]
        fn test_get_out_of_bounds() {
            let m = Mat4::identity();
            assert!(m.get(4, 0).is_err());
            assert!(m.get(0, 4).is_err());
            assert!(m.get(5, 5).is_err());
        }

        #[test]
        fn test_set_out_of_bounds() {
            let mut m = Mat4::identity();
            assert!(m.set(0, 4, 1.0).is_err());
            assert!(m.set(4, 0, 1.0).is_err());
            assert!(m.set(5, 5, 1.0).is_err());
        }
    }

    mod transform_tests {
        use super::*;

        #[test]
        fn test_new() {
            let pos = Vec3::new(1.0, 2.0, 3.0);
            let rot = Vec3::new(0.1, 0.2, 0.3);
            let scale = Vec3::new(2.0, 2.0, 2.0);
            let t = Transform::new(pos, rot, scale);
            assert_eq!(t.position, pos);
            assert_eq!(t.rotation, rot);
            assert_eq!(t.scale, scale);
        }

        #[test]
        fn test_identity() {
            let t = Transform::identity();
            assert_eq!(t.position, Vec3::zero());
            assert_eq!(t.rotation, Vec3::zero());
            assert_eq!(t.scale, Vec3::new(1.0, 1.0, 1.0));
        }

        #[test]
        fn test_default() {
            let t = Transform::default();
            assert_eq!(t, Transform::identity());
        }

        #[test]
        fn test_to_matrix_identity() {
            let t = Transform::identity();
            let m = t.to_matrix();
            assert_eq!(m, Mat4::identity());
        }

        #[test]
        fn test_to_matrix_translation_only() {
            let t = Transform {
                position: Vec3::new(1.0, 2.0, 3.0),
                rotation: Vec3::zero(),
                scale: Vec3::new(1.0, 1.0, 1.0),
            };
            let m = t.to_matrix();
            let expected = Mat4::translation(1.0, 2.0, 3.0);
            assert_eq!(m, expected);
        }

        #[test]
        fn test_to_matrix_scale_only() {
            let t = Transform {
                position: Vec3::zero(),
                rotation: Vec3::zero(),
                scale: Vec3::new(2.0, 3.0, 4.0),
            };
            let m = t.to_matrix();
            let expected = Mat4::scale(2.0, 3.0, 4.0);
            assert_eq!(m, expected);
        }

        #[test]
        fn test_to_matrix_combined() {
            let t = Transform {
                position: Vec3::new(1.0, 0.0, 0.0),
                rotation: Vec3::zero(),
                scale: Vec3::new(2.0, 2.0, 2.0),
            };
            let m = t.to_matrix();

            // Test that a point at origin scales then translates correctly
            let v = Vec4::new(1.0, 0.0, 0.0, 1.0);
            let result = m.multiply_vec4(&v);
            assert_eq!(result.x, 3.0); // 1 * 2 + 1 = 3
            assert_eq!(result.y, 0.0);
            assert_eq!(result.z, 0.0);
            assert_eq!(result.w, 1.0);
        }
    }
}
