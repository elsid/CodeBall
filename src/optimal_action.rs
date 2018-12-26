use crate::model::{Robot, Action, Ball, Rules};
use crate::my_strategy::random::Rng;
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::common::{Square, IsBetween};
use crate::my_strategy::simulator::Solid;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::render::{Render, Color, Object, Tag};

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
    pub action_score: i32,
}

pub struct OptimalAction {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    pub history: Vec<State>,
    pub stats: Stats,
}

const OPTIMAL_ME_POSITION: Color = Color::new(0.0, 0.8, 0.4, 0.5);
const OPTIMAL_BALL_POSITION: Color = Color::new(0.0, 0.4, 0.8, 0.5);

impl Robot {
    pub fn get_optimal_action(&self, world: &World, rng: &mut XorShiftRng, render: &mut Render) -> OptimalAction {
//        log!(world.game.current_tick, "generate actions {}", self.id);
        let initial_simulator = Simulator::new(world, self.id);
        let mut global_simulator = initial_simulator.clone();
        global_simulator.me_mut().set_velocity(Vec3::default());
        let default_action = Action::default();
        let near_micro_ticks_per_tick = world.rules.MICROTICKS_PER_TICK / 2;
        let far_micro_ticks_per_tick = near_micro_ticks_per_tick / 10;
        let near_time_interval = world.rules.tick_time_interval();
        let far_time_interval = near_time_interval * 2.0;
        let simulation_time_depth = world.rules.tick_time_interval() * 90.0;
        let ball_distance_limit = world.rules.ROBOT_MAX_RADIUS + world.rules.BALL_RADIUS;
        let max_micro_ticks = 1000;
        let mut next_action_id = 0;
        let mut optimal_action = OptimalAction {
            id: next_action_id,
            robot_id: self.id,
            action: default_action,
            score: std::i32::MIN,
            history: vec![State::new(&global_simulator)],
            stats: Stats::default(),
        };
        next_action_id += 1;
        let steps = [1, 3, 4, 8];
        let mut iterations = 0;
        while (iterations < 3 || optimal_action.id == 0) && global_simulator.current_time() + near_time_interval < simulation_time_depth {
            log!(world.game.current_tick, "[{}]  try time point {} {}", self.id, global_simulator.current_micro_tick(), global_simulator.current_time());
            let ball_y = global_simulator.ball().base().y;
            let ball_radius = global_simulator.ball().radius();
            if let Some(distance) = get_min_distance_between_spheres(ball_y, ball_radius, world.rules.ROBOT_MAX_RADIUS) {
                log!(world.game.current_tick, "[{}]  use time point {} {}", self.id, global_simulator.current_micro_tick(), global_simulator.current_time());
                iterations += 1;
                let points = get_points(
                    distance,
                    global_simulator.ball().base(),
                    global_simulator.me().base(),
                    global_simulator.rules(),
                    rng
                );
                for point in points {
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
                    log!(world.game.current_tick, "[{}]    suggest target {}:{} distance={} speed={} target={:?}", self.id, global_simulator.current_time(), global_simulator.current_micro_tick(), distance_to_target, required_speed, target);
                    if required_speed.is_between(0.9 * world.rules.ROBOT_MAX_GROUND_SPEED, 1.01 * world.rules.ROBOT_MAX_GROUND_SPEED) {
                        continue;
                    }
                    let action_id = next_action_id;
                    next_action_id += 1;
                    let mut local_simulator = initial_simulator.clone();
                    let mut action = Action::default();
                    let mut stats = Stats::default();
                    let velocity = if distance_to_target > 1e-3 {
                        if distance_to_target > world.rules.ROBOT_MAX_GROUND_SPEED * far_time_interval {
                            to_target * world.rules.ROBOT_MAX_GROUND_SPEED / distance_to_target
                        } else {
                            to_target * required_speed / distance_to_target
                        }
                    } else {
                        Vec3::default()
                    };
                    let mut history = vec![State::new(&local_simulator)];
                    log!(world.game.current_tick, "[{}]    <{}> use velocity {}:{} {} {:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), velocity.norm(), velocity);
                    action.set_target_velocity(velocity);
                    if local_simulator.me().position().distance(target)
                            > velocity.norm() * near_time_interval
                        && local_simulator.me().position().distance(local_simulator.ball().position())
                            > ball_distance_limit + velocity.norm() * near_time_interval
                    {
                        log!(world.game.current_tick, "[{}]    <{}> will move {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), 1.5 * velocity.norm() * near_time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * near_time_interval);
                        local_simulator.me_mut().action = action.clone();
                        while local_simulator.current_time() + far_time_interval < simulation_time_depth
                            && local_simulator.current_micro_tick() < max_micro_ticks
                            && local_simulator.score() == 0
                            && local_simulator.me().position().distance(target)
                                > 1.5 * velocity.norm() * far_time_interval
                            && local_simulator.me().position().distance(local_simulator.ball().position())
                                > ball_distance_limit + velocity.norm() * far_time_interval
                        {
                            log!(world.game.current_tick, "[{}]    <{}> move far {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), 1.5 * velocity.norm() * far_time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * far_time_interval);
                            local_simulator.tick(far_time_interval, far_micro_ticks_per_tick, rng);
                            history.push(State::new(&local_simulator));
                        }
                        stats.micro_ticks_to_near = local_simulator.current_micro_tick();
                        stats.time_to_near = local_simulator.current_time();
                        while local_simulator.current_time() + near_time_interval < simulation_time_depth
                            && local_simulator.current_micro_tick() < max_micro_ticks
                            && local_simulator.score() == 0
                            && local_simulator.me().position().distance(target)
                                > velocity.norm() * near_time_interval
                            && local_simulator.me().position().distance(local_simulator.ball().position())
                                > ball_distance_limit + velocity.norm() * near_time_interval
                        {
                            log!(world.game.current_tick, "[{}]    <{}> move near {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), 1.5 * velocity.norm() * near_time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * near_time_interval);
                            local_simulator.tick(near_time_interval, near_micro_ticks_per_tick, rng);
                            history.push(State::new(&local_simulator));
                        }
                        stats.micro_ticks_to_jump = local_simulator.current_micro_tick();
                        stats.time_to_jump = local_simulator.current_time();
                    } else {
                        log!(world.game.current_tick, "[{}]    <{}> will jump {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), 1.5 * velocity.norm() * near_time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * near_time_interval);
                        action.jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
                    }
                    local_simulator.me_mut().action.jump_speed = world.rules.ROBOT_MAX_JUMP_SPEED;
                    let time_to_ball = local_simulator.current_time();
                    while local_simulator.current_time() + near_time_interval < simulation_time_depth
                        && local_simulator.current_micro_tick() < max_micro_ticks
                        && local_simulator.score() == 0
                        && (
                            local_simulator.me().position().distance(target)
                                <= velocity.norm() * near_time_interval
                            || local_simulator.me().position().distance(local_simulator.ball().position())
                                <= ball_distance_limit + velocity.norm() * near_time_interval
                        )
                    {
                        log!(world.game.current_tick, "[{}]    <{}> jump {}:{} target={}/{} ball={}/{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), local_simulator.me().position().distance(target), 1.5 * velocity.norm() * near_time_interval, local_simulator.me().position().distance(local_simulator.ball().position()), ball_distance_limit + velocity.norm() * near_time_interval);
                        local_simulator.tick(near_time_interval, near_micro_ticks_per_tick, rng);
                        history.push(State::new(&local_simulator));
                        stats.jump_simulation = true;
                    }
                    stats.micro_ticks_to_watch = local_simulator.current_micro_tick();
                    stats.time_to_watch = local_simulator.current_time();
                    local_simulator.me_mut().action.jump_speed = 0.0;
                    local_simulator.me_mut().action.set_target_velocity(Vec3::default());
                    while local_simulator.current_time() + far_time_interval < simulation_time_depth
                        && local_simulator.current_micro_tick() < max_micro_ticks
                        && local_simulator.score() == 0
                    {
                        log!(world.game.current_tick, "[{}]    <{}> watch {}:{}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick());
                        local_simulator.tick(far_time_interval, far_micro_ticks_per_tick, rng);
                        history.push(State::new(&local_simulator));
                    }
                    let action_score = get_action_score(
                        &world.rules,
                        &local_simulator,
                        time_to_ball,
                        simulation_time_depth + far_time_interval,
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
                    log!(world.game.current_tick, "[{}]    <{}> suggest action {}:{} score={} speed={}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), action_score, action.target_velocity().norm());
                    if optimal_action.score < action_score {
                        optimal_action = OptimalAction {
                            id: action_id,
                            robot_id: self.id,
                            action,
                            score: action_score,
                            history,
                            stats,
                        };
                    }
                }
            }
            for _ in 0..steps[iterations.min(steps.len() - 1)] {
                global_simulator.tick(near_time_interval, near_micro_ticks_per_tick, rng);
            }
        }
        if cfg!(feature = "enable_render") {
            for state in optimal_action.history.iter() {
                render.add_with_tag(Tag::RobotId(self.id), Object::sphere(state.ball.position, world.rules.BALL_RADIUS, OPTIMAL_BALL_POSITION));
                render.add_with_tag(Tag::RobotId(self.id), Object::sphere(state.me.position, state.me.radius, OPTIMAL_ME_POSITION));
                for (i, robot) in state.robots.iter().enumerate() {
                    render.add_with_tag(Tag::RobotId(self.id), Object::sphere(robot.position, robot.radius, get_robot_color(i, state.robots.len())));
                }
            }
            for (prev, next) in (&optimal_action.history[0..optimal_action.history.len() - 1]).iter()
                .zip((&optimal_action.history[1..optimal_action.history.len()]).iter()) {
                render.add_with_tag(Tag::RobotId(self.id), Object::line(prev.ball.position, next.ball.position, 1.0, OPTIMAL_BALL_POSITION));
                render.add_with_tag(Tag::RobotId(self.id), Object::line(prev.me.position, next.me.position, 1.0, OPTIMAL_ME_POSITION));
                for (i, (prev_robot, next_robot)) in (prev.robots.iter().zip(next.robots.iter())).enumerate() {
                    render.add_with_tag(Tag::RobotId(self.id), Object::line(prev_robot.position, next_robot.position, 1.0, get_robot_color(i, prev.robots.len())));
                }
            }
            render.add_with_tag(Tag::RobotId(self.id), Object::text(format!("robot: {}\n  position: {:?}\n  target_speed: {}\n  velocity.norm(): {}", self.id, self.position(), optimal_action.action.target_velocity().norm(), self.velocity().norm())));
        }
        optimal_action.stats.iterations = iterations;
        optimal_action
    }
}

fn get_action_score(rules: &Rules, simulator: &Simulator, time_to_ball: f64, max_time: f64) -> i32 {
    let ball = simulator.ball();
    let to_goal = rules.arena.get_goal_target() - ball.position();
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
    let time_score = if simulator.score() < 0 {
        time_to_ball / max_time - 1.0
    } else {
        1.0 - time_to_ball / max_time
    };
    let score = 0.0
        + ball_goal_distance_score
        + 0.1 * ball_goal_direction_score
        + 0.5 * time_score;
//    log!(current_tick, "    <{}> action ball_goal_distance_score={} ball_goal_direction_score={} time_score={}", action_id, ball_goal_distance_score, ball_goal_direction_score, time_score);
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
//    let to_robot = (robot.position() - ball.position().with_y(rules.ROBOT_MIN_RADIUS)).normalized();
//    result.push(ball.position().with_y(rules.ROBOT_MIN_RADIUS) + to_robot * distance);
//    let to_defend = (rules.arena.get_defend_target() - ball.position().with_y(rules.ROBOT_MIN_RADIUS)).normalized();
//    result.push(ball.position().with_y(rules.ROBOT_MIN_RADIUS) + to_defend * distance);
//    result.push(get_position_to_jump(distance, ball, robot, rules));
    result
}

pub fn get_min_distance_between_spheres(ball_y: f64, ball_radius: f64, robot_radius: f64) -> Option<f64> {
    let a = (ball_radius + robot_radius).square();
    let b = (ball_y - robot_radius).square();
    if a >= b {
        Some((a - b).sqrt())
    } else {
        None
    }
}

//fn get_position_to_jump(distance: f64, ball: &Ball, robot: &Robot, rules: &Rules) -> Vec3 {
//    let goal_target = rules.arena.get_goal_target();
//    let to_goal = goal_target - ball.position();
//    let to_goal_direction = to_goal.normalized();
//    let desired_ball_velocity = to_goal_direction * rules.ROBOT_MAX_JUMP_SPEED;
//    let desired_robot_hit_direction = (desired_ball_velocity - ball.velocity()).with_y(0.0).normalized();
//    (ball.position() - desired_robot_hit_direction * (ball.radius + distance))
//        .with_y(rules.ROBOT_MIN_RADIUS)
//}

fn get_robot_color(i: usize, n: usize) -> Color {
    Color::new(0.8, 0.2 + (i as f64 / n as f64) * 0.8, 0.2, 0.5)
}
