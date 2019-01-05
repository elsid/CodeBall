use crate::model::{Robot, Action, Ball, Rules};
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec2::Vec2;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::simulator::Solid;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::render::Render;
#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Color;
use crate::my_strategy::history::{Stats, State};

pub struct OptimalAction {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    pub history: Vec<State>,
    pub stats: Stats,
}

impl Robot {
    pub fn get_optimal_action(&self, world: &World, rng: &mut XorShiftRng, render: &mut Render) -> Option<OptimalAction> {
        use crate::my_strategy::physics::get_min_distance_between_spheres;
        use crate::my_strategy::scenarios::{Context, JumpAtPosition, JumpToBall, DoNothing};

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
        let near_micro_ticks_per_tick = world.rules.MICROTICKS_PER_TICK / 4;
        let far_micro_ticks_per_tick = world.rules.MICROTICKS_PER_TICK / 30;
        let time_interval = world.rules.tick_time_interval();
        let simulation_time_depth = world.rules.tick_time_interval() * 90.0;
        let ball_distance_limit = world.rules.ROBOT_MAX_RADIUS + world.rules.BALL_RADIUS;
        let max_micro_ticks = 1000;
        let mut total_micro_ticks = 0;
        let mut next_action_id = 0;
        let mut optimal_action: Option<OptimalAction> = None;
        let steps = [1, 3, 4, 8];
        let mut iterations = 0;
        while (iterations < 5 || optimal_action.is_none()) && global_simulator.current_time() + time_interval < simulation_time_depth {
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
                        if distance_to_target > world.rules.ROBOT_MAX_GROUND_SPEED * 20.0 * time_interval {
                            world.rules.ROBOT_MAX_GROUND_SPEED
                        } else {
                            distance_to_target / global_simulator.current_time()
                        }
                    } else {
                        world.rules.ROBOT_MAX_GROUND_SPEED
                    };
                    let action_id = next_action_id;
                    next_action_id += 1;
                    log!(world.game.current_tick, "[{}] <{}> suggest target {}:{} distance={} speed={} target={:?}", self.id, action_id, global_simulator.current_time(), global_simulator.current_micro_tick(), distance_to_target, required_speed, target);
                    let mut local_simulator = initial_simulator.clone();
                    let mut stats = Stats::default();
                    let velocity = if distance_to_target > 1e-3 {
                        to_target.normalized() * required_speed
                    } else {
                        Vec3::default()
                    };
                    let mut history = vec![State::new(&local_simulator)];
                    log!(world.game.current_tick, "[{}] <{}> use velocity {}:{} {} {:?}", self.id, action_id, local_simulator.current_time(), local_simulator.current_micro_tick(), velocity.norm(), velocity);
                    let before_micro_ticks_per_tick = if local_simulator.me().position().distance(local_simulator.ball().position()) > ball_distance_limit + velocity.norm() * time_interval {
                        log!(world.game.current_tick, "[{}] <{}> far", self.id, action_id);
                        far_micro_ticks_per_tick
                    } else {
                        log!(world.game.current_tick, "[{}] <{}> near", self.id, action_id);
                        near_micro_ticks_per_tick
                    };
                    let mut time_to_ball = None;
                    let mut ctx = Context {
                        current_tick: world.game.current_tick,
                        robot_id: self.id,
                        action_id,
                        simulator: &mut local_simulator,
                        rng,
                        history: &mut history,
                        stats: &mut stats,
                        time_to_ball: &mut time_to_ball,
                    };
                    let action = JumpAtPosition {
                        ball: global_simulator.ball().base(),
                        kick_ball_position: target,
                        my_max_speed: required_speed,
                        my_jump_speed: world.rules.ROBOT_MAX_JUMP_SPEED,
                        ball_target: world.rules.arena.get_goal_target(world.rules.BALL_RADIUS),
                        max_time: simulation_time_depth,
                        tick_time_interval: time_interval,
                        micro_ticks_per_tick_before_jump: before_micro_ticks_per_tick,
                        micro_ticks_per_tick_after_jump: far_micro_ticks_per_tick,
                        max_micro_ticks,
                    }.perform(&mut ctx);
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
                    if optimal_action.is_none() || optimal_action.as_ref().unwrap().score < action_score {
                        optimal_action = Some(OptimalAction {
                            id: action_id,
                            robot_id: self.id,
                            action,
                            score: action_score,
                            history,
                            stats,
                        });
                    }
                    total_micro_ticks += local_simulator.current_micro_tick();
                }
            }
            for _ in 0..steps[iterations.min(steps.len() - 1)] {
                global_simulator.tick(time_interval, near_micro_ticks_per_tick, rng);
            }
        }
        total_micro_ticks += global_simulator.current_micro_tick();

        let action_id = next_action_id;
        next_action_id += 1;
        let mut local_simulator = initial_simulator.clone();
        let mut history = vec![State::new(&local_simulator)];
        let mut stats = Stats::default();
        let mut time_to_ball = None;

        let mut ctx = Context {
            current_tick: world.game.current_tick,
            robot_id: self.id,
            action_id,
            simulator: &mut local_simulator,
            rng,
            history: &mut history,
            stats: &mut stats,
            time_to_ball: &mut time_to_ball,
        };

        let action = JumpToBall {
            max_time: simulation_time_depth,
            tick_time_interval: time_interval,
            micro_ticks_per_tick_before_jump: near_micro_ticks_per_tick,
            micro_ticks_per_tick_after_jump: far_micro_ticks_per_tick,
            max_micro_ticks,
        }.perform(&mut ctx);

        total_micro_ticks += local_simulator.current_micro_tick();

        if let Some(v) = action {
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

            log!(
                world.game.current_tick, "[{}] <{}> suggest action far jump {}:{} score={}",
                self.id, action_id,
                local_simulator.current_time(), local_simulator.current_micro_tick(), action_score
            );

            if optimal_action.is_none() || optimal_action.as_ref().unwrap().score < action_score {
                optimal_action = Some(OptimalAction {
                    id: action_id,
                    robot_id: self.id,
                    action: v,
                    score: action_score,
                    history,
                    stats,
                });
            }
        }

        if optimal_action.is_none() || optimal_action.as_ref().unwrap().score < 0 {
            let action_id = next_action_id;
            next_action_id += 1;
            let mut local_simulator = initial_simulator.clone();
            let mut history = vec![State::new(&local_simulator)];
            let mut stats = Stats::default();
            let mut time_to_ball = None;

            let mut ctx = Context {
                current_tick: world.game.current_tick,
                robot_id: self.id,
                action_id,
                simulator: &mut local_simulator,
                rng,
                history: &mut history,
                stats: &mut stats,
                time_to_ball: &mut time_to_ball,
            };

            let action = DoNothing {
                max_time: simulation_time_depth,
                tick_time_interval: time_interval,
                micro_ticks_per_tick: far_micro_ticks_per_tick,
                max_micro_ticks,
            }.perform(&mut ctx);

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

            if optimal_action.is_none() || optimal_action.as_ref().unwrap().score < action_score {
                optimal_action = Some(OptimalAction {
                    id: action_id,
                    robot_id: self.id,
                    action,
                    score: action_score,
                    history,
                    stats,
                });
            }
        }

        if let Some(v) = &mut optimal_action {
            #[cfg(feature = "enable_render")]
                self.render_optimal_action(v, &world.rules, render);

            v.stats.iterations = iterations;
            v.stats.total_micro_ticks = total_micro_ticks;
        }

        optimal_action
    }

    #[cfg(feature = "enable_render")]
    pub fn render_optimal_action(&self, optimal_action: &OptimalAction, rules: &Rules, render: &mut Render) {
        use crate::my_strategy::render::{Tag, Object};

        self.render_history(&optimal_action.history, rules, render);
        render.add_with_tag(Tag::RobotId(self.id), Object::text(
            format!(
                "robot: {}\n  position: {:?}\n  speed: {}\n  target_speed: {}\n  jump: {}",
                self.id, self.position(), self.velocity().norm(),
                optimal_action.action.target_velocity().norm(),
                optimal_action.action.jump_speed
            )
        ));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_history(&self, history: &Vec<State>, rules: &Rules, render: &mut Render) {
        use crate::my_strategy::render::{Tag, Object};

        if history.is_empty() {
            return;
        }

        let max_time = history.last().unwrap().time;

        for state in history.iter() {
            let time = state.time / if max_time == 0.0 { 1.0 } else { max_time };
            render.add_with_tag(
                Tag::RobotId(self.id),
                Object::sphere(state.ball.position, rules.BALL_RADIUS, get_ball_color(time))
            );
            render.add_with_tag(
                Tag::RobotId(self.id),
                Object::sphere(state.me.position, state.me.radius, get_my_color(time))
            );
            for (i, robot) in state.robots.iter().enumerate() {
                render.add_with_tag(
                    Tag::RobotId(self.id),
                    Object::sphere(robot.position, robot.radius, get_robot_color(i, state.robots.len(), time))
                );
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
        2.0
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
    use crate::my_strategy::random::Rng;

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
fn get_ball_color(time: f64) -> Color {
    Color::new(0.0, time, 0.8, 0.5)
}

#[cfg(feature = "enable_render")]
fn get_my_color(time: f64) -> Color {
    Color::new(0.0, 0.8, time, 0.5)
}

#[cfg(feature = "enable_render")]
fn get_robot_color(i: usize, n: usize, time: f64) -> Color {
    Color::new(0.8, 0.2 + (i as f64 / n as f64) * 0.8, time, 0.5)
}
