use std::ops::{Add, Sub, Mul, Div, Neg};
//use std::hash::{Hash, Hasher};
use crate::my_strategy::common::{Square};

#[derive(Default, Clone, Copy, Debug, PartialOrd)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

//    pub fn from_polar(radius: f64, angle: f64) -> Self {
//        Vec2 { x: radius * angle.cos(), y: radius * angle.sin() }
//    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn with_x(&self, x: f64) -> Vec2 {
        Vec2::new(x, self.y)
    }

    pub fn with_y(&self, y: f64) -> Vec2 {
        Vec2::new(self.x, y)
    }

    pub fn with_min_y(&self, y: f64) -> Vec2 {
        Vec2::new(self.x, self.y.max(y))
    }

    pub fn with_neg_x(&self) -> Vec2 {
        self.with_x(-self.x)
    }

    pub fn with_neg_y(&self) -> Vec2 {
        self.with_y(-self.y)
    }

//    pub fn with_dx(&self, dx: f64) -> Vec2 {
//        Vec2::new(self.x + dx, self.y)
//    }
//
//    pub fn with_dy(&self, dy: f64) -> Vec2 {
//        Vec2::new(self.x, self.y + dy)
//    }
//
//    pub fn squared_norm(&self) -> f64 {
//        self.x.square() + self.y.square()
//    }

    pub fn norm(&self) -> f64 {
        (self.x.square() + self.y.square()).sqrt()
    }

//    pub fn squared_distance(&self, other: Vec2) -> f64 {
//        (other - *self).squared_norm()
//    }

    pub fn distance(&self, other: Vec2) -> f64 {
        (other - *self).norm()
    }

//    pub fn rotated(&self, angle: f64) -> Vec2 {
//        let sin = angle.sin();
//        let cos = angle.cos();
//        Vec2::new(self.x * cos - self.y * sin, self.y * cos + self.x * sin)
//    }
//
//    pub fn det(&self, other: Vec2) -> f64 {
//        self.x * other.y - self.y * other.x
//    }
//
//    pub fn square(&self) -> f64 {
//        self.x * self.x + self.y * self.y
//    }
//
//    pub fn cos(&self, other: Vec2) -> f64 {
//        self.dot(other) / (self.norm() * other.norm())
//    }

    pub fn dot(&self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

//    pub fn abs(&self) -> Vec2 {
//        Vec2::new(self.x.abs(), self.y.abs())
//    }

    pub fn normalized(&self) -> Vec2 {
        *self / self.norm()
    }

//    pub fn left_orthogonal(&self) -> Vec2 {
//        Vec2 { x: -self.y, y: self.x }
//    }
//
//    pub fn increased_by(&self, value: f64) -> Vec2 {
//        let norm = self.norm();
//        *self * (norm + value) / norm
//    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

//impl Mul for Vec2 {
//    type Output = Vec2;
//
//    fn mul(self, rhs: Vec2) -> Vec2 {
//        Vec2::new(self.x * rhs.x, self.y * rhs.y)
//    }
//}

impl Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<f64> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x / rhs, self.y / rhs)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, rhs: &Vec2) -> bool {
        (self.x, self.y).eq(&(rhs.x, rhs.y))
    }
}

impl Eq for Vec2 {}

//impl Hash for Vec2 {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        hash_f64(self.x, state);
//        hash_f64(self.y, state);
//    }
//}
