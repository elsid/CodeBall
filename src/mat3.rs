use std::ops::Mul;
use crate::my_strategy::vec3::Vec3;

#[derive(Default, Copy, Clone, Debug)]
pub struct Mat3 {
    x: Vec3,
    y: Vec3,
    z: Vec3,
}

impl Mat3 {
    pub const fn new(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Mat3 { x, y, z }
    }

    pub const fn identity() -> Self {
        Mat3::new(Vec3::i(), Vec3::j(), Vec3::k())
    }

    pub fn rotation(axis: Vec3, angle: f64) -> Self {
        if angle == 0.0 {
            Self::identity()
        } else {
            let ux = axis.x();
            let uy = axis.y();
            let uz = axis.z();
            let sqx = ux * ux;
            let sqy = uy * uy;
            let sqz = uz * uz;
            let (sin, cos) = (-angle).sin_cos();
            let one_minus_cos = 1.0 - cos;

            Mat3::new(
                Vec3::new(
                    sqx + (1.0 - sqx) * cos,
                    ux * uy * one_minus_cos - uz * sin,
                    ux * uz * one_minus_cos + uy * sin,
                ),
                Vec3::new(
                    ux * uy * one_minus_cos + uz * sin,
                    sqy + (1.0 - sqy) * cos,
                    uy * uz * one_minus_cos - ux * sin,
                ),
                Vec3::new(
                    ux * uz * one_minus_cos - uy * sin,
                    uy * uz * one_minus_cos + ux * sin,
                    sqz + (1.0 - sqz) * cos,
                )
            )
        }
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x.dot(rhs), self.y.dot(rhs), self.z.dot(rhs))
    }
}
