use crate::model::Ball;
use crate::my_strategy::vec3::Vec3;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

impl Ball {
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

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        self.render_velocity(render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_velocity(&self, render: &mut Render) {
        use crate::my_strategy::render::{Object, Color, VELOCITY_FACTOR};

        render.add(Object::line(
            self.position(),
            self.position() + self.velocity() * VELOCITY_FACTOR,
            3.0,
            Color::new(0.0, 0.6, 0.0, 1.0),
        ));
    }
}

impl PartialEq for Ball {
    fn eq(&self, other: &Ball) -> bool {
        (
            self.x,
            self.y,
            self.z,
            self.velocity_x,
            self.velocity_y,
            self.velocity_z,
            self.radius,
        ).eq(&(
            other.x,
            other.y,
            other.z,
            other.velocity_x,
            other.velocity_y,
            other.velocity_z,
            other.radius,
        ))
    }
}

impl Eq for Ball {}
