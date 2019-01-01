use crate::model::Robot;
use crate::model::Rules;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::common::Clamp;

impl Robot {
    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn velocity(&self) -> Vec3 {
        Vec3::new(self.velocity_x, self.velocity_y, self.velocity_z)
    }

    pub fn set_position(&mut self, value: Vec3) {
        self.x = value.x();
        self.y = value.y();
        self.z = value.z();
    }

    pub fn set_velocity(&mut self, value: Vec3) {
        self.velocity_x = value.x();
        self.velocity_y = value.y();
        self.velocity_z = value.z();
    }

    pub fn jump(&mut self, jump_speed: f64, rules: &Rules) {
        self.radius = rules.ROBOT_MIN_RADIUS
            + (rules.ROBOT_MAX_RADIUS - rules.ROBOT_MIN_RADIUS)
            * jump_speed.clamp(0.0, rules.ROBOT_MAX_JUMP_SPEED) / rules.ROBOT_MAX_JUMP_SPEED;
    }
}
