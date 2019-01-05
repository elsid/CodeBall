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

    pub fn get_time_at_y(&self, y: f64) -> Vec<f64> {
        use crate::my_strategy::common::Square;

        if self.acceleration.y() == 0.0 {
            if self.initial_velocity.y() == 0.0 {
                if self.initial_position.y() == y {
                    vec![0.0]
                } else {
                    Vec::new()
                }
            } else {
                vec![(y - self.initial_position.y()) / self.initial_velocity.y()]
            }
        } else {
            let a = 2.0 * self.acceleration.y() * (y - self.initial_position.y())
                + self.initial_velocity.y().square();
            if a < 0.0 {
                Vec::new()
            } else if a == 0.0 {
                vec![self.initial_velocity.y() / self.acceleration.y()]
            } else {
                let d = a.sqrt();
                let first = (d - self.initial_velocity.y()) / self.acceleration.y();
                let second = (d + self.initial_velocity.y()) / self.acceleration.y();
                if first < 0.0 && 0.0 <= second {
                    vec![second]
                } else if second < 0.0 && 0.0 <= first {
                    vec![first]
                } else if 0.0 <= first && 0.0 <= second {
                    vec![first, second]
                } else {
                    let time_to_stop = -self.initial_velocity.y() / self.acceleration.y();
                    if time_to_stop < 0.0 {
                        Vec::new()
                    } else {
                        let stopped_y = self.get_position(time_to_stop).y();
                        vec![
                            time_to_stop
                                - (2.0 * self.acceleration.y() * (y - stopped_y)).sqrt()
                                    / self.acceleration.y()
                        ]
                    }
                }
            }
        }
    }

    pub fn get_time_to_target(&self, target: Vec3, min_y: f64, max_time: f64, min_distance: f64, iterations: usize) -> f64 {
        use crate::my_strategy::optimization::minimize1d;

        let get_distance_to_target_penalty = |time: f64| {
            let position = self.get_position(time);
            (position.with_max_y(min_y).distance(target) - min_distance).abs()
                + min_y - position.y().min(min_y)
        };

        minimize1d(0.0, max_time, iterations, get_distance_to_target_penalty)
    }

    pub fn get_closest_possible_distance_to_target(&self, target: Vec3, min_y: f64, max_time: f64,
                                                   iterations: usize) -> f64 {
        self.get_position(
            self.get_time_to_target(target, min_y, max_time, 0.0, iterations)
        ).distance(target)
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
