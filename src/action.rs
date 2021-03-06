use crate::model::Action;
use crate::my_strategy::vec3::Vec3;

#[cfg(feature = "enable_render")]
use crate::model::Robot;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

impl Action {
    pub fn target_velocity(&self) -> Vec3 {
        Vec3::new(self.target_velocity_x, self.target_velocity_y, self.target_velocity_z)
    }

    pub fn set_target_velocity(&mut self, value: Vec3) {
        self.target_velocity_x = value.x();
        self.target_velocity_y = value.y();
        self.target_velocity_z = value.z();
    }

    pub fn opposite(&self) -> Action {
        Action {
            target_velocity_x: self.target_velocity_x,
            target_velocity_y: self.target_velocity_y,
            target_velocity_z: -self.target_velocity_z,
            jump_speed: self.jump_speed,
            use_nitro: self.use_nitro,
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, robot: &Robot, render: &mut Render) {
        self.render_target_velocity(robot, render);
        self.render_jump_speed(robot, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_target_velocity(&self, robot: &Robot, render: &mut Render) {
        use crate::my_strategy::render::{Color, Object, VELOCITY_FACTOR};

        render.add(Object::line(
            robot.position(),
            robot.position() + self.target_velocity() * VELOCITY_FACTOR,
            3.0,
            Color::new(0.0, 0.0, 0.6, 1.0),
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_jump_speed(&self, robot: &Robot, render: &mut Render) {
        use crate::my_strategy::render::{Color, Object, VELOCITY_FACTOR};

        render.add(Object::line(
            robot.position(),
            robot.position() + Vec3::only_y(self.jump_speed) * VELOCITY_FACTOR,
            3.0,
            Color::new(0.6, 0.0, 0.0, 1.0),
        ));
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Action) -> bool {
        (
            self.target_velocity_x,
            self.target_velocity_y,
            self.target_velocity_z,
            self.jump_speed,
            self.use_nitro,
        ).eq(&(
            other.target_velocity_x,
            other.target_velocity_y,
            other.target_velocity_z,
            other.jump_speed,
            other.use_nitro,
        ))
    }
}

impl Eq for Action {}
