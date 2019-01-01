use crate::model::{Ball, Rules};
use crate::my_strategy::vec3::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct MoveEquation {
    pub initial_position: Vec3,
    pub initial_velocity: Vec3,
    pub acceleration: Vec3,
}

impl MoveEquation {
    pub fn from_ball(ball: &Ball, rules: &Rules) -> Self {
        MoveEquation {
            initial_position: ball.position(),
            initial_velocity: ball.velocity(),
            acceleration: rules.gravity_acceleration()
        }
    }

    pub fn get_position(&self, time: f64) -> Vec3 {
        use crate::my_strategy::common::Square;

        self.initial_position + self.initial_velocity * time + self.acceleration * time.square() / 2.0
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
