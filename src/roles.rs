use crate::model::Robot;
use crate::my_strategy::world::World;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::{Color, Render};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Role {
    Forward,
    Goalkeeper,
}

impl Role {
    pub fn forward() -> Self {
        Role::Forward
    }

    pub fn goalkeeper() -> Self {
        Role::Goalkeeper
    }

    pub fn get_score(&self, robot: &Robot, world: &World) -> i32 {
        use crate::my_strategy::common::as_score;

        let result = match self {
            Role::Forward => Self::get_forward_score(robot, world),
            Role::Goalkeeper => Self::get_goalkeeper_score(robot, world),
        };

        as_score(result)
    }

    fn get_forward_score(robot: &Robot, world: &World) -> f64 {
        let distance_to_goal_score = 1.0 - world.rules.get_goal_target()
            .distance(robot.position()) / world.rules.arena.max_distance();

        let distance_to_ball_score = 1.0 - world.game.ball.position()
            .distance(robot.position()) / world.rules.arena.max_distance();

        (
            0.0
                + distance_to_goal_score
                + distance_to_ball_score
        ) / 2.0
    }

    pub fn get_goalkeeper_score(me: &Robot, world: &World) -> f64 {
        let distance_to_my_goal_score = 1.0 - world.rules.get_my_goal_target()
            .distance(me.position()) / world.rules.arena.max_distance();

        distance_to_my_goal_score
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, robot: &Robot, render: &mut Render) {
        self.render_text(render);
        self.render_marker(robot, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_marker(&self, robot: &Robot, render: &mut Render) {
        use crate::my_strategy::render::Object;
        use crate::my_strategy::vec3::Vec3;

        render.add(Object::sphere(
            robot.position() + Vec3::only_y(2.0 * robot.radius),
            robot.radius / 2.0,
            self.get_color(),
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_text(&self, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::text(format!("  role: {:?}", self)));
    }

    #[cfg(feature = "enable_render")]
    fn get_color(&self) -> Color {
        match self {
            Role::Forward => Color::new(0.8, 0.0, 0.0, 0.6),
            Role::Goalkeeper => Color::new(0.0, 0.8, 0.0, 0.6),
        }
    }
}

