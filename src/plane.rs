use crate::my_strategy::vec3::Vec3;

pub struct Plane {
    position: Vec3,
    normal: Vec3,
}

impl Plane {
    pub const fn new(position: Vec3, normal: Vec3) -> Self {
        Plane { position, normal }
    }

    pub fn distance(&self, position: Vec3) -> f64 {
        self.normal.dot(position - self.position)
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn collide(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        let distance_to = self.distance(position);
        if *distance > distance_to {
            *distance = distance_to;
            *normal = self.normal;
        }
    }

    pub fn distance_and_normal(&self, position: Vec3) -> (f64, Vec3) {
        (self.distance(position), self.normal)
    }

    pub fn projected(value: Vec3, normal: Vec3) -> Vec3 {
        value - normal * normal.dot(value)
    }
}
