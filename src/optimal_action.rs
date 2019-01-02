use crate::model::{Robot, Action, Ball, Rules};
use crate::my_strategy::random::Rng;
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::{Simulator, CollisionType};
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::simulator::Solid;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::render::Render;
#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Color;
use crate::my_strategy::optimization::optimize1d;

pub struct BallState {
    pub position: Vec3,
}

pub struct RobotState {
    pub id: i32,
    pub radius: f64,
    pub position: Vec3,
    pub velocity: Vec3,
}

pub struct State {
    pub ball: BallState,
    pub me: RobotState,
    pub robots: Vec<RobotState>,
}

impl State {
    pub fn new(simulator: &Simulator) -> Self {
        let ball = simulator.ball().base();
        let me = simulator.me().base();
        let mut robots: Vec<RobotState> = simulator.robots().iter()
            .filter(|v| !v.is_me())
            .map(|v| RobotState {
                id: v.id(),
                radius: v.radius(),
                position: v.position(),
                velocity: v.velocity(),
            })
            .collect();
        robots.sort_by_key(|v| v.id);
        State {
            ball: BallState {
                position: ball.position(),
            },
            me: RobotState {
                id: me.id,
                radius: me.radius,
                position: me.position(),
                velocity: me.velocity(),
            },
            robots,
        }
    }
}

#[derive(Default, Serialize)]
pub struct Stats {
    pub micro_ticks_to_near: i32,
    pub micro_ticks_to_jump: i32,
    pub micro_ticks_to_watch: i32,
    pub micro_ticks_to_end: i32,
    pub time_to_near: f64,
    pub time_to_jump: f64,
    pub time_to_watch: f64,
    pub time_to_end: f64,
    pub time_to_score: Option<f64>,
    pub iterations: usize,
    pub score: i32,
    pub jump_simulation: bool,
    pub far_jump_simulation: bool,
    pub action_score: i32,
    pub total_micro_ticks: i32,
}

pub struct OptimalAction {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    pub history: Vec<State>,
    pub stats: Stats,
    pub ball_hit_position: Option<Vec3>,
}

#[cfg(feature = "enable_render")]
const OPTIMAL_ME_POSITION: Color = Color::new(0.0, 0.8, 0.4, 0.5);

#[cfg(feature = "enable_render")]
const OPTIMAL_BALL_POSITION: Color = Color::new(0.0, 0.4, 0.8, 0.5);

impl Robot {
    pub fn get_optimal_action(&self, world: &World, rng: &mut XorShiftRng, render: &mut Render) -> OptimalAction {
        use crate::my_strategy::physics::get_min_distance_between_spheres;

        log!(world.game.current_tick, "[{}] get optimal action robot_position={:?} robot_velocity={:?} ball_position={:?} ball_velocity={:?}", self.id, self.position(), self.velocity(), world.game.ball.position(), world.game.ball.velocity());
        let initial_simulator = {
            let mut s = Simulator::new(world, self.id);
            s.robots_mut().iter_mut()
                .filter(|v| !v.is_teammate())
                .for_each(|v| v.action.set_target_velocity(v.velocity()));
            s
        };
        let mut global_simulator = initial_simulator.clone();
        global_simulator.set_ignore_me(true);
        let default_action = Action::default();
        let near_micro_ticks_per_tick = world.rules.MICROTICKS_PER_TICK / 4;
        let far_micro_ticks_per_tick = world.rules.MICROTICKS_PER_TICK / 30;
        let time_interval = world.rules.tick_time_interval();
        let simulation_time_depth = world.rules.tick_time_interval() * 90.0;
        let ball_distance_limit = world.rules.ROBOT_MAX_RADIUS + world.rules.BALL_RADIUS;
        let max_micro_ticks = 1000;
        let mut total_micro_ticks = 0;
        let mut next_action_id = 0;
        let mut optimal_action = OptimalAction {
            id: next_action_id,
            robot_id: self.id,
            action: default_action,
            score: 0,
            history: vec![State::new(&global_simulator)],
            stats: Stats::default(),
            ball_hit_position: None,
        };
        next_action_id += 1;
        let steps = [1, 3, 4, 8];
        let mut iterations = 0;
        while (iterations < 5 || optimal_action.score <= 0) && global_simulator.current_time() + time_interval < simulation_time_depth {
            log!(world.game.current_tick, "[{}] try time point {} {}", self.id, global_simulator.current_micro_tick(), global_simulator.current_time());
            let ball_y = global_simulator.ball().base().y;
            let ball_radius = global_simulator.ball().radius();
            if let Some(distance) = get_min_distance_between_spheres(ball_y, ball_radius, world.rules.ROBOT_MAX_RADIUS) {
                log!(world.game.current_tick, "[{}] use time point {} {} position={:?} velocity={:?} ball_position={:?} ball_velocity={:?}", self.id, global_simulator.current_micro_tick(), global_simulator.current_time(), global_simulator.me().position(), global_simulator.me().velocity(), global_simulator.ball().position(), global_simulator.ball().velocity());
                iterations += 1;
                let points = get_points(
                    distance,
                    global_simulator.ball().base(),
                    global_simulator.me().base(),
                    global_simulator.rules(),
                    rng
                );
                let mut try_target = |point: Vec3| {
                    let target = {
                        let mut robot = global_simulator.me().clone();
                        robot.set_position(point);
                        world.rules.arena.collide(&mut robot);
                        robot.position()
                    };
                    let to_target = target - self.position();
                    let distance_to_target = to_target.norm();
                    let required_speed = if global_simulator.current_time() > 0.0 {
                        distance_to_target / global_simulator.current_time()
                    } else {
                        world.rules.ROBOT_MAX_GROUND_SPEED
                    };
                    let action_id = next_action_id;
                    next_action_id += 1;
                    log!(world.game.current_tick, "[{}] <{}> suggest target {}:{} distance={} speed={} target={:?}", self.id, action_id, global_simulator.current_time(), global_simulator.current_micro_tick(), distance_to_target, required_speed, target);
                    let mut local_simulator = initial_simulator.clone();
                    let mut action = Action::default();
                    let mut stats = Stats::default();
                    let velocity = if distance_to_target > 1e-3 {
                        if distance_to_target > world.rules.ROBOT_MAX_GROUND_SPEED * 20.0 * time_interval {
                            to_target * world.rules.ROBOT_MAX_GROUND_SPEED / distance_to_target
                        } else {
                            to_target * required_speed / distance_to_target
                        }
                    } else {
                        Vec3::default()
                    };
                    let mut history = vec![State::new(&local_simulator)];
                    log!(world.game.current_tick, "[{}] <{}> use velocity {}:{} {} {:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), velocity.norm(), velocity);
                    action.set_target_velocity(velocity);
                    let before_micro_ticks_per_tick = if local_simulator.me().position().distance(local_simulator.ball().position()) > ball_distance_limit + velocity.norm() * time_interval {
                        log!(world.game.current_tick, "[{}] <{}> far", self.id, action_id);
                        far_micro_ticks_per_tick
                    } else {
                        log!(world.game.current_tick, "[{}] <{}> near", self.id, action_id);
                        near_micro_ticks_per_tick
                    };
                    let mut time_to_ball = None;
                    let mut ball_hit_position = None;
                    let mut on_hit_ball = |collision_type: CollisionType, position: Vec3, time: f64| {
                        if collision_type != CollisionType::None && ball_hit_position.is_none() {
                            ball_hit_position = Some(position);
                            time_to_ball = Some(time);
                        }
                    };
                    if local_simulator.me().position().distance(target)
                            > velocity.norm() * time_interval
                        && local_simulator.me().position().distance(local_simulator.ball().position())
                            > ball_distance_limit + velocity.norm() * time_interval
                    {
                        log!(world.game.current_tick, "[{}] <{}> will move {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), velocity.norm() * time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * time_interval);
                        local_simulator.me_mut().action = action.clone();
                        stats.micro_ticks_to_near = local_simulator.current_micro_tick();
                        stats.time_to_near = local_simulator.current_time();
                        while local_simulator.current_time() + time_interval < simulation_time_depth
                            && local_simulator.current_micro_tick() < max_micro_ticks
                            && local_simulator.score() == 0
                            && local_simulator.me().position().distance(target)
                                > velocity.norm() * time_interval
                            && local_simulator.me().position().distance(local_simulator.ball().position())
                                > ball_distance_limit + velocity.norm() * time_interval
                            && local_simulator.me().ball_collision_type() == CollisionType::None
                        {
                            log!(world.game.current_tick, "[{}] <{}> move {}:{} target={}/{} ball={}/{} position={:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), velocity.norm() * time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * time_interval, global_simulator.me().position());
                            local_simulator.tick(time_interval, before_micro_ticks_per_tick, rng);
                            history.push(State::new(&local_simulator));
                            on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
                        }
                        stats.micro_ticks_to_jump = local_simulator.current_micro_tick();
                        stats.time_to_jump = local_simulator.current_time();
                    } else {
                        log!(world.game.current_tick, "[{}] <{}> will jump {}:{} target={}/{} ball={}/{} position={:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), velocity.norm() * time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * time_interval, global_simulator.me().position());
                        action.jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
                    }
                    local_simulator.me_mut().action.jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
                    while local_simulator.current_time() + time_interval < simulation_time_depth
                        && local_simulator.current_micro_tick() < max_micro_ticks
                        && local_simulator.score() == 0
                        && (
                            local_simulator.me().position().distance(target)
                                <= velocity.norm() * time_interval
                            || local_simulator.me().position().distance(local_simulator.ball().position())
                                <= ball_distance_limit + velocity.norm() * time_interval
                            || local_simulator.me().ball_collision_type() == CollisionType::None
                        )
                    {
                        log!(world.game.current_tick, "[{}] <{}> jump {}:{} target={}/{} ball={}/{} position={:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), velocity.norm() * time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * time_interval, global_simulator.me().position());
                        local_simulator.tick(time_interval, before_micro_ticks_per_tick, rng);
                        history.push(State::new(&local_simulator));
                        on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
                        stats.jump_simulation = true;
                    }
                    stats.micro_ticks_to_watch = local_simulator.current_micro_tick();
                    stats.time_to_watch = local_simulator.current_time();
                    local_simulator.me_mut().action.jump_speed = 0.0;
                    local_simulator.me_mut().action.set_target_velocity(Vec3::default());
                    while local_simulator.current_time() + time_interval < simulation_time_depth
                        && local_simulator.current_micro_tick() < max_micro_ticks
                        && local_simulator.score() == 0
                    {
                        log!(world.game.current_tick, "[{}] <{}> watch {}:{} ball_position={:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.ball().position());
                        local_simulator.tick(time_interval, far_micro_ticks_per_tick, rng);
                        history.push(State::new(&local_simulator));
                        on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
                    }
                    if local_simulator.score() != 0 {
                        log!(world.game.current_tick, "[{}] <{}> goal {}:{} score={}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.score());
                    }
                    let action_score = get_action_score(
                        &world.rules,
                        &local_simulator,
                        time_to_ball,
                        simulation_time_depth + time_interval,
                    );
                    stats.micro_ticks_to_end = local_simulator.current_micro_tick();
                    stats.time_to_end = local_simulator.current_time();
                    stats.time_to_score = if local_simulator.score() != 0 {
                        Some(stats.time_to_end)
                    } else {
                        None
                    };
                    stats.score = local_simulator.score();
                    stats.action_score = action_score;
                    log!(world.game.current_tick, "[{}] <{}> suggest action {}:{} score={} speed={}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), action_score, action.target_velocity().norm());
                    if optimal_action.score < action_score {
                        optimal_action = OptimalAction {
                            id: action_id,
                            robot_id: self.id,
                            action,
                            score: action_score,
                            history,
                            stats,
                            ball_hit_position,
                        };
                    }
                    total_micro_ticks += local_simulator.current_micro_tick();
                };
                for point in points {
                    try_target(point);
                }
            }
            for _ in 0..steps[iterations.min(steps.len() - 1)] {
                global_simulator.tick(time_interval, near_micro_ticks_per_tick, rng);
            }
        }
        total_micro_ticks += global_simulator.current_micro_tick();
        if self.does_jump_hit_ball(&world.rules, &world.game.ball) {
            let action_id = next_action_id;
//            next_action_id += 1;
            let mut action = Action::default();
            if self.velocity().norm() > 0.0 {
                action.set_target_velocity(self.velocity().normalized() * world.rules.ROBOT_MAX_GROUND_SPEED);
            }
            action.jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
            let mut stats = Stats::default();
            let mut local_simulator = initial_simulator.clone();
            local_simulator.me_mut().action = action.clone();
            let mut history = vec![State::new(&local_simulator)];
            let mut time_to_ball = None;
            let mut ball_hit_position = None;
            let mut on_hit_ball = |collision_type: CollisionType, position: Vec3, time: f64| {
                if collision_type != CollisionType::None && ball_hit_position.is_none() {
                    ball_hit_position = Some(position);
                    time_to_ball = Some(time);
                }
            };
            log!(world.game.current_tick, "[{}] <{}> jump to ball {}:{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit);
            local_simulator.tick(time_interval, near_micro_ticks_per_tick, rng);
            history.push(State::new(&local_simulator));
            on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
            stats.far_jump_simulation = true;
            while local_simulator.current_time() + time_interval < simulation_time_depth
                && local_simulator.current_micro_tick() < max_micro_ticks
                && local_simulator.score() == 0
                && local_simulator.me().position().distance(local_simulator.ball().position())
                    > ball_distance_limit + local_simulator.me().velocity().norm() * time_interval
            {
                log!(world.game.current_tick, "[{}] <{}> jump far to ball {}:{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + local_simulator.me().velocity().norm() * time_interval);
                local_simulator.tick(time_interval, far_micro_ticks_per_tick, rng);
                history.push(State::new(&local_simulator));
                on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
            }
            while local_simulator.current_time() + time_interval < simulation_time_depth
                && local_simulator.current_micro_tick() < max_micro_ticks
                && local_simulator.score() == 0
                && local_simulator.me().velocity().y().abs() > 0.0
            {
                log!(world.game.current_tick, "[{}] <{}> jump near to ball {}:{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit);
                local_simulator.tick(time_interval, near_micro_ticks_per_tick, rng);
                history.push(State::new(&local_simulator));
                on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
            }
            while local_simulator.current_time() + time_interval < simulation_time_depth
                && local_simulator.current_micro_tick() < max_micro_ticks
                && local_simulator.score() == 0
            {
                log!(world.game.current_tick, "[{}] <{}> watch {}:{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick());
                local_simulator.tick(time_interval, far_micro_ticks_per_tick, rng);
                history.push(State::new(&local_simulator));
                on_hit_ball(local_simulator.me().ball_collision_type(), local_simulator.me().position(), local_simulator.current_time());
            }
            total_micro_ticks += local_simulator.current_micro_tick();
            let action_score = get_action_score(
                &world.rules,
                &local_simulator,
                time_to_ball,
                simulation_time_depth + time_interval,
            );
            stats.micro_ticks_to_end = local_simulator.current_micro_tick();
            stats.time_to_end = local_simulator.current_time();
            stats.time_to_score = if local_simulator.score() != 0 {
                Some(stats.time_to_end)
            } else {
                None
            };
            stats.score = local_simulator.score();
            stats.action_score = action_score;
            log!(world.game.current_tick, "[{}] <{}> suggest action jump {}:{} score={}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), action_score);
            if optimal_action.score < action_score {
                optimal_action = OptimalAction {
                    id: action_id,
                    robot_id: self.id,
                    action,
                    score: action_score,
                    history,
                    stats,
                    ball_hit_position,
                };
            }
        }

        #[cfg(feature = "enable_render")]
        self.render_optimal_action(&optimal_action, &world.rules, render);

        optimal_action.stats.iterations = iterations;
        optimal_action.stats.total_micro_ticks = total_micro_ticks;
        optimal_action
    }

    pub fn does_jump_hit_ball(&self, rules: &Rules, ball: &Ball) -> bool {
        use crate::my_strategy::physics::MoveEquation;

        let get_my_position = {
            let equation = MoveEquation {
                initial_position: self.position(),
                initial_velocity: self.velocity() + Vec3::new(0.0, rules.ROBOT_MAX_JUMP_SPEED, 0.0),
                acceleration: Vec3::new(0.0, -rules.GRAVITY, 0.0),
            };
            move |time| {
                let result = equation.get_position(time);
                result.with_max_y(rules.ROBOT_MIN_RADIUS)
            }
        };
        let get_ball_position = {
            let equation = MoveEquation {
                initial_position: ball.position(),
                initial_velocity: ball.velocity(),
                acceleration: Vec3::new(0.0, -rules.GRAVITY, 0.0),
            };
            move |time| {
                let result = equation.get_position(time);
                result.with_max_y(rules.BALL_RADIUS)
            }
        };
        let get_distance = |time| {
            get_my_position(time).distance(get_ball_position(time))
        };
        let max_time = rules.ROBOT_MAX_JUMP_SPEED / rules.GRAVITY
            * (1.0 - (self.position().distance(rules.arena.get_my_goal_target()) / rules.arena.max_distance()));
        let time = optimize1d(0.0, max_time, 10, get_distance);
        get_distance(time) < 1.05 * rules.BALL_RADIUS + rules.ROBOT_MIN_RADIUS
    }

    #[cfg(feature = "enable_render")]
    pub fn render_optimal_action(&self, optimal_action: &OptimalAction, rules: &Rules, render: &mut Render) {
        use crate::my_strategy::render::{Tag, Object};

        self.render_history(&optimal_action.history, rules, render);
        render.add_with_tag(Tag::RobotId(self.id), Object::text(format!("robot: {}\n  position: {:?}\n  target_speed: {}\n  speed: {}", self.id, self.position(), optimal_action.action.target_velocity().norm(), self.velocity().norm())));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_history(&self, history: &Vec<State>, rules: &Rules, render: &mut Render) {
        use crate::my_strategy::render::{Tag, Object};

        for state in history.iter() {
            render.add_with_tag(Tag::RobotId(self.id), Object::sphere(state.ball.position, rules.BALL_RADIUS, OPTIMAL_BALL_POSITION));
            render.add_with_tag(Tag::RobotId(self.id), Object::sphere(state.me.position, state.me.radius, OPTIMAL_ME_POSITION));
            for (i, robot) in state.robots.iter().enumerate() {
                render.add_with_tag(Tag::RobotId(self.id), Object::sphere(robot.position, robot.radius, get_robot_color(i, state.robots.len())));
            }
        }
        for (prev, next) in (&history[0..history.len() - 1]).iter()
            .zip((&history[1..history.len()]).iter()) {
            render.add_with_tag(Tag::RobotId(self.id), Object::line(prev.ball.position, next.ball.position, 1.0, OPTIMAL_BALL_POSITION));
            render.add_with_tag(Tag::RobotId(self.id), Object::line(prev.me.position, next.me.position, 1.0, OPTIMAL_ME_POSITION));
            for (i, (prev_robot, next_robot)) in (prev.robots.iter().zip(next.robots.iter())).enumerate() {
                render.add_with_tag(Tag::RobotId(self.id), Object::line(prev_robot.position, next_robot.position, 1.0, get_robot_color(i, prev.robots.len())));
            }
        }
    }
}

fn get_action_score(rules: &Rules, simulator: &Simulator, time_to_ball: Option<f64>, max_time: f64) -> i32 {
    let ball = simulator.ball();
    let to_goal = rules.arena.get_goal_target(rules.BALL_RADIUS) - ball.position();
    let ball_goal_distance_score = if simulator.score() == 0 {
        1.0 - to_goal.norm()
            / Vec2::new(rules.arena.width + 2.0 * rules.arena.goal_depth, rules.arena.depth).norm()
    } else if simulator.score() > 0 {
        1.0
    } else {
        0.0
    };
    let ball_goal_direction_score = if ball.velocity().norm() > 0.0 {
        (to_goal.cos(ball.velocity()) + 1.0) / 2.0
    } else {
        0.0
    };
    let time_score = if let Some(v) = time_to_ball {
        if simulator.score() < 0 {
            v / max_time - 1.0
        } else {
            1.0 - v / max_time
        }
    } else {
        0.0
    };
    let score = 0.0
        + ball_goal_distance_score
        + 0.1 * ball_goal_direction_score
        + 0.5 * time_score;
//    log!(current_tick, " <{}> action ball_goal_distance_score={} ball_goal_direction_score={} time_score={}", action_id, ball_goal_distance_score, ball_goal_direction_score, time_score);
    (1000.0 * score).round() as i32
}

pub fn get_points(distance: f64, ball: &Ball, robot: &Robot, rules: &Rules, rng: &mut XorShiftRng) -> Vec<Vec3> {
    let mut result = Vec::new();
    let ball_position = ball.position().with_y(rules.ROBOT_MAX_RADIUS);
    let to_robot = (robot.position() - ball_position).normalized();
    for _ in 0..3 {
        let angle = rng.gen_range(-std::f64::consts::PI, std::f64::consts::PI);
        result.push(ball_position + to_robot.rotated_by_y(angle) * distance);
    }
    result
}

#[cfg(feature = "enable_render")]
fn get_robot_color(i: usize, n: usize) -> Color {
    Color::new(0.8, 0.2 + (i as f64 / n as f64) * 0.8, 0.2, 0.5)
}
