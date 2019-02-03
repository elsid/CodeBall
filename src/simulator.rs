use crate::model::{Action, Ball, Robot, Rules, NitroPack, Game, Player};
use crate::my_strategy::common::Square;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::random::{Rng, XorShiftRng};
use crate::my_strategy::world::World;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::arena::ArenaCollisionMask;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::{Render, Color};

trait Shiftable : Entity {
    fn shift(&mut self, time_interval: f64, gravity: f64, max_entity_speed: f64) {
        let clamped_velocity = self.velocity().clamp(max_entity_speed);
        let next_position = self.position() + clamped_velocity * time_interval
            - Vec3::default().with_y(gravity * time_interval.square() / 2.0);
        let next_velocity = clamped_velocity
            - Vec3::default().with_y(gravity * time_interval);
        self.set_position(next_position);
        self.set_velocity(next_velocity);
    }
}

pub trait Solid : Entity {
    fn radius(&self) -> f64;
    fn mass(&self) -> f64;
    fn radius_change_speed(&self) -> f64;
    fn arena_e(&self) -> f64;
    fn set_distance_to_arena(&mut self, value: f64);
    fn set_normal_to_arena(&mut self, value: Vec3);
    fn arena_collision_mask(&self) -> ArenaCollisionMask;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum RobotCollisionType {
    None,
    TouchBall,
    KickBall,
}

impl RobotCollisionType {
    pub fn with(self, other: Self) -> Self {
        if self == other || other == RobotCollisionType::None {
            self
        } else if self == RobotCollisionType::None {
            other
        } else {
            RobotCollisionType::KickBall
        }
    }
}

#[derive(Clone, Debug)]
pub struct RobotExt {
    base: Robot,
    radius_change_speed: f64,
    action: Action,
    mass: f64,
    arena_e: f64,
    is_me: bool,
    collision_type: RobotCollisionType,
    distance_to_arena: f64,
    normal_to_arena: Vec3,
    ignore: bool,
    arena_collision_mask: ArenaCollisionMask,
}

impl RobotExt {
    pub fn from_robot(robot: &Robot, rules: &Rules) -> Self {
        RobotExt {
            base: robot.clone(),
            radius_change_speed: 0.0,
            action: Action::default(),
            mass: rules.ROBOT_MASS,
            arena_e: rules.ROBOT_ARENA_E,
            is_me: false,
            collision_type: RobotCollisionType::None,
            distance_to_arena: 0.0,
            normal_to_arena: Vec3::default(),
            ignore: false,
            arena_collision_mask: ArenaCollisionMask::All,
        }
    }

    pub fn opposite(&self) -> Self {
        RobotExt {
            base: self.base.opposite(),
            radius_change_speed: self.radius_change_speed,
            action: self.action.opposite(),
            mass: self.mass,
            arena_e: self.arena_e,
            is_me: self.is_me,
            collision_type: self.collision_type,
            distance_to_arena: self.distance_to_arena,
            normal_to_arena: self.normal_to_arena.opposite(),
            ignore: self.ignore,
            arena_collision_mask: self.arena_collision_mask,
        }
    }

    pub fn id(&self) -> i32 {
        self.base.id
    }

    pub fn player_id(&self) -> i32 {
        self.base.player_id
    }

    pub fn base(&self) -> &Robot {
        &self.base
    }

    pub fn touch_normal(&self) -> Option<Vec3> {
        self.base.touch_normal()
    }

    pub fn set_touch_normal(&mut self, value: Option<Vec3>) {
        self.base.set_touch_normal(value)
    }

    pub fn is_me(&self) -> bool {
        self.is_me
    }

    pub fn is_teammate(&self) -> bool {
        self.base.is_teammate
    }

    pub fn collision_type(&self) -> RobotCollisionType {
        self.collision_type
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn action_mut(&mut self) -> &mut Action {
        &mut self.action
    }

    pub fn jump(&mut self, jump_speed: f64, rules: &Rules) {
        self.base.jump(jump_speed, rules);
        self.radius_change_speed = jump_speed;
    }

    pub fn distance_to_arena(&self) -> f64 {
        self.distance_to_arena
    }

    pub fn normal_to_arena(&self) -> Vec3 {
        self.normal_to_arena
    }

    pub fn nitro_amount(&self) -> f64 {
        self.base.nitro_amount
    }

    pub fn set_nitro_amount(&mut self, value: f64) {
        self.base.nitro_amount = value;
    }

    pub fn ignore(&self) -> bool {
        self.ignore
    }

    pub fn set_ignore(&mut self, value: bool) {
        self.ignore = value;
    }

    pub fn arena_collision_mask(&self) -> ArenaCollisionMask {
        self.arena_collision_mask
    }

    pub fn set_arena_collision_mask(&mut self, value: ArenaCollisionMask) {
        self.arena_collision_mask = value;
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, relative_time: f64, relative_number: f64, render: &mut Render) {
        self.render_base(render);
        self.render_position(relative_time, relative_number, render);
        self.render_action(render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_position(&self, relative_time: f64, relative_number: f64, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::sphere(
            self.position(),
            self.radius(),
            Self::get_color(relative_number, relative_time)
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_action(&self, render: &mut Render) {
        self.action().render(self.base(), render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_base(&self, render: &mut Render) {
        self.base().render_velocity(render);
    }

    #[cfg(feature = "enable_render")]
    pub fn get_color(relative_number: f64, relative_time: f64) -> Color {
        Color::new(0.8, 0.2 + relative_number * 0.6, 0.2 + relative_time * 0.6, 0.4)
    }
}

impl Entity for RobotExt {
    fn position(&self) -> Vec3 {
        self.base.position()
    }

    fn velocity(&self) -> Vec3 {
        self.base.velocity()
    }

    fn set_position(&mut self, value: Vec3) {
        self.base.set_position(value)
    }

    fn set_velocity(&mut self, value: Vec3) {
        self.base.set_velocity(value)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BallCollisionType {
    None,
    Arena,
    Robot,
    ArenaAndRobot,
}

impl BallCollisionType {
    pub fn with(self, other: Self) -> Self {
        if self == other || other == BallCollisionType::None {
            self
        } else if self == BallCollisionType::None {
            other
        } else {
            BallCollisionType::ArenaAndRobot
        }
    }
}

#[derive(Clone)]
pub struct BallExt {
    base: Ball,
    mass: f64,
    arena_e: f64,
    distance_to_arena: f64,
    normal_to_arena: Vec3,
    collision_type: BallCollisionType,
    arena_collision_mask: ArenaCollisionMask,
}

impl BallExt {
    pub fn new(base: Ball, mass: f64, arena_e: f64) -> Self {
        BallExt {
            base,
            mass,
            arena_e,
            distance_to_arena: 0.0,
            normal_to_arena: Vec3::default(),
            collision_type: BallCollisionType::None,
            arena_collision_mask: ArenaCollisionMask::All,
        }
    }

    pub fn from_ball(ball: &Ball, rules: &Rules) -> Self {
        BallExt {
            base: ball.clone(),
            mass: rules.BALL_MASS,
            arena_e: rules.BALL_ARENA_E,
            distance_to_arena: 0.0,
            normal_to_arena: Vec3::default(),
            collision_type: BallCollisionType::None,
            arena_collision_mask: ArenaCollisionMask::All,
        }
    }

    pub fn opposite(&self) -> Self {
        BallExt {
            base: self.base.opposite(),
            mass: self.mass,
            arena_e: self.arena_e,
            distance_to_arena: self.distance_to_arena,
            normal_to_arena: self.normal_to_arena.opposite(),
            collision_type: self.collision_type,
            arena_collision_mask: self.arena_collision_mask,
        }
    }

    pub fn base(&self) -> &Ball {
        &self.base
    }

    pub fn distance_to_arena(&self) -> f64 {
        self.distance_to_arena
    }

    pub fn normal_to_arena(&self) -> Vec3 {
        self.normal_to_arena
    }

    pub fn collision_type(&self) -> BallCollisionType {
        self.collision_type
    }

    pub fn projected_to_arena_position_with_shift(&self, shift: f64) -> Vec3 {
        self.base().position() - self.normal_to_arena * (self.distance_to_arena - shift)
    }

    pub fn arena_collision_mask(&self) -> ArenaCollisionMask {
        self.arena_collision_mask
    }

    pub fn set_arena_collision_mask(&mut self, value: ArenaCollisionMask) {
        self.arena_collision_mask = value;
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, relative_time: f64, render: &mut Render) {
        self.render_base(render);
        self.render_position(relative_time, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_position(&self, relative_time: f64, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::sphere(
            self.position(),
            self.radius(),
            Self::get_color(relative_time)
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_base(&self, render: &mut Render) {
        self.base().render_velocity(render);
    }

    #[cfg(feature = "enable_render")]
    pub fn get_color(relative_time: f64) -> Color {
        Color::new(0.0, 0.2 + relative_time * 0.6, 0.8, 0.4)
    }
}

impl Entity for BallExt {
    fn position(&self) -> Vec3 {
        self.base.position()
    }

    fn velocity(&self) -> Vec3 {
        self.base.velocity()
    }

    fn set_position(&mut self, value: Vec3) {
        self.base.set_position(value)
    }

    fn set_velocity(&mut self, value: Vec3) {
        self.base.set_velocity(value)
    }
}

impl Shiftable for BallExt {}

impl Shiftable for RobotExt {}

impl Solid for BallExt {
    fn radius(&self) -> f64 {
        self.base.radius
    }

    fn mass(&self) -> f64 {
        self.mass
    }

    fn radius_change_speed(&self) -> f64 {
        0.0
    }

    fn arena_e(&self) -> f64 {
        self.arena_e
    }

    fn set_distance_to_arena(&mut self, value: f64) {
        self.distance_to_arena = value;
    }

    fn set_normal_to_arena(&mut self, value: Vec3) {
        self.normal_to_arena = value;
    }

    fn arena_collision_mask(&self) -> ArenaCollisionMask {
        self.arena_collision_mask
    }
}

impl Solid for RobotExt {
    fn radius(&self) -> f64 {
        self.base.radius
    }

    fn mass(&self) -> f64 {
        self.mass
    }

    fn radius_change_speed(&self) -> f64 {
        self.radius_change_speed
    }

    fn arena_e(&self) -> f64 {
        self.arena_e
    }

    fn set_distance_to_arena(&mut self, value: f64) {
        self.distance_to_arena = value;
    }

    fn set_normal_to_arena(&mut self, value: Vec3) {
        self.normal_to_arena = value;
    }

    fn arena_collision_mask(&self) -> ArenaCollisionMask {
        self.arena_collision_mask
    }
}

#[derive(Clone)]
pub struct Simulator {
    players: Vec<Player>,
    robots: Vec<RobotExt>,
    ball: BallExt,
    nitro_packs: Vec<NitroPack>,
    rules: Rules,
    current_tick: i32,
    current_micro_tick: i32,
    current_time: f64,
    score: i32,
    me_index: usize,
}

impl Simulator {
    pub fn new(world: &World, me_id: i32) -> Self {
        let robots: Vec<RobotExt> = world.game.robots.iter()
            .map(|robot| {
                let (radius_change_speed, touch_normal) = if world.me.id == robot.id {
                    (
                        world.rules.get_approximate_robot_radius_change_speed(robot.radius),
                        if robot.touch {
                            Some(Vec3::new(
                            robot.touch_normal_x.unwrap(),
                            robot.touch_normal_y.unwrap(),
                            robot.touch_normal_z.unwrap()
                            ))
                        } else {
                            None
                        }
                    )
                } else {
                    (
                        world.rules.get_approximate_robot_radius_change_speed(robot.radius),
                        world.rules.arena.get_approximate_touch_normal(robot)
                    )
                };
                let (distance, normal) = world.rules.arena
                    .distance_and_normal(robot.position());
                let mut base = robot.clone();
                base.set_touch_normal(touch_normal);
                RobotExt {
                    base: robot.clone(),
                    radius_change_speed,
                    action: Action::default(),
                    mass: world.rules.ROBOT_MASS,
                    arena_e: world.rules.ROBOT_ARENA_E,
                    is_me: robot.id == me_id,
                    collision_type: RobotCollisionType::None,
                    distance_to_arena: distance,
                    normal_to_arena: normal,
                    ignore: false,
                    arena_collision_mask: ArenaCollisionMask::All,
                }
            })
            .collect();
        let me_index = robots.iter()
            .position(|v| v.is_me)
            .unwrap();
        let (distance, normal) = world.rules.arena
            .distance_and_normal(world.game.ball.position());

        Simulator {
            players: world.game.players.clone(),
            robots,
            ball: BallExt {
                base: world.game.ball.clone(),
                mass: world.rules.BALL_MASS,
                arena_e: world.rules.BALL_ARENA_E,
                distance_to_arena: distance,
                normal_to_arena: normal,
                collision_type: BallCollisionType::None,
                arena_collision_mask: ArenaCollisionMask::All,
            },
            nitro_packs: world.game.nitro_packs.clone(),
            rules: world.rules.clone(),
            current_tick: 0,
            current_micro_tick: 0,
            current_time: 0.0,
            score: 0,
            me_index,
        }
    }

    pub fn opposite(self) -> Simulator {
        Simulator {
            players: self.players.into_iter().map(|v| v.opposite()).collect(),
            robots: self.robots.into_iter().map(|v| v.opposite()).collect(),
            ball: self.ball.opposite(),
            nitro_packs: self.nitro_packs.into_iter().map(|v| v.opposite()).collect(),
            rules: self.rules,
            current_tick: self.current_tick,
            current_micro_tick: self.current_micro_tick,
            current_time: self.current_time,
            score: self.score,
            me_index: self.me_index,
        }
    }

    pub fn robots(&self) -> &Vec<RobotExt> {
        &self.robots
    }

    pub fn robots_mut(&mut self) -> &mut Vec<RobotExt> {
        &mut self.robots
    }

    pub fn ball(&self) -> &BallExt {
        &self.ball
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn nitro_packs(&self) -> &Vec<NitroPack> {
        &self.nitro_packs
    }

    pub fn nitro_packs_mut(&mut self) -> &mut Vec<NitroPack> {
        &mut self.nitro_packs
    }

    pub fn game(&self) -> Game {
        Game {
            current_tick: self.current_tick,
            players: self.players.clone(),
            robots: self.robots.iter().map(|v| v.base().clone()).collect(),
            nitro_packs: self.nitro_packs.clone(),
            ball: self.ball.base().clone(),
        }
    }

    pub fn current_tick(&self) -> i32 {
        self.current_tick
    }

    pub fn current_micro_tick(&self) -> i32 {
        self.current_micro_tick
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn me(&self) -> &RobotExt {
        &self.robots[self.me_index]
    }

    pub fn me_mut(&mut self) -> &mut RobotExt {
        &mut self.robots[self.me_index]
    }

    pub fn ignore_me(&mut self) -> bool {
        self.me().ignore
    }

    pub fn set_ignore_me(&mut self, value: bool) {
        self.me_mut().ignore = value;
    }

    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize, rng: &mut XorShiftRng) {
        let micro_tick_time_interval = time_interval / micro_ticks_per_tick as f64;
        let max_path = time_interval * self.rules.MAX_ENTITY_SPEED;
        for robot in self.robots.iter_mut() {
            robot.collision_type = RobotCollisionType::None;
            robot.set_arena_collision_mask(self.rules.get_arena_collision_mask(
                &robot.position(), max_path + robot.radius()
            ));
        }
        self.ball.collision_type = BallCollisionType::None;
        self.ball.set_arena_collision_mask(self.rules.get_arena_collision_mask(
            &self.ball.position(), max_path + self.ball.radius()
        ));
        for _ in 0..micro_ticks_per_tick {
            self.micro_tick(micro_tick_time_interval, rng);
        }
        for nitro_pack in self.nitro_packs.iter_mut() {
            nitro_pack.respawn_ticks = if let Some(v) = nitro_pack.respawn_ticks {
                if v > 1 {
                    Some(v - 1)
                } else {
                    None
                }
            } else {
                None
            };
        }
        self.current_tick += 1;
        self.current_time += time_interval;
        self.me_index = self.robots.iter().position(|v| v.is_me).unwrap();
    }

    fn micro_tick(&mut self, time_interval: f64, rng: &mut XorShiftRng) {
        use crate::my_strategy::plane::Plane;

        rng.shuffle(&mut self.robots[..]);

        let min_e = self.rules.MIN_HIT_E;
        let max_e = self.rules.MAX_HIT_E;

        for robot in self.robots.iter_mut() {
            if robot.ignore {
                continue;
            }
            if let Some(touch_normal) = robot.touch_normal() {
                let target_velocity = robot.action.target_velocity()
                    .clamp(self.rules.ROBOT_MAX_GROUND_SPEED);
                let velocity = Plane::projected(target_velocity, touch_normal);
                let velocity_change = velocity - robot.velocity();
                let velocity_change_norm = velocity_change.norm();
                if velocity_change_norm > 0.0 {
                    let acceleration = self.rules.ROBOT_ACCELERATION
                        * touch_normal.y().max(0.0);
                    let robot_velocity = robot.velocity()
                        + (velocity_change.normalized() * acceleration * time_interval)
                        .clamp(velocity_change_norm);
                    robot.set_velocity(robot_velocity);
                }
            }
            if robot.action.use_nitro {
                let target_velocity_change = (robot.action.target_velocity() - robot.velocity())
                    .clamp(robot.nitro_amount() * self.rules.NITRO_POINT_VELOCITY_CHANGE);
                if target_velocity_change.norm() > 0.0 {
                    let acceleration = target_velocity_change.normalized()
                        * self.rules.ROBOT_NITRO_ACCELERATION;
                    let velocity_change = (acceleration * time_interval)
                        .clamp(target_velocity_change.norm());
                    let velocity = robot.velocity() + velocity_change;
                    robot.set_velocity(velocity);
                    let nitro_amount = robot.nitro_amount()
                        - velocity_change.norm() / self.rules.NITRO_POINT_VELOCITY_CHANGE;
                    robot.set_nitro_amount(nitro_amount);
                }
            }
            robot.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);
            robot.jump(robot.action.jump_speed, &self.rules);
        }

        self.ball.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);

        for i in 0 .. self.robots.len() - 1 {
            if self.robots[i].ignore {
                continue;
            }
            let (left, right) = self.robots.split_at_mut(i + 1);
            for j in 0 .. right.len() {
                if right[j].ignore {
                    continue;
                }
                let e = || { rng.gen_range(min_e, max_e) };
                Simulator::collide(e, &mut left[i], &mut right[j]);
            }
        }

        for i in 0 .. self.robots.len() {
            if self.robots[i].ignore {
                continue;
            }
            let robot = &mut self.robots[i];
            let e = || { rng.gen_range(min_e, max_e) };
            let collision_type = Simulator::collide(e, robot, &mut self.ball);
            let touch_normal = self.rules.arena.collide(robot);
            robot.set_touch_normal(touch_normal);
            if collision_type != RobotCollisionType::None {
                robot.collision_type = robot.collision_type.with(collision_type);
                self.ball.collision_type = self.ball.collision_type.with(BallCollisionType::Robot);
            }
        }

        if self.rules.arena.collide(&mut self.ball).is_some() {
            self.ball.collision_type = self.ball.collision_type.with(BallCollisionType::Arena);
        }

        if self.score == 0 {
            if self.ball.position().z() > self.rules.arena.depth / 2.0 + self.ball.radius() {
                self.score = 1;
            } else if self.ball.position().z() < -(self.rules.arena.depth / 2.0 + self.ball.radius()) {
                self.score = -1;
            }
        }

        for robot in self.robots.iter_mut() {
            if robot.nitro_amount() == self.rules.MAX_NITRO_AMOUNT {
                continue;
            }
            for nitro_pack in self.nitro_packs.iter_mut() {
                if nitro_pack.respawn_ticks.is_some() {
                    continue;
                }
                if robot.position().distance(nitro_pack.position()) <= robot.radius() + nitro_pack.radius {
                    robot.set_nitro_amount(self.rules.MAX_NITRO_AMOUNT);
                    nitro_pack.respawn_ticks = Some(self.rules.NITRO_PACK_RESPAWN_TICKS as i32);
                }
            }
        }

        self.current_micro_tick += 1;
    }

    pub fn collide<F>(mut e: F, a: &mut Solid, b: &mut Solid) -> RobotCollisionType
        where F: FnMut() -> f64 {

        let delta_position = b.position() - a.position();
        let distance = delta_position.norm();
        let penetration = a.radius() + b.radius() - distance;
        if penetration > 0.0 {
            let k_a = (1.0 / a.mass()) / ((1.0 / a.mass()) + (1.0 / b.mass()));
            let k_b = (1.0 / b.mass()) / ((1.0 / a.mass()) + (1.0 / b.mass()));
            let normal = delta_position.normalized();
            let a_position = a.position() - normal * penetration * k_a;
            let b_position = b.position() + normal * penetration * k_b;
            a.set_position(a_position);
            b.set_position(b_position);
            let delta_velocity = normal.dot(b.velocity() - a.velocity())
                - b.radius_change_speed() - a.radius_change_speed();
            if delta_velocity < 0.0 {
                let k = 1.0 + e();
                let impulse = normal * k * delta_velocity;
                let a_velocity = a.velocity() + impulse * k_a;
                let b_velocity = b.velocity() - impulse * k_b;
                a.set_velocity(a_velocity);
                b.set_velocity(b_velocity);
                return RobotCollisionType::KickBall;
            }
            return RobotCollisionType::TouchBall;
        }
        RobotCollisionType::None
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, relative_time: f64, render: &mut Render) {
        self.render_robots(relative_time, render);
        self.render_ball(relative_time, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_robots(&self, relative_time: f64, render: &mut Render) {
        let mut robots: Vec<&RobotExt> = self.robots.iter().collect::<Vec<_>>();
        robots.sort_by_key(|v| v.id());

        for (i, robot) in robots.iter().enumerate() {
            robot.render(
                relative_time,
                i as f64 / robots.len() as f64,
                render,
            );
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render_ball(&self, relative_time: f64, render: &mut Render) {
        self.ball.render(relative_time, render);
    }
}
