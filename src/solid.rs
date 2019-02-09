use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::arena::ArenaCollisionMask;

pub trait Solid : Entity {
    fn radius(&self) -> f64;
    fn mass(&self) -> f64;
    fn radius_change_speed(&self) -> f64;
    fn arena_e(&self) -> f64;
    fn set_distance_to_arena(&mut self, value: f64);
    fn set_normal_to_arena(&mut self, value: Vec3);
    fn arena_collision_mask(&self) -> ArenaCollisionMask;
}

#[derive(Debug, Copy, Clone)]
pub enum SolidId {
    Ball,
    Robot(i32),
}
