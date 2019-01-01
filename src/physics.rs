use crate::model::Rules;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::entity::Entity;

#[derive(Debug, Clone, PartialEq)]
pub struct MoveEquation {
    pub initial_position: Vec3,
    pub initial_velocity: Vec3,
    pub acceleration: Vec3,
}

impl MoveEquation {
    pub fn from_entity(entity: &Entity, rules: &Rules) -> Self {
        MoveEquation {
            initial_position: entity.position(),
            initial_velocity: entity.velocity(),
            acceleration: rules.gravity_acceleration()
        }
    }

    pub fn get_position(&self, time: f64) -> Vec3 {
        use crate::my_strategy::common::Square;

        self.initial_position + self.initial_velocity * time + self.acceleration * time.square() / 2.0
    }

    pub fn get_velocity(&self, time: f64) -> Vec3 {
        self.initial_velocity + self.acceleration * time
    }

    pub fn get_max_y(&self) -> f64 {
        self.get_position(-self.initial_velocity.y() / self.acceleration.y()).y()
    }
}

pub fn get_min_distance_between_spheres(ball_y: f64, ball_radius: f64, robot_radius: f64) -> Option<f64> {
    use crate::my_strategy::common::Square;

    let a = (ball_radius + robot_radius).square();
    let b = (ball_y - robot_radius).square();
    if a >= b {
        Some((a - b).sqrt())
    } else {
        None
    }
}
