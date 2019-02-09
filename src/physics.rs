use crate::model::{Rules, Robot, Ball};
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::solid::Solid;

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

    pub fn from_robot(robot: &Robot, rules: &Rules) -> Self {
        MoveEquation {
            initial_position: robot.position(),
            initial_velocity: robot.velocity(),
            acceleration: rules.gravity_acceleration()
        }
    }

    pub fn from_robot_with_nitro(robot: &Robot, rules: &Rules) -> Self {
        let nitro_acceleration = if robot.nitro_amount > 0.0 && robot.velocity().norm() > 0.0 {
            robot.velocity().normalized() * rules.ROBOT_NITRO_ACCELERATION
        } else {
            Vec3::default()
        };
        MoveEquation {
            initial_position: robot.position(),
            initial_velocity: robot.velocity(),
            acceleration: rules.gravity_acceleration() + nitro_acceleration
        }
    }

    pub fn from_solid(solid: &Solid, rules: &Rules) -> Self {
        MoveEquation {
            initial_position: solid.position(),
            initial_velocity: solid.velocity(),
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
