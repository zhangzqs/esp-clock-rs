use embedded_graphics::geometry::Point;

#[derive(Debug, Copy, Clone)]
pub struct Vec2f(f32, f32);

impl Vec2f {
    pub fn new(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    pub fn zero() -> Self {
        Self(0.0, 0.0)
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }
}

impl From<Vec2f> for Point {
    fn from(val: Vec2f) -> Self {
        Point::new(val.0 as i32, val.1 as i32)
    }
}

impl From<Vec2f> for (f32, f32) {
    fn from(val: Vec2f) -> Self {
        (val.0, val.1)
    }
}

impl std::ops::Add for Vec2f {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::AddAssign for Vec2f {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self(self.0 + rhs.0, self.1 + rhs.1);
    }
}

impl std::ops::Mul<f32> for Vec2f {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl std::ops::MulAssign<f32> for Vec2f {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self(self.0 * rhs, self.1 * rhs);
    }
}

impl std::ops::Div<f32> for Vec2f {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs, self.1 / rhs)
    }
}

impl std::ops::DivAssign<f32> for Vec2f {
    fn div_assign(&mut self, rhs: f32) {
        *self = Self(self.0 / rhs, self.1 / rhs);
    }
}

impl std::ops::Sub for Vec2f {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl std::ops::SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self(self.0 - rhs.0, self.1 - rhs.1);
    }
}

impl Vec2f {
    fn dot(self, rhs: Self) -> f32 {
        self.0 * rhs.0 + self.1 * rhs.1
    }

    fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    fn normalize(self) -> Self {
        self / self.length()
    }
}
