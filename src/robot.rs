use crate::model::Robot;
use crate::model::Rules;
use crate::my_strategy::vec3::Vec3;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

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

    pub fn touch_normal(&self) -> Option<Vec3> {
        if self.touch {
            Some(Vec3::new(
                self.touch_normal_x.unwrap(),
                self.touch_normal_y.unwrap(),
                self.touch_normal_z.unwrap(),
            ))
        } else {
            None
        }
    }

    pub fn set_touch_normal(&mut self, value: Option<Vec3>) {
        if let Some(value) = value {
            self.touch = true;
            self.touch_normal_x = Some(value.x());
            self.touch_normal_y = Some(value.y());
            self.touch_normal_z = Some(value.z());
        } else {
            self.touch = false;
            self.touch_normal_x = None;
            self.touch_normal_y = None;
            self.touch_normal_z = None;
        }
    }

    pub fn jump(&mut self, jump_speed: f64, rules: &Rules) {
        self.radius = rules.ROBOT_MIN_RADIUS
            + (rules.ROBOT_MAX_RADIUS - rules.ROBOT_MIN_RADIUS)
            * jump_speed / rules.ROBOT_MAX_JUMP_SPEED;
    }

    pub fn opposite(&self) -> Self {
        Robot {
            id: self.id,
            player_id: self.player_id,
            is_teammate: !self.is_teammate,
            x: -self.x,
            y: self.y,
            z: -self.z,
            velocity_x: -self.velocity_x,
            velocity_y: self.velocity_y,
            velocity_z: -self.velocity_z,
            radius: self.radius,
            nitro_amount: self.nitro_amount,
            touch: self.touch,
            touch_normal_x: self.touch_normal_x.map(|v| -v),
            touch_normal_y: self.touch_normal_y,
            touch_normal_z: self.touch_normal_z.map(|v| -v),
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        self.render_velocity(render);
        self.render_text(render);
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

    #[cfg(feature = "enable_render")]
    pub fn render_text(&self, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::text(format!(
            "robot: {}\n  speed: {}\n  nitro: {}", self.id, self.velocity().norm(), self.nitro_amount
        )));
    }
}
