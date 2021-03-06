use crate::model::{Rules, Robot};
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::arena::ArenaCollisionMask;
use crate::my_strategy::line2::Line2;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

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

    pub fn get_my_goal_target(&self) -> Vec3 {
        self.get_goal_target().opposite()
    }

    pub fn get_my_goalkeeper_line(&self) -> Line2 {
        Line2::new(
            Vec2::new(-self.arena.goal_width / 2.0, -self.arena.depth / 2.0 - 1.375),
            Vec2::new(self.arena.goal_width / 2.0, -self.arena.depth / 2.0 - 1.375),
        )
    }

    pub fn get_goalkeeper_position(&self, ball_position: Vec3) -> Vec3 {
        use crate::my_strategy::line2::Line2;
        use crate::my_strategy::common::Clamp;

        self.get_my_goalkeeper_line()
            .possible_intersection(&Line2::new(
                ball_position.xz(),
                self.get_my_goal_target().xz() - Vec2::new(0.0, self.arena.depth / 2.0 - self.arena.goal_depth)
            ))
            .map(|v| {
                Vec3::new(
                    v.x().clamp(-6.0, 6.0),
                    self.ROBOT_RADIUS,
                    v.y()
                )
            })
            .unwrap_or(Vec3::new(0.0, self.ROBOT_RADIUS, -self.arena.depth / 2.0))
    }

    pub fn max_robot_jump_height(&self) -> f64 {
        use crate::my_strategy::common::Square;

        let time = self.jump_to_max_height_time();
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

    pub fn get_arena_collision_mask(&self, position: &Vec3, max_path: f64) -> ArenaCollisionMask {
        if self.is_far_from_goal(position, max_path) {
            if self.is_far_from_ceiling(position, max_path) && self.is_far_from_walls(position, max_path) {
                ArenaCollisionMask::OnlyGround
            } else {
                ArenaCollisionMask::ExceptGoal
            }
        } else {
            ArenaCollisionMask::All
        }
    }

    pub fn is_far_from_goal(&self, position: &Vec3, max_path: f64) -> bool {
        use crate::my_strategy::common::IsBetween;

        position.z().is_between(
            -self.arena.depth / 2.0 + self.arena.corner_radius + max_path,
            self.arena.depth / 2.0 - self.arena.corner_radius - max_path,
        )
    }

    pub fn is_far_from_ceiling(&self, position: &Vec3, max_path: f64) -> bool {
        position.y() < self.arena.height - self.arena.top_radius - max_path
    }

    pub fn is_far_from_walls(&self, position: &Vec3, max_path: f64) -> bool {
        use crate::my_strategy::common::IsBetween;

        position.x().is_between(
            -self.arena.width / 2.0 + self.arena.bottom_radius + max_path,
            self.arena.width / 2.0 - self.arena.bottom_radius - max_path,
        )
    }

    pub fn is_near_my_goal(&self, position: Vec3) -> bool {
        use crate::my_strategy::common::IsBetween;

        position.z() < self.near_goal_max_z()
        && position.y() < self.near_goal_max_y()
        && position.x().is_between(self.near_goal_min_x(), self.near_goal_max_x())
    }

    pub fn near_goal_min_x(&self) -> f64 {
        -self.near_goal_max_x()
    }

    pub fn near_goal_max_x(&self) -> f64 {
        self.arena.goal_width / 2.0
    }

    pub fn near_goal_max_y(&self) -> f64 {
        self.arena.goal_height + 2.0 * self.BALL_RADIUS
    }

    pub fn near_goal_max_z(&self) -> f64 {
        -self.arena.depth / 2.0 + 3.0 * self.BALL_RADIUS
    }

    pub fn jump_to_max_height_time(&self) -> f64 {
        self.ROBOT_MAX_JUMP_SPEED / self.GRAVITY
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        use crate::my_strategy::render::{Color, Object};

        let color = Color::new(0.5, 0.5, 0.5, 0.8);
        let width = 3.0;

        for &y in &[self.ROBOT_RADIUS, self.near_goal_max_y()] {
            render.add(Object::line(
                Vec3::new(self.near_goal_min_x(), y, -self.arena.depth / 2.0),
                Vec3::new(self.near_goal_min_x(), y, self.near_goal_max_z()),
                width,
                color
            ));

            render.add(Object::line(
                Vec3::new(self.near_goal_min_x(), y, self.near_goal_max_z()),
                Vec3::new(self.near_goal_max_x(), y, self.near_goal_max_z()),
                width,
                color
            ));

            render.add(Object::line(
                Vec3::new(self.near_goal_max_x(), y, self.near_goal_max_z()),
                Vec3::new(self.near_goal_max_x(), y, -self.arena.depth / 2.0),
                width,
                color
            ));
        }

        for &x in &[self.near_goal_min_x(), self.near_goal_max_x()] {
            render.add(Object::line(
                Vec3::new(x, 0.0, self.near_goal_max_z()),
                Vec3::new(x, self.near_goal_max_y(), self.near_goal_max_z()),
                width,
                color
            ));
        }
    }
}

