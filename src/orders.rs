use crate::model::{Action, Robot, Rules};
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::targets::Target;
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::{Simulator, BallExt};
use crate::my_strategy::common::IdGenerator;
use crate::my_strategy::physics::MoveEquation;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

const BALL_MAX_SIMULATION_TICKS: i32 = 100;
const BALL_MICRO_TICKS_PER_TICK: usize = 5;
const PLAY_MAX_CANDIDATES: usize = 3;
const PLAY_MAX_MICRO_TICKS_PER_TICK: usize = 20;
const PLAY_MAX_OPTIMIZATION_ITERATIONS: usize = 10;
const PLAY_MAX_SIMULATION_TICKS: i32 = 100;
const PLAY_MIN_MICRO_TICKS_PER_TICK: usize = 2;
const WALK_MAX_CANDIDATES: usize = 3;
const WALK_MAX_SIMULATION_TICKS: i32 = 100;
const WALK_MICRO_TICKS_PER_TICK: usize = 1;

pub struct Walk {
    pub id: i32,
    pub action: Action,
    pub score: i32,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

pub struct PlayParams {
    pub angle: f64,
    pub distance: f64,
}

pub struct Play {
    pub id: i32,
    pub action: Action,
    pub score: i32,
    pub params: Option<PlayParams>,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

pub enum Order {
    Idle(i32),
    Walk(Walk),
    Play(Play)
}

impl Order {
    pub fn new(prev: Option<&Order>, target: &Target, robot: &Robot, world: &World, rng: &mut XorShiftRng) -> Self {
        let mut action_id_gen = IdGenerator::new();
        match target {
            Target::None => Order::Idle(action_id_gen.next()),
            Target::Ball => Self::from_ball(prev, robot, world, rng, &mut action_id_gen),
            Target::NitroPack(nitro_pack_id) => Self::from_nitro_pack(*nitro_pack_id, robot, world, &mut action_id_gen),
            Target::GoalkeeperPosition => Self::from_goalkeeper_position(robot, world, &mut action_id_gen),
            Target::Robot(robot_id) => Self::from_robot(*robot_id, robot, world, &mut action_id_gen),
        }
    }

    pub fn id(&self) -> i32 {
        match self {
            Order::Idle(id) => *id,
            Order::Walk(walk) => walk.id,
            Order::Play(play) => play.id,
        }
    }

    pub fn action(&self) -> Action {
        match self {
            Order::Idle(_) => Action::default(),
            Order::Walk(walk) => walk.action,
            Order::Play(play) => play.action,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Order::Idle(_) => "idle",
            Order::Walk(_) => "walk",
            Order::Play(_) => "play",
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            Order::Idle(_) => 0,
            Order::Walk(v) => v.score,
            Order::Play(v) => v.score,
        }
    }

    #[cfg(feature = "enable_stats")]
    pub fn stats(&self) -> Stats {
        match self {
            Order::Idle(_) => Stats::default(),
            Order::Walk(v) => v.stats.clone(),
            Order::Play(v) => v.stats.clone(),
        }
    }

    fn from_ball(prev: Option<&Order>, robot: &Robot, world: &World, rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Order {
        let max_near_distance = get_max_distance_to_play(&world.rules);
        let distance = robot.position().distance(world.game.ball.position());

        if distance > max_near_distance {
            Self::walk_to_ball_hit_arena_position(robot, world, rng, action_id_gen)
        } else {
            let prev_play = if let Some(order) = prev {
                match order {
                    Order::Play(play) => Some(play),
                    _ => None,
                }
            } else {
                None
            };
            Self::play(prev_play, robot, world, rng, action_id_gen)
        }.unwrap_or(Self::walk(
            action_id_gen.next(),
            world.game.ball.position(),
            world.rules.ROBOT_MAX_GROUND_SPEED,
            1.0,
            robot,
            world,
        ))
    }

    fn from_nitro_pack(nitro_pack_id: i32, robot: &Robot, world: &World, action_id_gen: &mut IdGenerator) -> Order {
        let target = world.get_nitro_pack(nitro_pack_id).position();

        Self::walk(action_id_gen.next(), target, world.rules.ROBOT_MAX_GROUND_SPEED, 1.0, robot, world)
    }

    fn from_robot(robot_id: i32, robot: &Robot, world: &World, action_id_gen: &mut IdGenerator) -> Order {
        let target = world.get_robot(robot_id).position();

        Self::walk(action_id_gen.next(), target, world.rules.ROBOT_MAX_GROUND_SPEED, 1.0, robot, world)
    }

    fn from_goalkeeper_position(robot: &Robot, world: &World, action_id_gen: &mut IdGenerator) -> Order {
        let target = world.rules.get_goalkeeper_position();

        Self::walk(action_id_gen.next(), target, 0.0, 1.0, robot, world)
    }

    fn play(prev: Option<&Play>, robot: &Robot, world: &World, rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Option<Order> {
        log!(world.game.current_tick, "[{}] try play", robot.id);

        let mut result: Option<Play> = None;
        let states = Self::collect_playable_ball_states(robot, world, rng);

        for (time, ball) in &states[0..PLAY_MAX_CANDIDATES.min(states.len())] {
            if let Some(candidate) = Play::try_new(prev, robot, world, ball, *time, rng, action_id_gen) {
                if result.is_none() || result.as_ref().unwrap().score < candidate.score {
                    log!(
                        world.game.current_tick, "[{}] <{}> suggest play score={}",
                        robot.id, candidate.id, candidate.score
                    );
                    result = Some(candidate);
                }
            }
        }

        if let Some(v) = result {
            Some(Order::Play(v))
        } else {
            Self::walk_to_ball_hit_arena_position_with_states(&states, robot, world, rng, action_id_gen)
        }
    }

    fn walk_to_ball_hit_arena_position(robot: &Robot, world: &World, rng: &mut XorShiftRng,
                                       action_id_gen: &mut IdGenerator) -> Option<Order> {
        Self::walk_to_ball_hit_arena_position_with_states(
            &Self::collect_playable_ball_states(robot, world, rng),
            robot, world, rng, action_id_gen,
        )
    }

    fn walk_to_ball_hit_arena_position_with_states(states: &Vec<(f64, BallExt)>, robot: &Robot, world: &World,
                                                   rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Option<Order> {
        use crate::my_strategy::entity::Entity;

        log!(world.game.current_tick, "[{}] try walk to ball hit arena position", robot.id);

        let mut result: Option<Walk> = None;

        for (time, ball) in &states[0..WALK_MAX_CANDIDATES.min(states.len())] {
            let candidate = Walk::with_score(
                ball.position(),
                world.rules.ROBOT_MAX_GROUND_SPEED,
                *time,
                robot,
                world,
                rng,
                action_id_gen
            );
            if result.is_none() || result.as_ref().unwrap().score < candidate.score {
                log!(
                    world.game.current_tick, "[{}] <{}> suggest walk score={}",
                    robot.id, candidate.id, candidate.score
                );
                result = Some(candidate);
            }
        }

        if let Some(v) = result {
            Some(Order::Walk(v))
        } else {
            None
        }
    }

    fn collect_playable_ball_states(robot: &Robot, world: &World, rng: &mut XorShiftRng) -> Vec<(f64, BallExt)> {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::Solid;
        use crate::my_strategy::common::as_score;

        let mut ball_simulator = Self::make_ball_simulator(&world, robot.id);
        let robot_ext = ball_simulator.me().clone();

        let tick_time_interval = world.rules.tick_time_interval();
        let micro_ticks_per_tick = BALL_MICRO_TICKS_PER_TICK;
        let max_time = BALL_MAX_SIMULATION_TICKS as f64 * tick_time_interval;
        let mut result = Vec::new();
        let move_equation = MoveEquation::from_entity(&robot_ext, &world.rules);
        let is_robot_on_surface = robot_ext.distance_to_arena() - robot_ext.radius() < 1e-3;

        while ball_simulator.current_time() < max_time {
            let is_able_to_play = Self::is_able_to_play(
                ball_simulator.current_time(),
                ball_simulator.ball(),
                world,
            );
            let will_robot_hit_ball = Self::will_robot_hit_ball(
                &move_equation,
                ball_simulator.current_time(),
                &ball_simulator.ball(),
                &world.rules,
            );
            if is_able_to_play && is_robot_on_surface {
                let to_ball = ball_simulator.ball()
                    .projected_to_arena_position_with_shift(world.rules.ROBOT_RADIUS) - robot.position();
                let time = world.rules.time_for_distance(
                    robot_ext.velocity().dot(to_ball.normalized()),
                    to_ball.norm(),
                );
                result.push((
                    as_score((ball_simulator.current_time() - time).abs()),
                    ball_simulator.current_time(),
                    ball_simulator.ball().clone(),
                ));
            } else if will_robot_hit_ball {
                result.push((
                    0,
                    ball_simulator.current_time(),
                    ball_simulator.ball().clone(),
                ));
            }

            ball_simulator.tick(tick_time_interval, micro_ticks_per_tick, rng);
        }

        result.sort_by_key(|(penalty, time, _)| (*penalty, as_score(*time)));

        result.into_iter().map(|(_, time, ball)| (time, ball)).collect()
    }

    fn make_ball_simulator(world: &World, me_id: i32) -> Simulator {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::Solid;

        let mut ball_simulator = Simulator::new(world, me_id);
        ball_simulator.set_ignore_me(true);
        ball_simulator.robots_mut().iter_mut()
            .filter(|v| v.id() != me_id)
            .for_each(|v| {
                let velocity = v.velocity();
                v.action_mut().set_target_velocity(velocity);
                if v.radius() > world.rules.ROBOT_MIN_RADIUS {
                    v.action_mut().jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
                }
            });

        ball_simulator
    }

    fn is_able_to_play(time: f64, ball: &BallExt, world: &World) -> bool {
        use crate::my_strategy::entity::Entity;

        let ball_position = ball.position();
        let (distance, normal) = world.rules.arena.distance_and_normal(ball_position);

        ball_position.y() < world.rules.max_robot_jump_height() || (
            distance < world.rules.max_robot_jump_height()
            && ball_position.y() < world.rules.max_robot_wall_walk_height()
            && Vec3::j().cos(normal) >= 0.0
        )
    }

    fn will_robot_hit_ball(move_equation: &MoveEquation, time: f64, ball: &BallExt, rules: &Rules) -> bool {
        use crate::my_strategy::entity::Entity;

        move_equation.get_position(time).distance(ball.position())
            < rules.ROBOT_MAX_RADIUS + rules.BALL_RADIUS
    }

    fn walk(id: i32, target: Vec3, final_speed: f64, time: f64, robot: &Robot, world: &World) -> Order {
        log!(world.game.current_tick, "[{}] try walk", robot.id);

        Order::Walk(Walk::new(id, target, final_speed, time, robot, world))
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, robot: &Robot, render: &mut Render) {
        self.render_text(render);
        self.render_action(robot, render);
        self.render_sub(render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_text(&self, render: &mut Render) {
        use crate::my_strategy::render::Object;

        render.add(Object::text(
            format!(
                "  order: {}",
                match self {
                    Order::Idle(v) => format!("Idle {:?}", (v)),
                    Order::Walk(v) => format!("Walk {:?}", (v.id, v.score, &v.action)),
                    Order::Play(v) => format!("Play {:?}", (v.id, v.score, &v.action)),
                }
            )
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_action(&self, robot: &Robot, render: &mut Render) {
        self.action().render(robot, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_sub(&self, render: &mut Render) {
        match self {
            Order::Idle(_) => (),
            Order::Walk(walk) => walk.render(render),
            Order::Play(play) => play.render(render),
        }
    }
}

impl Walk {
    pub fn new(id: i32, target: Vec3, final_speed: f64, time: f64, robot: &Robot, world: &World) -> Self {
        use crate::my_strategy::scenarios::WalkToPosition;

        let (_, normal) = world.rules.arena.distance_and_normal(robot.position());
        let mut action = Action::default();
        action.set_target_velocity(WalkToPosition::get_target_velocity(
            target,
            final_speed,
            world.rules.ROBOT_MAX_GROUND_SPEED,
            world.rules.ROBOT_ACCELERATION,
            world.rules.tick_time_interval(),
            time,
            world.rules.ROBOT_MAX_GROUND_SPEED,
            robot.position(),
            robot.velocity(),
            normal,
        ));

        Walk {
            id,
            action,
            score: 0,
            #[cfg(feature = "enable_render")]
            history: Vec::new(),
            #[cfg(feature = "enable_stats")]
            stats: Stats::default(),
        }
    }

    pub fn with_score(target: Vec3, final_speed: f64, time: f64, robot: &Robot, world: &World,
                      rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Self {
        use crate::my_strategy::scenarios::{Context, WalkToPosition, Output};

        let mut simulator = make_initial_simulator(robot, world);
        let mut output = Output::default();
        #[cfg(feature = "enable_render")]
        let mut history = vec![simulator.clone()];
        #[cfg(feature = "enable_stats")]
        let mut stats = Stats::default();

        let mut ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: robot.id,
            action_id: action_id_gen.next(),
            simulator: &mut simulator,
            rng,
            output: &mut output,
            #[cfg(feature = "enable_render")]
            history: Some(&mut history),
            #[cfg(feature = "enable_stats")]
            stats: Some(&mut stats),
        };

        let action = WalkToPosition {
            target,
            final_speed,
            max_time: WALK_MAX_SIMULATION_TICKS as f64 * world.rules.tick_time_interval(),
            tick_time_interval: world.rules.tick_time_interval(),
            micro_ticks_per_tick: WALK_MICRO_TICKS_PER_TICK,
        }.perform(&mut ctx);

        Walk {
            id: ctx.action_id,
            action,
            score: Self::get_action_score(target, &ctx.simulator),
            #[cfg(feature = "enable_render")]
            history,
            #[cfg(feature = "enable_stats")]
            stats,
        }
    }

    fn get_action_score(target: Vec3, simulator: &Simulator) -> i32 {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::common::as_score;

        let distance_score = 1.0 - simulator.me().position().distance(target)
            / simulator.rules().arena.max_distance();

        as_score(distance_score)
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        render_history(&self.history, render);
    }
}

impl Play {
    pub fn try_new(prev: Option<&Play>, robot: &Robot, world: &World, ball: &BallExt, time: f64,
                   rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Option<Self> {
        let (distance, _) = world.rules.arena.distance_and_normal(robot.position());
        if distance - robot.radius > 1e-3 {
            Self::jump_to_ball(robot, world, ball, time, rng, action_id_gen)
        } else {
            Self::jump_at_position(prev, robot, world, ball, time, rng, action_id_gen)
        }
    }

    pub fn jump_at_position(prev: Option<&Play>, robot: &Robot, world: &World, ball: &BallExt, time: f64,
                            rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Option<Self> {
        use crate::my_strategy::optimization::minimize2d;
        use crate::my_strategy::physics::get_min_distance_between_spheres;
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::scenarios::{Context, JumpAtPosition, Output};
        use crate::my_strategy::plane::Plane;
        use crate::my_strategy::mat3::Mat3;
        use crate::my_strategy::common::{Clamp, Square, as_score};

        let base_position = ball.projected_to_arena_position_with_shift(world.rules.ROBOT_MIN_RADIUS);
        let ball_target = world.rules.get_goal_target();
        let to_ball = base_position - robot.position();
        let base_direction = Plane::projected(to_ball, ball.normal_to_arena()).normalized();
        let time_to_jump = time;
        let min_distance = get_min_distance_between_spheres(
            ball.position().y(),
            world.rules.BALL_RADIUS,
            world.rules.ROBOT_MIN_RADIUS,
        ).unwrap_or(world.rules.ROBOT_MIN_RADIUS).max(world.rules.ROBOT_MIN_RADIUS);
        let distance_to_ball = robot.position().distance(base_position);
        let max_distance = get_max_distance_to_jump(&world.rules).min(distance_to_ball);
        let initial_simulator = make_initial_simulator(robot, world);

        let (initial_angle, initial_distance) = prev
            .map(|v| {
                v.params.as_ref()
                    .map(|v| {
                        (v.angle, v.distance.clamp(min_distance, max_distance))
                    })
                    .unwrap_or((std::f64::consts::PI, (max_distance + min_distance) / 2.0))
            })
            .unwrap_or((std::f64::consts::PI, (max_distance + min_distance) / 2.0));

        log!(
            world.game.current_tick,
            "[{}] find optimal jump at position base_position={:?} ball_target={:?} \
                base_direction={:?} time_to_jump={} min_distance={} max_distance={} \
                distance_to_ball={}",
            robot.id, base_position, ball_target,
            base_direction, time_to_jump, min_distance, max_distance,
            distance_to_ball
        );

        let clamp_angle = |v: f64| {
            v.clamp(
                initial_angle - std::f64::consts::PI,
                initial_angle + std::f64::consts::PI,
            )
        };
        let clamp_distance = |v: f64| {
            v.clamp(min_distance, max_distance)
        };

        let simulate = |angle: f64, distance: f64, ctx: &mut Context| {
            let rotation = Mat3::rotation(ball.normal_to_arena(), angle);
            let jump_position = world.rules.arena.projected_with_shift(
                base_position + rotation * base_direction * distance,
                world.rules.ROBOT_MAX_RADIUS
            );

            if ctx.action_id >= 0 {
                log!(
                    world.game.current_tick,
                    "[{}] <{}> simulate jump_position={:?} robot_position={:?} distance={}",
                    robot.id, ctx.action_id, jump_position,
                    ctx.simulator.me().position(),
                    ctx.simulator.me().position().distance(jump_position)
                );
            }

            let jump_at_position = JumpAtPosition {
                jump_position,
                speed_when_jump: world.rules.ROBOT_MAX_GROUND_SPEED,
                jump_speed: world.rules.ROBOT_MAX_JUMP_SPEED,
                time_to_jump,
                ball_target,
                max_time: PLAY_MAX_SIMULATION_TICKS as f64 * world.rules.tick_time_interval(),
                tick_time_interval: world.rules.tick_time_interval(),
                micro_ticks_per_tick_before_jump: PLAY_MIN_MICRO_TICKS_PER_TICK
                    + (
                        (PLAY_MAX_MICRO_TICKS_PER_TICK - PLAY_MIN_MICRO_TICKS_PER_TICK) as f64
                        * (1.0 - distance_to_ball / world.rules.arena.max_distance())
                            .clamp(0.0, 1.0)
                    ).round() as usize,
                micro_ticks_per_tick_after_jump: PLAY_MIN_MICRO_TICKS_PER_TICK,
            };

            let action = jump_at_position.perform(ctx);

            if ctx.action_id >= 0 {
                log!(
                    world.game.current_tick,
                    "[{}] <{}> simulate robot_position={:?} distance={} micro_ticks={}",
                    robot.id, ctx.action_id, ctx.simulator.me().position(),
                    ctx.simulator.me().position().distance(jump_position),
                    ctx.simulator.current_micro_tick()
                );
            }

            (
                action,
                Self::get_action_score(
                    &ctx.simulator,
                    ball_target,
                    ctx.output.time_to_ball,
                    jump_at_position.max_time + world.rules.tick_time_interval(),
                )
            )
        };

        let get_score = |angle: f64, distance: f64| {
            let clamped_angle = clamp_angle(angle);
            let clamped_distance = clamp_distance(distance);

            let mut simulator = initial_simulator.clone();
            let mut output = Output::default();
            let mut rng = rng.clone();

            let mut ctx = Context {
                current_tick: world.game.current_tick,
                robot_id: robot.id,
                action_id: -1,
                simulator: &mut simulator,
                rng: &mut rng,
                output: &mut output,
                #[cfg(feature = "enable_render")]
                history: None,
                #[cfg(feature = "enable_stats")]
                stats: None,
            };

            let angle_overflow_penalty = (angle - clamped_angle).square();
            let distance_overflow_penalty = (distance - clamped_distance).square();

            let (_, action_score) = simulate(
                clamped_angle,
                clamped_distance,
                &mut ctx,
            );

            let min_distance_to_ball_score = if let Some(v) = output.min_distance_to_ball {
                1.0 - v / world.rules.arena.max_distance()
            } else {
                0.0
            };

            let result = 0.0
                + angle_overflow_penalty
                + distance_overflow_penalty
                - action_score
                - min_distance_to_ball_score;

            verbose_log!(
                world.game.current_tick,
                "[{}] get_score angle={} distance={} result={}",
                robot.id, angle, distance, result
            );

            result
        };

        let result = minimize2d(
            &(
                initial_angle,
                initial_distance,
            ),
            PLAY_MAX_OPTIMIZATION_ITERATIONS,
            get_score,
        );

        let result_angle = clamp_angle(result.0);
        let result_distance = clamp_distance(result.1);

        let mut simulator = initial_simulator.clone();
        let mut output = Output::default();
        #[cfg(feature = "enable_render")]
        let mut history = vec![simulator.clone()];
        #[cfg(feature = "enable_stats")]
        let mut stats = Stats::default();

        let mut ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: robot.id,
            action_id: action_id_gen.next(),
            simulator: &mut simulator,
            rng,
            output: &mut output,
            #[cfg(feature = "enable_render")]
            history: Some(&mut history),
            #[cfg(feature = "enable_stats")]
            stats: Some(&mut stats),
        };

        log!(
            world.game.current_tick,
            "[{}] <{}> find optimal jump at position result_angle={} result_distance={}",
            robot.id, ctx.action_id, result_angle, result_distance
        );

        let (action, action_score) = simulate(
            result_angle,
            result_distance,
            &mut ctx,
        );

        if let Some(action) = action {
            Some(Play {
                id: ctx.action_id,
                action,
                score: as_score(action_score),
                params: if action_score > 0.0 {
                    Some(PlayParams {
                        angle: result_angle,
                        distance: result_distance,
                    })
                } else {
                    None
                },
                #[cfg(feature = "enable_render")]
                history,
                #[cfg(feature = "enable_stats")]
                stats,
            })
        } else {
            None
        }
    }

    pub fn jump_to_ball(robot: &Robot, world: &World, ball: &BallExt, time: f64,
                        rng: &mut XorShiftRng, action_id_gen: &mut IdGenerator) -> Option<Self> {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::scenarios::{Context, JumpToBall, Output};
        use crate::my_strategy::common::{Clamp, as_score};

        let ball_target = world.rules.get_goal_target();
        let distance_to_ball = robot.position().distance(ball.position());

        let mut simulator = make_initial_simulator(robot, world);
        let mut output = Output::default();
        #[cfg(feature = "enable_render")]
        let mut history = vec![simulator.clone()];
        #[cfg(feature = "enable_stats")]
        let mut stats = Stats::default();

        let mut ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: robot.id,
            action_id: action_id_gen.next(),
            simulator: &mut simulator,
            rng,
            output: &mut output,
            #[cfg(feature = "enable_render")]
            history: Some(&mut history),
            #[cfg(feature = "enable_stats")]
            stats: Some(&mut stats),
        };

        let jump_to_ball = JumpToBall {
            jump_speed: world.rules.ROBOT_MAX_JUMP_SPEED,
            ball_target,
            max_time: PLAY_MAX_SIMULATION_TICKS as f64 * world.rules.tick_time_interval(),
            tick_time_interval: world.rules.tick_time_interval(),
            micro_ticks_per_tick_before_jump: PLAY_MIN_MICRO_TICKS_PER_TICK
                + (
                    (PLAY_MAX_MICRO_TICKS_PER_TICK - PLAY_MIN_MICRO_TICKS_PER_TICK) as f64
                    * (1.0 - distance_to_ball / get_max_distance_to_play(&world.rules))
                        .clamp(0.0, 1.0)
                ).round() as usize,
            micro_ticks_per_tick_after_jump: PLAY_MIN_MICRO_TICKS_PER_TICK,
        };

        let action = jump_to_ball.perform(&mut ctx);

        let action_score = Self::get_action_score(
            &ctx.simulator,
            ball_target,
            ctx.output.time_to_ball,
            jump_to_ball.max_time + world.rules.tick_time_interval(),
        );

        if let Some(action) = action {
            Some(Play {
                id: ctx.action_id,
                action,
                score: as_score(action_score),
                params: None,
                #[cfg(feature = "enable_render")]
                history,
                #[cfg(feature = "enable_stats")]
                stats,
            })
        } else {
            None
        }
    }

    fn get_action_score(simulator: &Simulator, ball_target: Vec3, time_to_ball: Option<f64>, max_time: f64) -> f64 {
        use crate::my_strategy::entity::Entity;

        let ball = simulator.ball();
        let to_ball_target = ball_target - ball.position();
        let ball_goal_distance_score = 1.0 - to_ball_target.norm()
            / simulator.rules().arena.max_distance();
        let ball_goal_direction_score = (to_ball_target.cos(ball.velocity()) + 1.0) / 2.0;
        let time_score = if let Some(v) = time_to_ball {
            1.0 - v / max_time
        } else {
            0.0
        };
        let goal_score = if simulator.score() > 0 {
            1.0
        } else if simulator.score() < 0 {
            0.0
        } else {
            0.5
        };

        (
            0.0
                + ball_goal_distance_score
                + 0.1 * ball_goal_direction_score
                + 0.5 * time_score
                + 2.0 * goal_score
        ) / (1.0 + 0.1 + 0.5 + 2.0)
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        render_history(&self.history, render);
    }
}

#[cfg(feature = "enable_render")]
pub fn render_history(history: &Vec<Simulator>, render: &mut Render) {
    if history.is_empty() {
        return;
    }

    let max_time = history.last().unwrap().current_time();

    for state in history.iter() {
        state.render(
            state.current_time() / if max_time == 0.0 { 1.0 } else { max_time },
            render,
        );
    }
}

fn make_initial_simulator(robot: &Robot, world: &World) -> Simulator {
    use crate::my_strategy::entity::Entity;
    use crate::my_strategy::simulator::Solid;

    let mut result = Simulator::new(world, robot.id);
    result.robots_mut().iter_mut()
        .filter(|v| v.id() != robot.id)
        .for_each(|v| {
            let velocity = v.velocity();
            v.action_mut().set_target_velocity(velocity);
            if v.radius() > world.rules.ROBOT_MIN_RADIUS {
                v.action_mut().jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
            }
        });
    result
}

fn get_max_distance_to_jump(rules: &Rules) -> f64 {
    rules.ROBOT_MAX_RADIUS + rules.BALL_RADIUS + rules.min_running_distance()
}

fn get_max_distance_to_play(rules: &Rules) -> f64 {
    rules.arena.depth / 3.0
}
