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
        let mut result = Self::zero();
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.get(i, k) * other.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
        result
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
    pub fn get(&self, row: usize, col: usize) -> f32 {
        match (row, col) {
            (0, 0) => self.cols[0].x,
            (1, 0) => self.cols[0].y,
            (2, 0) => self.cols[0].z,
            (3, 0) => self.cols[0].w,
            (0, 1) => self.cols[1].x,
            (1, 1) => self.cols[1].y,
            (2, 1) => self.cols[1].z,
            (3, 1) => self.cols[1].w,
            (0, 2) => self.cols[2].x,
            (1, 2) => self.cols[2].y,
            (2, 2) => self.cols[2].z,
            (3, 2) => self.cols[2].w,
            (0, 3) => self.cols[3].x,
            (1, 3) => self.cols[3].y,
            (2, 3) => self.cols[3].z,
            (3, 3) => self.cols[3].w,
            _ => panic!("Index out of bounds"),
        }
    }

    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        match (row, col) {
            (0, 0) => self.cols[0].x = value,
            (1, 0) => self.cols[0].y = value,
            (2, 0) => self.cols[0].z = value,
            (3, 0) => self.cols[0].w = value,
            (0, 1) => self.cols[1].x = value,
            (1, 1) => self.cols[1].y = value,
            (2, 1) => self.cols[1].z = value,
            (3, 1) => self.cols[1].w = value,
            (0, 2) => self.cols[2].x = value,
            (1, 2) => self.cols[2].y = value,
            (2, 2) => self.cols[2].z = value,
            (3, 2) => self.cols[2].w = value,
            (0, 3) => self.cols[3].x = value,
            (1, 3) => self.cols[3].y = value,
            (2, 3) => self.cols[3].z = value,
            (3, 3) => self.cols[3].w = value,
            _ => panic!("Index out of bounds"),
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
