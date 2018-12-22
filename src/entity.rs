use crate::my_strategy::vec3::Vec3;

pub trait Entity {
    fn position(&self) -> Vec3;
    fn velocity(&self) -> Vec3;
    fn set_position(&mut self, value: Vec3);
    fn set_velocity(&mut self, value: Vec3);
}
