use crate::model::Rules;
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
        self.ROBOT_MIN_RADIUS + self.BALL_RADIUS
    }

    pub fn max_robot_jump_height(&self) -> f64 {
        use crate::my_strategy::common::Square;

        let time = self.ROBOT_MAX_JUMP_SPEED / self.GRAVITY;
        self.ROBOT_MAX_RADIUS
            + self.ROBOT_MAX_JUMP_SPEED * time
            - self.GRAVITY * time.square() / 2.0
    }
}
