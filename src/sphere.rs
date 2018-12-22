use crate::my_strategy::vec3::Vec3;

pub struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f64) -> Self {
        Sphere {center, radius}
    }

    pub fn distance_inner(&self, position: Vec3) -> f64 {
        self.radius - (self.center - position).norm()
    }

    pub fn distance_outer(&self, position: Vec3) -> f64 {
        (position - self.center).norm() - self.radius
    }

    pub fn inner_normal_to(&self, position: Vec3) -> Vec3 {
        (self.center - position).normalized()
    }

    pub fn outer_normal_to(&self, position: Vec3) -> Vec3 {
        (position - self.center).normalized()
    }

    pub fn inner_collide(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        let distance_to = self.distance_inner(position);
        if *distance > distance_to {
            *distance = distance_to;
            *normal = self.inner_normal_to(position);
        }
    }

    pub fn outer_collide(&self, position: Vec3, distance: &mut f64, normal: &mut Vec3) {
        let distance_to = self.distance_outer(position);
        if *distance > distance_to {
            *distance = distance_to;
            *normal = self.outer_normal_to(position);
        }
    }
}
