use crate::model::Action;
use crate::my_strategy::vec3::Vec3;

impl Action {
    pub fn target_velocity(&self) -> Vec3 {
        Vec3::new(self.target_velocity_x, self.target_velocity_y, self.target_velocity_z)
    }

    pub fn set_target_velocity(&mut self, value: Vec3) {
        self.target_velocity_x = value.x();
        self.target_velocity_y = value.y();
        self.target_velocity_z = value.z();
    }
}
