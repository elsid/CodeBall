use std::ops::{Add, Sub, Mul, Div, Neg};
//use std::hash::{Hash, Hasher};
use crate::my_strategy::common::{Square};
use crate::my_strategy::vec2::Vec2;

#[derive(Default, Clone, Copy, Debug, PartialOrd)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    pub const fn i() -> Self {
        Vec3::only_x(1.0)
    }

    pub const fn j() -> Self {
        Vec3::only_y(1.0)
    }

    pub const fn k() -> Self {
        Vec3::only_z(1.0)
    }

    pub const fn only_x(x: f64) -> Self {
        Vec3 { x, y: 0.0, z: 0.0 }
    }

    pub const fn only_y(y: f64) -> Self {
        Vec3 { x: 0.0, y, z: 0.0 }
    }

    pub const fn only_z(z: f64) -> Self {
        Vec3 { x: 0.0, y: 0.0, z }
    }

    pub const fn x(&self) -> f64 {
        self.x
    }

    pub const fn y(&self) -> f64 {
        self.y
    }

    pub const fn z(&self) -> f64 {
        self.z
    }

    pub const fn xy(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub const fn with_x(&self, x: f64) -> Vec3 {
        Vec3::new(x, self.y, self.z)
    }

    pub const fn with_y(&self, y: f64) -> Vec3 {
        Vec3::new(self.x, y, self.z)
    }

    pub const fn with_z(&self, z: f64) -> Vec3 {
        Vec3::new(self.x, self.y, z)
    }

    pub fn with_max_y(&self, y: f64) -> Vec3 {
        Vec3::new(self.x, self.y.max(y), self.z)
    }

    pub fn with_neg_x(&self) -> Vec3 {
        self.with_x(-self.x)
    }

    pub fn with_neg_z(&self) -> Vec3 {
        self.with_z(-self.z)
    }

    pub fn clamp(&self, value: f64) -> Vec3 {
        let norm = self.norm();
        if norm > value {
            *self / norm * value
        } else {
            *self
        }
    }

    pub fn squared_norm(&self) -> f64 {
        self.x.square() + self.y.square() + self.z.square()
    }

    pub fn norm(&self) -> f64 {
        self.squared_norm().sqrt()
    }

    pub fn distance(&self, other: Vec3) -> f64 {
        (other - *self).norm()
    }

    pub fn cos(&self, other: Vec3) -> f64 {
        self.dot(other) / (self.norm() * other.norm())
    }

    pub fn dot(&self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalized(&self) -> Vec3 {
        *self / self.norm()
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Vec3 {
        Vec3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl PartialEq for Vec3 {
    fn eq(&self, rhs: &Vec3) -> bool {
        (self.x, self.y, self.z).eq(&(rhs.x, rhs.y, rhs.z))
    }
}

impl Eq for Vec3 {}
