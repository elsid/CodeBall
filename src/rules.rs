use crate::model::{Rules, Robot};
use crate::my_strategy::vec3::Vec3;

impl Rules {
    pub fn tick_time_interval(&self) -> f64 {
        1.0 / self.TICKS_PER_SECOND as f64
    }

    pub fn micro_tick_time_interval(&self) -> f64 {
        self.tick_time_interval() / self.MICROTICKS_PER_TICK as f64
    }

    pub fn mean_e(&self) -> f64 {
        (self.MIN_HIT_E + self.MAX_HIT_E) / 2.0
    }

    pub fn gravity_acceleration(&self) -> Vec3 {
        Vec3::new(0.0, -self.GRAVITY, 0.0)
    }

    pub fn ball_distance_limit(&self) -> f64 {
        self.ROBOT_MAX_RADIUS + self.BALL_RADIUS
    }

    pub fn get_goal_target(&self) -> Vec3 {
        Vec3::new(
            0.0,
            self.arena.goal_height - self.BALL_RADIUS,
            self.arena.depth / 2.0 + self.arena.goal_depth - self.BALL_RADIUS
        )
    }

    pub fn get_goalkeeper_position(&self) -> Vec3 {
        Vec3::new(0.0, self.ROBOT_MIN_RADIUS, -self.arena.depth / 2.0)
    }

    pub fn max_robot_jump_height(&self) -> f64 {
        use crate::my_strategy::common::Square;

        let time = self.ROBOT_MAX_JUMP_SPEED / self.GRAVITY;
        self.ROBOT_MAX_RADIUS
            + self.ROBOT_MAX_JUMP_SPEED * time
            - self.GRAVITY * time.square() / 2.0
    }

    pub fn min_acceleration_time(&self) -> f64 {
        self.ROBOT_MAX_GROUND_SPEED / self.ROBOT_ACCELERATION
    }

    pub fn min_running_distance(&self) -> f64 {
        use crate::my_strategy::common::Square;

        self.ROBOT_ACCELERATION * self.min_acceleration_time().square() / 2.0
    }

    pub fn max_robot_wall_walk_height(&self) -> f64 {
        use crate::my_strategy::common::Square;

        let time = self.ROBOT_MAX_GROUND_SPEED / self.GRAVITY;
        self.ROBOT_MAX_RADIUS
            + self.ROBOT_MAX_GROUND_SPEED * time
            - self.GRAVITY * time.square() / 2.0
    }

    pub fn acceleration_time(&self, initial_speed: f64, final_speed: f64) -> f64 {
        (final_speed - initial_speed).abs() / self.ROBOT_ACCELERATION
    }

    pub fn time_for_distance(&self, speed: f64, mut distance: f64) -> f64 {
        use crate::my_strategy::common::Square;

        let brake_time = if speed < 0.0 {
            self.acceleration_time(speed, 0.0)
        } else {
            0.0
        };

        distance += -speed * brake_time - self.ROBOT_ACCELERATION * brake_time.square() / 2.0;

        let acceleration_time = self.acceleration_time(speed, self.ROBOT_MAX_GROUND_SPEED);
        let acceleration_distance = speed * acceleration_time
            + self.ROBOT_ACCELERATION * acceleration_time.square() / 2.0;
        brake_time + if distance < acceleration_distance {
            let speed_change = self.ROBOT_MAX_GROUND_SPEED - speed;
            let final_speed = (
                (2.0 * speed_change * acceleration_time * distance + acceleration_time.square() * speed.square()).sqrt()
                    - acceleration_time * speed
            ) / acceleration_time + speed;
            (final_speed - speed) * acceleration_time / speed_change
        } else if distance > acceleration_distance {
            acceleration_time + (distance - acceleration_distance) / self.ROBOT_MAX_GROUND_SPEED
        } else {
            acceleration_time
        }
    }

    pub fn get_approximate_robot_radius_change_speed(&self, radius: f64) -> f64 {
        self.ROBOT_MAX_JUMP_SPEED * (radius - self.ROBOT_MIN_RADIUS) / self.robot_radius_max_change()
    }

    pub fn robot_radius_max_change(&self) -> f64 {
        self.ROBOT_MAX_RADIUS - self.ROBOT_MIN_RADIUS
    }

    pub fn is_flying(&self, robot: &Robot) -> bool {
        use crate::my_strategy::physics::MoveEquation;

        (
            robot.velocity_y > 0.0
            && self.arena.distance(robot.position()) - robot.radius > 1e-3
        ) || self.arena.distance(
            MoveEquation::from_robot(robot, self)
                .get_position(self.tick_time_interval())
        ) - robot.radius > 1e-3
    }
}
