use std::ops::{Add, Sub, Mul, Div, Neg};
//use std::hash::{Hash, Hasher};
use crate::my_strategy::common::{Square};

#[derive(Default, Clone, Copy, Debug, PartialOrd)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

//    pub fn from_polar(radius: f64, angle: f64) -> Self {
//        Vec3 { x: radius * angle.cos(), y: radius * angle.sin() }
//    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

//    pub fn with_x(&self, x: f64) -> Vec3 {
//        Vec3::new(x, self.y)
//    }

    pub fn with_y(&self, y: f64) -> Vec3 {
        Vec3::new(self.x, y, self.z)
    }

    pub fn with_min_y(&self, y: f64) -> Vec3 {
        Vec3::new(self.x, self.y.max(y), self.z)
    }

//    pub fn with_dx(&self, dx: f64) -> Vec3 {
//        Vec3::new(self.x + dx, self.y)
//    }
//
//    pub fn with_dy(&self, dy: f64) -> Vec3 {
//        Vec3::new(self.x, self.y + dy)
//    }
//
//    pub fn squared_norm(&self) -> f64 {
//        self.x.square() + self.y.square()
//    }

    pub fn norm(&self) -> f64 {
        (self.x.square() + self.y.square() + self.z.square()).sqrt()
    }

//    pub fn squared_distance(&self, other: Vec3) -> f64 {
//        (other - *self).squared_norm()
//    }

    pub fn distance(&self, other: Vec3) -> f64 {
        (other - *self).norm()
    }

//    pub fn rotated(&self, angle: f64) -> Vec3 {
//        let sin = angle.sin();
//        let cos = angle.cos();
//        Vec3::new(self.x * cos - self.y * sin, self.y * cos + self.x * sin)
//    }
//
//    pub fn det(&self, other: Vec3) -> f64 {
//        self.x * other.y - self.y * other.x
//    }
//
//    pub fn square(&self) -> f64 {
//        self.x * self.x + self.y * self.y
//    }
//
//    pub fn cos(&self, other: Vec3) -> f64 {
//        self.dot(other) / (self.norm() * other.norm())
//    }
//
//    pub fn dot(&self, other: Vec3) -> f64 {
//        self.x * other.x + self.y * other.y
//    }
//
//    pub fn abs(&self) -> Vec3 {
//        Vec3::new(self.x.abs(), self.y.abs())
//    }

    pub fn normalized(&self) -> Vec3 {
        *self / self.norm()
    }

//    pub fn left_orthogonal(&self) -> Vec3 {
//        Vec3 { x: -self.y, y: self.x }
//    }
//
//    pub fn increased_by(&self, value: f64) -> Vec3 {
//        let norm = self.norm();
//        *self * (norm + value) / norm
//    }
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

//impl Mul for Vec3 {
//    type Output = Vec3;
//
//    fn mul(self, rhs: Vec3) -> Vec3 {
//        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
//    }
//}

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

//impl Hash for Vec3 {
//    fn hash<H: Hasher>(&self, state: &mut H) {
//        hash_f64(self.x, state);
//        hash_f64(self.y, state);
//    }
//}
