use crate::model::Robot;
use crate::my_strategy::world::World;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::{Color, Render};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Role {
    Forward(Forward),
    Goalkeeper(Goalkeeper),
}

impl Role {
    pub fn forward(robot_id: i32) -> Self {
        Role::Forward(Forward { robot_id })
    }

    pub fn goalkeeper(robot_id: i32) -> Self {
        Role::Goalkeeper(Goalkeeper { robot_id })
    }

    pub fn get_score(&self, world: &World) -> i32 {
        match self {
            Role::Forward(v) => Forward::get_score(world.get_robot(v.robot_id), world),
            Role::Goalkeeper(v) => Goalkeeper::get_score(world.get_robot(v.robot_id), world),
        }
    }

    pub fn robot_id(&self) -> i32 {
        match self {
            Role::Forward(v) => v.robot_id,
            Role::Goalkeeper(v) => v.robot_id,
        }
    }

    pub fn max_z(&self, world: &World) -> f64 {
        match self {
            Role::Forward(_) => Forward::max_z(),
            Role::Goalkeeper(_) => Goalkeeper::max_z(world),
        }
    }

    pub fn can_quit(&self, world: &World) -> bool {
        match self {
            Role::Forward(_) => Forward::can_quit(),
            Role::Goalkeeper(v) => v.can_quit(world),
        }
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

        render.add(Object::text(format!("  role: {}", self.name())));
    }

    #[cfg(feature = "enable_render")]
    fn get_color(&self) -> Color {
        match self {
            Role::Forward(_) => Forward::get_color(),
            Role::Goalkeeper(_) => Goalkeeper::get_color(),
        }
    }

    #[cfg(feature = "enable_render")]
    fn name(&self) -> &'static str {
        match self {
            Role::Forward(_) => "forward",
            Role::Goalkeeper(_) => "goalkeeper",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Forward {
    pub robot_id: i32,
}

impl Forward {
    pub fn get_score(robot: &Robot, world: &World) -> i32 {
        use crate::my_strategy::common::as_score;

        let distance_to_goal_score = 1.0 - world.rules.get_goal_target()
            .distance(robot.position()) / world.rules.arena.max_distance();

        let distance_to_ball_score = 1.0 - world.game.ball.position()
            .distance(robot.position()) / world.rules.arena.max_distance();

        as_score((
            0.0
                + distance_to_goal_score
                + distance_to_ball_score
        ) / 2.0)
    }

    pub fn max_z() -> f64 {
        std::f64::MAX
    }

    pub fn can_quit() -> bool {
        true
    }

    #[cfg(feature = "enable_render")]
    pub fn get_color() -> Color {
        Color::new(0.8, 0.1, 0.1, 0.8)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Goalkeeper {
    pub robot_id: i32,
}

impl Goalkeeper {
    pub fn get_score(robot: &Robot, world: &World) -> i32 {
        use crate::my_strategy::common::as_score;

        let distance_to_my_goal_score = 1.0 - world.rules.get_goal_target().opposite()
            .distance(robot.position()) / world.rules.arena.max_distance();

        as_score(distance_to_my_goal_score)
    }

    pub fn max_z(world: &World) -> f64 {
        (-world.rules.arena.depth / 2.0 + world.rules.arena.corner_radius + world.rules.BALL_RADIUS + 10.0)
            .max(-world.rules.NITRO_PACK_Z)
    }

    pub fn can_quit(&self, world: &World) -> bool {
        let robot = world.get_robot(self.robot_id);
        world.game.robots.iter()
            .find(|v| {
                self.robot_id != v.id
                    && v.is_teammate
                    && v.z < Self::max_z(world)
                    && v.z < robot.z
            })
            .is_some()
    }

    #[cfg(feature = "enable_render")]
    pub fn get_color() -> Color {
        Color::new(0.1, 0.8, 0.1, 0.8)
    }
}
