use crate::model::NitroPack;
use crate::my_strategy::vec3::Vec3;

impl NitroPack {
    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn set_position(&mut self, value: Vec3) {
        self.x = value.x();
        self.y = value.y();
        self.z = value.z();
    }

    pub fn opposite(&self) -> Self {
        NitroPack {
            id: self.id,
            x: -self.x,
            y: self.y,
            z: -self.z,
            radius: self.radius,
            nitro_amount: self.nitro_amount,
            respawn_ticks: self.respawn_ticks,
        }
    }
}
