use crate::model::{Robot, Action, Rules};
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::common::IdGenerator;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

const MAX_TIME: f64 = 1.6666666666666667;
const NEAR_MICRO_TICKS_PER_TICK: usize = 25;
const FAR_MICRO_TICKS_PER_TICK: usize = 3;
const MAX_MICRO_TICK: i32 = 1000;
const MAX_TOTAL_MICRO_TICKS: i32 = 15000;
const MAX_ITERATIONS: usize = 5;

pub enum Order {
    Play(Play),
    WalkToGoalkeeperPosition(WalkToGoalkeeperPosition),
}

impl Order {
    pub fn try_play(robot: &Robot, world: &World, ctx: &mut OrderContext) -> Option<Order> {
        Play::try_new(robot, world, ctx)
            .map(|v| Order::Play(v))
    }

    pub fn walk_to_goalkeeper_position(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Order {
        Order::WalkToGoalkeeperPosition(
            WalkToGoalkeeperPosition::new(robot, world, order_id_generator)
        )
    }

    pub fn id(&self) -> i32 {
        match self {
            Order::Play(v) => v.id,
            Order::WalkToGoalkeeperPosition(v) => v.id,
        }
    }

    pub fn robot_id(&self) -> i32 {
        match self {
            Order::Play(v) => v.robot_id,
            Order::WalkToGoalkeeperPosition(v) => v.robot_id,
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            Order::Play(v) => v.score,
            Order::WalkToGoalkeeperPosition(v) => v.score,
        }
    }

    pub fn action(&self) -> &Action {
        match self {
            Order::Play(v) => &v.action,
            Order::WalkToGoalkeeperPosition(v) => &v.action,
        }
    }

    #[cfg(feature = "enable_stats")]
    pub fn stats(&self) -> &Stats {
        match self {
            Order::Play(v) => &v.stats,
            Order::WalkToGoalkeeperPosition(v) => &v.stats,
        }
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

        render.add(Object::text(format!(
            "  order:\n    score: {}\n    speed: {}\n    jump: {}\n",
            self.score(), self.action().target_velocity().norm(), self.action().jump_speed
        )));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_action(&self, robot: &Robot, render: &mut Render) {
        self.action().render(robot, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_sub(&self, render: &mut Render) {
        match self {
            Order::Play(v) => v.render(render),
            Order::WalkToGoalkeeperPosition(_) => (),
        }
    }
}

pub struct Play {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    pub time_to_ball: Option<f64>,
    #[cfg(feature = "enable_render")]
    pub position_to_jump: Option<Vec3>,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl Play {
    pub fn try_new(robot: &Robot, world: &World, ctx: &mut OrderContext) -> Option<Self> {
        log!(world.game.current_tick, "[{}] get optimal action robot_position={:?} robot_velocity={:?} ball_position={:?} ball_velocity={:?}", robot.id, robot.position(), robot.velocity(), world.game.ball.position(), world.game.ball.velocity());

        let mut ctx = InnerOrderContext {
            rng: ctx.rng,
            order_id_generator: ctx.order_id_generator,
            micro_ticks: ctx.micro_ticks,
            total_micro_ticks: 0,
            total_iterations: 0,
        };

        let mut order = if world.rules.is_flying(robot) {
            let fly = Self::try_continue_jump(0.0, robot, world, &mut ctx);
            let jump = Self::try_continue_jump(world.rules.ROBOT_MAX_JUMP_SPEED, robot, world, &mut ctx);

            Self::get_with_max_score(fly, jump)
        } else {
            let jump_to_ball = Self::try_jump_to_ball(robot, world, &mut ctx);
            let jump_at_position = Self::try_jump_at_position(robot, world, &mut ctx);

            Self::get_with_max_score(jump_at_position, jump_to_ball)
        };

        #[cfg(feature = "enable_stats")]
        {
            if let Some(v) = &mut order {
                v.stats.total_iterations = ctx.total_iterations;
                v.stats.total_micro_ticks += ctx.total_micro_ticks;
                v.stats.reached_play_limit = ctx.total_micro_ticks >= MAX_TOTAL_MICRO_TICKS;
                v.stats.reached_game_limit = world.is_micro_ticks_limit_reached(*ctx.micro_ticks);
            }
        }

        order
    }

    fn try_jump_at_position(robot: &Robot, world: &World, ctx: &mut InnerOrderContext) -> Option<Self> {
        let initial_simulator = make_initial_simulator(robot, world);
        let mut global_simulator = initial_simulator.clone();
        global_simulator.set_ignore_me(true);
        let time_interval = world.rules.tick_time_interval();
        let mut order: Option<Play> = None;
        let steps = [1usize, 3, 4, 8];
        let mut iterations = 0;

        while (iterations < MAX_ITERATIONS || order.is_none())
            && global_simulator.current_time() + time_interval < MAX_TIME
            && ctx.total_micro_ticks < MAX_TOTAL_MICRO_TICKS
            && !world.is_micro_ticks_limit_reached(*ctx.micro_ticks) {

            log!(
                world.game.current_tick, "[{}] try time point {} {}",
                robot.id, global_simulator.current_micro_tick(), global_simulator.current_time()
            );

            let ball_position = global_simulator.ball().position();
            let (distance, normal) = world.rules.arena.distance_and_normal(ball_position);

            if ball_position.y() < world.rules.max_robot_jump_height() || (
                distance < world.rules.max_robot_jump_height()
                && ball_position.y() < world.rules.max_robot_wall_walk_height()
                && Vec3::j().cos(normal) >= 0.0
            ) {

                log!(
                    world.game.current_tick,
                    "[{}] use time point {} {} position={:?} velocity={:?} ball_position={:?} ball_velocity={:?}",
                    robot.id, global_simulator.current_micro_tick(), global_simulator.current_time(),
                    global_simulator.me().position(), global_simulator.me().velocity(),
                    global_simulator.ball().position(), global_simulator.ball().velocity()
                );

                iterations += 1;

                let candidate = Self::try_jump_near_ball(&initial_simulator, &global_simulator, robot, world, ctx);

                if candidate.is_some() && (order.is_none() || order.as_ref().unwrap().score < candidate.as_ref().unwrap().score) {
                    order = candidate;

                    #[cfg(feature = "enable_stats")]
                    {
                        order.as_mut().unwrap().stats.iteration = iterations;
                        order.as_mut().unwrap().stats.current_step = steps[iterations.min(steps.len() - 1)];
                    }
                }
            }

            for _ in 0..steps[iterations.min(steps.len() - 1)] {
                global_simulator.tick(time_interval, NEAR_MICRO_TICKS_PER_TICK, ctx.rng);
                ctx.total_micro_ticks += NEAR_MICRO_TICKS_PER_TICK as i32;
                *ctx.micro_ticks += NEAR_MICRO_TICKS_PER_TICK;
            }
        }

        #[cfg(feature = "enable_stats")]
        {
            ctx.total_iterations = iterations;
        }

        order
    }

    fn try_jump_near_ball(initial_simulator: &Simulator, global_simulator: &Simulator,
                              robot: &Robot, world: &World, ctx: &mut InnerOrderContext) -> Option<Play> {
        use crate::my_strategy::scenarios::{Context, JumpAtPosition};

        let time_interval = world.rules.tick_time_interval();
        let ball_distance_limit = world.rules.ROBOT_MAX_RADIUS + world.rules.BALL_RADIUS;

        let points = get_points(&global_simulator, ctx.rng);

        let mut order: Option<Play> = None;

        for point in points {
            let target = {
                let mut robot = global_simulator.me().clone();
                robot.set_position(point);
                world.rules.arena.collide(&mut robot);
                robot.position()
            };
            let to_target = target - robot.position();
            let distance_to_target = to_target.norm();
            let required_speed = if global_simulator.current_time() > 0.0 {
                if distance_to_target > world.rules.ROBOT_MAX_GROUND_SPEED * 20.0 * time_interval {
                    world.rules.ROBOT_MAX_GROUND_SPEED
                } else {
                    distance_to_target / global_simulator.current_time()
                }
            } else {
                world.rules.ROBOT_MAX_GROUND_SPEED
            };
            let action_id = ctx.order_id_generator.next();
            log!(world.game.current_tick, "[{}] <{}> suggest target {}:{} distance={} speed={} target={:?}", robot.id, action_id, global_simulator.current_time(), global_simulator.current_micro_tick(), distance_to_target, required_speed, target);
            let mut local_simulator = initial_simulator.clone();
            let velocity = if distance_to_target > 1e-3 {
                to_target.normalized() * required_speed
            } else {
                Vec3::default()
            };
            log!(world.game.current_tick, "[{}] <{}> use velocity {}:{} {} {:?}", robot.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), velocity.norm(), velocity);
            let before_micro_ticks_per_tick = if local_simulator.me().position().distance(local_simulator.ball().position()) > ball_distance_limit + velocity.norm() * time_interval {
                log!(world.game.current_tick, "[{}] <{}> far", robot.id, action_id);
                FAR_MICRO_TICKS_PER_TICK
            } else {
                log!(world.game.current_tick, "[{}] <{}> near", robot.id, action_id);
                NEAR_MICRO_TICKS_PER_TICK
            };
            let mut time_to_ball = None;
            let mut time_to_goal = None;
            #[cfg(feature = "enable_render")]
            let mut history = vec![local_simulator.clone()];
            #[cfg(feature = "enable_stats")]
            let mut stats = Stats::new(robot.player_id, robot.id, world.game.current_tick, "jump_at_position");

            let mut scenario_ctx = Context {
                current_tick: world.game.current_tick,
                robot_id: robot.id,
                action_id,
                simulator: &mut local_simulator,
                rng: ctx.rng,
                time_to_ball: &mut time_to_ball,
                time_to_goal: &mut time_to_goal,
                #[cfg(feature = "enable_render")]
                history: &mut history,
                #[cfg(feature = "enable_stats")]
                stats: &mut stats,
            };

            let action = JumpAtPosition {
                ball: global_simulator.ball().base(),
                kick_ball_position: target,
                my_max_speed: required_speed,
                my_jump_speed: world.rules.ROBOT_MAX_JUMP_SPEED,
                ball_target: world.rules.get_goal_target(),
                max_time: MAX_TIME,
                tick_time_interval: time_interval,
                micro_ticks_per_tick_before_jump: before_micro_ticks_per_tick,
                micro_ticks_per_tick_after_jump: FAR_MICRO_TICKS_PER_TICK,
                max_micro_tick: MAX_MICRO_TICK,
            }.perform(&mut scenario_ctx);

            *ctx.micro_ticks += local_simulator.current_micro_tick() as usize;
            ctx.total_micro_ticks += local_simulator.current_micro_tick();

            if local_simulator.score() != 0 {
                log!(world.game.current_tick, "[{}] <{}> goal {}:{} score={}", robot.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.score());
            }

            if let Some(action) = action {
                let action_score = get_action_score(
                    &world.rules,
                    &local_simulator,
                    time_to_ball,
                    time_to_goal,
                    MAX_TIME + time_interval,
                    world.game.current_tick,
                    robot.id,
                    action_id,
                );

                #[cfg(feature = "enable_stats")]
                {
                    stats.micro_ticks_to_end = local_simulator.current_micro_tick();
                    stats.time_to_end = local_simulator.current_time();
                    stats.time_to_score = if local_simulator.score() != 0 {
                        Some(stats.time_to_end)
                    } else {
                        None
                    };
                    stats.score = local_simulator.score();
                    stats.action_score = action_score;
                    stats.reached_scenario_limit = local_simulator.current_micro_tick() >= MAX_MICRO_TICK;
                }

                log!(world.game.current_tick, "[{}] <{}> suggest action {}:{} score={} speed={}", robot.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), action_score, action.target_velocity().norm());
                if order.is_none() || order.as_ref().unwrap().score < action_score {
                    order = Some(Play {
                        id: action_id,
                        robot_id: robot.id,
                        action,
                        score: action_score,
                        time_to_ball,
                        #[cfg(feature = "enable_render")]
                        position_to_jump: Some(target),
                        #[cfg(feature = "enable_render")]
                        history,
                        #[cfg(feature = "enable_stats")]
                        stats,
                    });
                }
            }
        }

        order
    }

    fn try_jump_to_ball(robot: &Robot, world: &World, ctx: &mut InnerOrderContext) -> Option<Play> {
        use crate::my_strategy::scenarios::{Context, JumpToBall};

        if world.is_micro_ticks_limit_reached(*ctx.micro_ticks) {
            return None;
        }

        let action_id = ctx.order_id_generator.next();
        let mut local_simulator = make_initial_simulator(robot, world);
        let mut time_to_ball = None;
        let mut time_to_goal = None;
        let time_interval = world.rules.tick_time_interval();
        #[cfg(feature = "enable_render")]
        let mut history = vec![local_simulator.clone()];
        #[cfg(feature = "enable_stats")]
        let mut stats = Stats::new(robot.player_id, robot.id, world.game.current_tick, "jump_to_ball");

        let mut scenario_ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: robot.id,
            action_id,
            simulator: &mut local_simulator,
            rng: ctx.rng,
            time_to_ball: &mut time_to_ball,
            time_to_goal: &mut time_to_goal,
            #[cfg(feature = "enable_render")]
            history: &mut history,
            #[cfg(feature = "enable_stats")]
            stats: &mut stats,
        };

        let action = JumpToBall {
            max_time: MAX_TIME,
            tick_time_interval: time_interval,
            micro_ticks_per_tick_before_jump: NEAR_MICRO_TICKS_PER_TICK,
            micro_ticks_per_tick_after_jump: FAR_MICRO_TICKS_PER_TICK,
            max_micro_ticks: MAX_MICRO_TICK,
        }.perform(&mut scenario_ctx);

        *ctx.micro_ticks += local_simulator.current_micro_tick() as usize;
        ctx.total_micro_ticks += local_simulator.current_micro_tick();

        if let Some(action) = action {
            let action_score = get_action_score(
                &world.rules,
                &local_simulator,
                time_to_ball,
                time_to_goal,
                MAX_TIME + time_interval,
                world.game.current_tick,
                robot.id,
                action_id,
            );

            #[cfg(feature = "enable_stats")]
            {
                stats.micro_ticks_to_end = local_simulator.current_micro_tick();
                stats.time_to_end = local_simulator.current_time();
                stats.time_to_score = if local_simulator.score() != 0 {
                    Some(stats.time_to_end)
                } else {
                    None
                };
                stats.score = local_simulator.score();
                stats.action_score = action_score;
                stats.reached_scenario_limit = local_simulator.current_micro_tick() >= MAX_MICRO_TICK;
            }

            log!(
                world.game.current_tick, "[{}] <{}> suggest action far jump {}:{} score={}",
                robot.id, action_id,
                local_simulator.current_time(), local_simulator.current_micro_tick(), action_score
            );

            Some(Play {
                id: action_id,
                robot_id: robot.id,
                action,
                score: action_score,
                time_to_ball,
                #[cfg(feature = "enable_render")]
                position_to_jump: None,
                #[cfg(feature = "enable_render")]
                history,
                #[cfg(feature = "enable_stats")]
                stats,
            })
        } else {
            None
        }
    }

    fn try_continue_jump(jump_speed: f64, robot: &Robot, world: &World, ctx: &mut InnerOrderContext) -> Option<Play> {
        use crate::my_strategy::scenarios::{Context, ContinueJump};

        if world.is_micro_ticks_limit_reached(*ctx.micro_ticks) {
            return None;
        }

        let action_id = ctx.order_id_generator.next();
        let mut simulator = make_initial_simulator(robot, world);
        let mut time_to_ball = None;
        let mut time_to_goal = None;
        let time_interval = world.rules.tick_time_interval();
        #[cfg(feature = "enable_render")]
        let mut history = vec![simulator.clone()];
        #[cfg(feature = "enable_stats")]
        let mut stats = Stats::new(robot.player_id, robot.id, world.game.current_tick, "continue_jump");

        let mut scenario_ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: robot.id,
            action_id,
            simulator: &mut simulator,
            rng: ctx.rng,
            time_to_ball: &mut time_to_ball,
            time_to_goal: &mut time_to_goal,
            #[cfg(feature = "enable_render")]
            history: &mut history,
            #[cfg(feature = "enable_stats")]
            stats: &mut stats,
        };

        let action = ContinueJump {
            jump_speed,
            max_time: MAX_TIME,
            tick_time_interval: time_interval,
            micro_ticks_per_tick_before_land: NEAR_MICRO_TICKS_PER_TICK,
            micro_ticks_per_tick_after_land: FAR_MICRO_TICKS_PER_TICK,
            max_micro_ticks: MAX_MICRO_TICK,
        }.perform(&mut scenario_ctx);

        *ctx.micro_ticks += simulator.current_micro_tick() as usize;
        ctx.total_micro_ticks += simulator.current_micro_tick();

        let action_score = get_action_score(
            &world.rules,
            &simulator,
            time_to_ball,
            time_to_goal,
            MAX_TIME + time_interval,
            world.game.current_tick,
            robot.id,
            action_id,
        );

        #[cfg(feature = "enable_stats")]
        {
            stats.micro_ticks_to_end = simulator.current_micro_tick();
            stats.time_to_end = simulator.current_time();
            stats.time_to_score = if simulator.score() != 0 {
                Some(stats.time_to_end)
            } else {
                None
            };
            stats.score = simulator.score();
            stats.action_score = action_score;
        }

        log!(
            world.game.current_tick, "[{}] <{}> suggest action jump with jump_speed {} {}:{} score={}",
            robot.id, action_id, jump_speed,
            simulator.current_time(), simulator.current_micro_tick(), action_score
        );

        Some(Play {
            id: action_id,
            robot_id: robot.id,
            action,
            score: action_score,
            time_to_ball,
            #[cfg(feature = "enable_render")]
            position_to_jump: None,
            #[cfg(feature = "enable_render")]
            history,
            #[cfg(feature = "enable_stats")]
            stats,
        })
    }

    fn get_with_max_score(lhs: Option<Play>, rhs: Option<Play>) -> Option<Play> {
        if let Some(lhs) = lhs {
            if let Some(rhs) = rhs {
                if lhs.score < rhs.score {
                    Some(rhs)
                } else {
                    Some(lhs)
                }
            } else {
                Some(lhs)
            }
        } else {
            rhs
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        use crate::my_strategy::render::{Object, Color};

        if let Some(position) = self.position_to_jump {
            render.add(Object::sphere(position, 1.0, Color::new(0.5, 0.0, 0.0, 0.8)));
        }
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

fn get_action_score(rules: &Rules, simulator: &Simulator, time_to_ball: Option<f64>,
                    time_to_goal: Option<f64>, max_time: f64, current_tick: i32, robot_id: i32, action_id: i32) -> i32 {
    use crate::my_strategy::common::as_score;

    let ball = simulator.ball();
    let to_goal = rules.get_goal_target() - ball.position();
    let ball_goal_distance_score = if simulator.score() == 0 {
        1.0 - to_goal.norm()
            / Vec2::new(rules.arena.width + 2.0 * rules.arena.goal_depth, rules.arena.depth).norm()
    } else if simulator.score() > 0 {
        2.0
    } else {
        0.0
    };
    let ball_goal_direction_score = if ball.velocity().norm() > 0.0 {
        (to_goal.cos(ball.velocity()) + 1.0) / 2.0
    } else {
        0.0
    };
    let time_to_ball_score = if let Some(v) = time_to_ball {
        1.0 - v / max_time
    } else {
        0.0
    };
    let time_to_goal_score = if let Some(v) = time_to_goal {
        if simulator.score() > 0 {
            1.0 - v / max_time
        } else {
            v / max_time
        }
    } else {
        0.0
    };
    let distance_to_opponents_score = simulator.robots().iter()
        .filter(|v| !v.is_teammate())
        .map(|v| v.position().distance(ball.position()) / rules.arena.max_distance())
        .sum::<f64>() / (simulator.robots().len() / 2) as f64;
    let distance_to_teammate_score = 1.0 - simulator.robots().iter()
        .filter(|v| v.is_teammate() && v.id() != simulator.me().id())
        .map(|v| v.position().distance(ball.position()) / rules.arena.max_distance())
        .min_by_key(|v| as_score(*v))
        .unwrap_or_default();
    let score = 0.0
        + ball_goal_distance_score
        + 0.1 * ball_goal_direction_score
        + 0.5 * time_to_ball_score
        + 0.25 * time_to_goal_score
        + 0.1 * distance_to_opponents_score
        + 0.05 * distance_to_teammate_score;
    log!(
        current_tick,
        "[{}] <{}> action ball_goal_distance_score={} ball_goal_direction_score={} \
        time_to_ball_score={} time_to_goal_score={} distance_to_opponents_score={} \
        distance_to_teammate_score={} total={}",
        robot_id, action_id, ball_goal_distance_score, ball_goal_direction_score,
        time_to_ball_score, time_to_goal_score, distance_to_opponents_score,
        distance_to_teammate_score, score
    );
    as_score(score)
}

pub fn get_points(simulator: &Simulator, rng: &mut XorShiftRng) -> Vec<Vec3> {
    use crate::my_strategy::physics::get_min_distance_between_spheres;
    use crate::my_strategy::random::Rng;
    use crate::my_strategy::common::Clamp;
    use crate::my_strategy::plane::Plane;
    use crate::my_strategy::mat3::Mat3;

    let ball = simulator.ball();
    let robot = simulator.me();
    let rules = simulator.rules();

    let distance_to_ball = ball.position().distance(robot.position());
    let time_to_ball = rules.time_for_distance(rules.ROBOT_MAX_GROUND_SPEED, distance_to_ball);
    let max_time_diff = 2.0 * (rules.ROBOT_RADIUS + rules.BALL_RADIUS) / rules.ROBOT_MAX_GROUND_SPEED;
    let number = if time_to_ball < simulator.current_time() + max_time_diff {
        if time_to_ball < rules.tick_time_interval() * 10.0 {
            7
        } else {
            3
        }
    } else {
        1
    };
    let base_position = ball.projected_to_arena_position_with_shift(rules.ROBOT_MIN_RADIUS);
    let to_robot = (robot.position() - base_position).normalized();
    let min_distance = get_min_distance_between_spheres(
        ball.distance_to_arena(),
        rules.BALL_RADIUS,
        rules.ROBOT_MIN_RADIUS,
    ).unwrap_or(0.0);
    let max_distance = base_position.distance(robot.position())
        .clamp(min_distance + 1e-3, rules.BALL_RADIUS + rules.ROBOT_MAX_RADIUS);
    let base_direction = Plane::projected(to_robot, ball.normal_to_arena()).normalized();
    let mut result = Vec::new();
    for _ in 0..number {
        let distance = rng.gen_range(min_distance, max_distance);
        let angle = rng.gen_range(-std::f64::consts::PI, std::f64::consts::PI);
        let rotation = Mat3::rotation(ball.normal_to_arena(), angle);
        let position = base_position + rotation * base_direction * distance;
        result.push(rules.arena.projected_with_shift(position, rules.ROBOT_MAX_RADIUS));
    }

    result
}

pub struct WalkToGoalkeeperPosition {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl WalkToGoalkeeperPosition {
    pub fn new(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Self {
        let target = world.rules.get_goalkeeper_position();
        let to_target = target - robot.position();
        let velocity = if to_target.norm() > world.rules.min_running_distance() {
            to_target.normalized() * world.rules.ROBOT_MAX_GROUND_SPEED
        } else {
            to_target
        };
        let mut action = Action::default();
        action.set_target_velocity(velocity);
        WalkToGoalkeeperPosition {
            id: order_id_generator.next(),
            robot_id: robot.id,
            action,
            score: 0,
            #[cfg(feature = "enable_stats")]
            stats: Stats::new(robot.player_id, robot.id, world.game.current_tick, "walk_to_goalkeeper_position"),
        }
    }
}

pub struct OrderContext<'r> {
    pub rng: &'r mut XorShiftRng,
    pub order_id_generator: &'r mut IdGenerator,
    pub micro_ticks: &'r mut usize,
}

struct InnerOrderContext<'r> {
    pub rng: &'r mut XorShiftRng,
    pub order_id_generator: &'r mut IdGenerator,
    pub micro_ticks: &'r mut usize,
    pub total_micro_ticks: i32,
    pub total_iterations: usize,
}

fn make_initial_simulator(robot: &Robot, world: &World) -> Simulator {
    use crate::my_strategy::entity::Entity;

    let mut result = Simulator::new(world, robot.id);
    result.robots_mut().iter_mut()
        .filter(|v| !v.is_teammate())
        .for_each(|v| {
            let velocity = v.velocity();
            v.action_mut().set_target_velocity(velocity);
        });
    result
}
