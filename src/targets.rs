use crate::model::Robot;
use crate::my_strategy::world::World;
use crate::my_strategy::roles::Role;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::{Color, Render};

#[cfg(feature = "enable_render")]
use crate::my_strategy::vec3::Vec3;

#[derive(Debug)]
pub enum Target {
    None,
    Ball,
    NitroPack(i32),
    GoalkeeperPosition,
    Robot(i32),
}

impl Target {
    pub fn generate(world: &World) -> Vec<Target> {
        let mut result = Vec::new();
        result.push(Target::Ball);
        world.game.nitro_packs.iter()
            .for_each(|v| result.push(Target::NitroPack(v.id)));
        world.game.robots.iter()
            .filter(|v| !v.is_teammate)
            .for_each(|v| result.push(Target::Robot(v.id)));
        result.push(Target::GoalkeeperPosition);
        for _ in 0..world.game.robots.len() {
            result.push(Target::None);
        }
        result
    }

    pub fn get_priority(&self, world: &World) -> i32 {
        use crate::my_strategy::common::as_score;

        let result = match self {
            Target::Ball => 1.0,
            Target::NitroPack(id) => 0.75 * Self::get_nitro_pack_priority(world, *id),
            Target::Robot(id) => 0.5 * Self::get_robot_priority(world, *id),
            Target::GoalkeeperPosition => 0.25,
            Target::None => 0.0,
        };

        -as_score(result)
    }

    pub fn get_nitro_pack_priority(world: &World, id: i32) -> f64 {
        let nitro_pack = world.get_nitro_pack(id);
        let respawn_ticks_priority = if let Some(respawn_ticks) = nitro_pack.respawn_ticks {
            1.0 - respawn_ticks as f64 / world.rules.NITRO_PACK_RESPAWN_TICKS as f64
        } else {
            1.0
        };
        let opponent_distance_priority = world.game.robots.iter()
            .find(|v| {
                !v.is_teammate
                    && v.position().distance(nitro_pack.position())
                        < world.rules.NITRO_PACK_RADIUS + world.rules.ROBOT_MIN_RADIUS
            })
            .map(|_| 0.0)
            .unwrap_or(1.0);

        (respawn_ticks_priority + opponent_distance_priority) / 2.0
    }

    pub fn get_robot_priority(world: &World, id: i32) -> f64 {
        let robot = world.get_robot(id);
        let distance_to_my_goal_priority = 1.0 - robot.position()
            .distance(world.rules.get_my_goal_target()) / world.rules.arena.max_distance();

        distance_to_my_goal_priority
    }

    pub fn get_score(&self, role: &Role, robot: &Robot, world: &World) -> i32 {
        use crate::my_strategy::common::as_score;

        let result = match self {
            Target::None => 1.0,
            Target::Ball => Self::get_ball_score(role, robot, world),
            Target::NitroPack(id) => Self::get_nitro_pack_score(role, robot, world, *id),
            Target::Robot(id) => Self::get_robot_score(role, robot, world, *id),
            Target::GoalkeeperPosition => Self::get_goalkeeper_position_score(role),
        };

        as_score(result)
    }

    fn get_ball_score(role: &Role, robot: &Robot, world: &World) -> f64 {
        use crate::my_strategy::physics::MoveEquation;
        use crate::my_strategy::entity::Entity;

        let z_score = match role {
            Role::Goalkeeper => -world.game.ball.z / world.rules.arena.max_z() / 2.0,
            Role::Forward => world.game.ball.z / world.rules.arena.max_z() / 2.0,
        };
        let distance_score = 1.0 - robot.position().distance(world.game.ball.position())
            / world.rules.arena.max_distance();
        let move_equation = MoveEquation::from_entity(&world.game.ball, &world.rules);
        let closest_distance_score = 1.0 - move_equation.get_closest_possible_distance_to_target(
            robot.position(),
            world.rules.BALL_RADIUS,
            100.0 * world.rules.tick_time_interval(),
            10,
        ) / world.rules.arena.max_distance();

        (z_score + distance_score + closest_distance_score) / 3.0
    }

    fn get_nitro_pack_score(role: &Role, robot: &Robot, world: &World, id: i32) -> f64 {
        let nitro_pack = world.get_nitro_pack(id);
        let role_factor = match role {
            Role::Goalkeeper => -world.game.ball.z / world.rules.arena.max_z() / 2.0,
            Role::Forward => 1.0,
        };
        let distance = robot.position().distance(nitro_pack.position());
        let distance_score = 1.0 - distance / world.rules.arena.max_distance();
        let respawn_score = if let Some(respawn_ticks) = nitro_pack.respawn_ticks {
            if (distance / respawn_ticks as f64) < world.rules.ROBOT_MAX_GROUND_SPEED {
                0.0
            } else {
                0.0
            }
        } else {
            1.0
        };
        let current_nitro_amount_score = robot.nitro_amount as f64
            / world.rules.NITRO_PACK_AMOUNT;

        role_factor * (
            0.0
                + distance_score
                + current_nitro_amount_score
                + respawn_score
        ) / 3.0
    }

    fn get_robot_score(role: &Role, robot: &Robot, world: &World, id: i32) -> f64 {
        let robot = world.get_robot(id);
        let role_factor = match role {
            Role::Goalkeeper => 0.0,
            Role::Forward => 1.0,
        };
        let distance = robot.position().distance(robot.position());
        let distance_score = 1.0 - distance / world.rules.arena.max_distance();

        role_factor * distance_score
    }

    fn get_goalkeeper_position_score(role: &Role) -> f64 {
        match role {
            Role::Goalkeeper => 1.0,
            Role::Forward => 0.0,
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, robot: &Robot, world: &World, render: &mut Render) {
        self.render_text(render);
        self.render_pointer(robot, world, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_text(&self, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::text(format!("  target: {:?}", self)));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_pointer(&self, robot: &Robot, world: &World, render: &mut Render) {
        use crate::my_strategy::render::Object;

        let begin = robot.position() + Vec3::only_y(2.0 * robot.radius);

        render.add(Object::line(
            begin,
            self.get_position(begin, world),
            3.0,
            Color::new(0.5, 0.5, 0.5, 1.0),
        ));
    }

    #[cfg(feature = "enable_render")]
    fn get_position(&self, default: Vec3, world: &World) -> Vec3 {
        match self {
            Target::None => default,
            Target::Ball => world.game.ball.position(),
            Target::NitroPack(id) => world.get_nitro_pack(*id).position(),
            Target::Robot(id) => world.get_robot(*id).position(),
            Target::GoalkeeperPosition => world.rules.get_goalkeeper_position(),
        }
    }
}
