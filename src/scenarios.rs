use crate::model::{Action, Ball, Robot, Rules};
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::history::{State, Stats};

pub struct Context<'r> {
    pub current_tick: i32,
    pub robot_id: i32,
    pub action_id: i32,
    pub simulator: &'r mut Simulator,
    pub rng: &'r mut XorShiftRng,
    pub history: &'r mut Vec<State>,
    pub stats: &'r mut Stats,
    pub time_to_ball: &'r mut Option<f64>,
}

impl Context<'_> {
    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize) {
        use crate::my_strategy::simulator::CollisionType;

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        if self.simulator.me().ball_collision_type() != CollisionType::None && self.time_to_ball.is_none() {
            *self.time_to_ball = Some(self.simulator.current_time());
        }

        self.history.push(State::new(&self.simulator));
    }
}

pub struct JumpAtPosition<'r> {
    pub ball: &'r Ball,
    pub kick_ball_position: Vec3,
    pub my_max_speed: f64,
    pub my_jump_speed: f64,
    pub ball_target: Vec3,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_ticks: i32,
}

impl JumpAtPosition<'_> {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        log!(
            ctx.current_tick, "[{}] <{}> jump at position {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        let before_move = ctx.simulator.current_time();

        let mut action = MoveMeToPosition {
            target: self.kick_ball_position,
            max_speed: self.my_max_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        if before_move == ctx.simulator.current_time() {
            action.jump_speed = self.my_jump_speed;

            log!(
                ctx.current_tick, "[{}] <{}> jump now {}:{} kick_ball_position={} ball={}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().position().distance(self.kick_ball_position),
                ctx.simulator.me().position().distance(ctx.simulator.ball().position())
            );
        }

        Jump {
            target: self.kick_ball_position,
            max_speed: self.my_max_speed,
            jump_speed: self.my_jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        WatchBallMove {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_ticks,
            stop: true,
        }.perform(ctx);

        action
    }
}

pub struct MoveMeToPosition {
    pub target: Vec3,
    pub max_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl MoveMeToPosition {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::CollisionType;

        let stored_action = ctx.simulator.me().action;

        let initial_position = ctx.simulator.me().position();
        let to_target = self.target - initial_position;
        let velocity = if to_target.norm() > 1e-3 {
            to_target.normalized() * self.max_speed
        } else {
            Vec3::default()
        };
        ctx.simulator.me_mut().action.set_target_velocity(velocity);

        let action = ctx.simulator.me().action;

        let max_distance_to_target = self.max_speed * self.tick_time_interval;
        let max_distance_to_ball = ctx.simulator.rules().ball_distance_limit()
            + self.max_speed * self.tick_time_interval;

        log!(
            ctx.current_tick, "[{}] <{}> move to position {}:{} target={}/{} ball={}/{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(self.target), max_distance_to_target,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            max_distance_to_ball
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && ctx.simulator.me().position().distance(self.target)
                > max_distance_to_target
            && ctx.simulator.me().position().distance(ctx.simulator.ball().position())
                > max_distance_to_ball
            && ctx.simulator.me().ball_collision_type() == CollisionType::None {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            log!(
                ctx.current_tick, "[{}] <{}> move {}:{} target={}/{} ball={}/{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().position().distance(self.target), max_distance_to_target,
                ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
                max_distance_to_ball
            );
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct Jump {
    pub target: Vec3,
    pub max_speed: f64,
    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl Jump {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::CollisionType;

        ctx.stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
        ctx.stats.time_to_jump = ctx.simulator.current_time();

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action.jump_speed = self.jump_speed;
        ctx.simulator.me_mut().action.set_target_velocity(stored_action.target_velocity());

        let action = ctx.simulator.me().action;

        let min_distance_to_target = self.max_speed * self.tick_time_interval;
        let min_distance_to_ball = ctx.simulator.rules().ball_distance_limit()
            + min_distance_to_target;

        log!(
            ctx.current_tick, "[{}] <{}> jump {}:{} target={}/{} ball={}/{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(self.target), min_distance_to_target,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            min_distance_to_ball
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && (
                ctx.simulator.me().position().distance(self.target)
                    <= min_distance_to_target
                || ctx.simulator.me().position().distance(ctx.simulator.ball().position())
                    <= min_distance_to_ball
                || ctx.simulator.me().ball_collision_type() == CollisionType::None
            ) {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            log!(
                ctx.current_tick, "[{}] <{}> jump {}:{} target={}/{} ball={}/{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().position().distance(self.target), min_distance_to_target,
                ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
                min_distance_to_ball
            );
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct WatchBallMove {
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
    pub stop: bool,
}

impl WatchBallMove {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        ctx.stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
        ctx.stats.time_to_watch = ctx.simulator.current_time();

        let stored_action = ctx.simulator.me().action;

        if self.stop {
            ctx.simulator.me_mut().action.jump_speed = 0.0;
            ctx.simulator.me_mut().action.set_target_velocity(Vec3::default());
        }

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> watch ball move {}:{} ball_position={:?}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.ball().position()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0 {

            log!(
                ctx.current_tick, "[{}] <{}> watch {}:{} ball_position={:?}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.ball().position()
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct JumpToBall {
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_ticks: i32,
}

impl JumpToBall {
    pub fn perform(&self, ctx: &mut Context) -> Option<Action> {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::optimization::optimize1d;
        use crate::my_strategy::physics::MoveEquation;

        log!(
            ctx.current_tick, "[{}] <{}> jump to ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        if !ctx.simulator.me().base().does_jump_hit_ball(
            ctx.simulator.rules(),
            ctx.simulator.ball().base()
        ) {
            return None;
        }

        log!(
            ctx.current_tick, "[{}] <{}> jump now {}:{} ball={}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(ctx.simulator.ball().position())
        );

        let action = FarJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick_before_jump: self.micro_ticks_per_tick_before_jump,
            micro_ticks_per_tick_after_jump: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        WatchMeJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        WatchBallMove {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_ticks,
            stop: false,
        }.perform(ctx);

        Some(action)
    }
}

pub struct FarJump {
    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_ticks: i32,
}

impl FarJump {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        ctx.stats.far_jump_simulation = true;
        ctx.stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
        ctx.stats.time_to_jump = ctx.simulator.current_time();

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action.jump_speed = self.jump_speed;

        let velocity = ctx.simulator.me().velocity();
        if velocity.norm() > 0.0 {
            let target_velocity = velocity.normalized() * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
            ctx.simulator.me_mut().action.set_target_velocity(target_velocity);
        }

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * self.tick_time_interval
        );

        ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick_before_jump);

        log!(
            ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * self.tick_time_interval
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && ctx.simulator.me().position().distance(ctx.simulator.ball().position())
                > ctx.simulator.rules().ball_distance_limit()
                    + ctx.simulator.me().velocity().norm() * self.tick_time_interval {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick_after_jump);

            log!(
                ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
                ctx.simulator.rules().ball_distance_limit()
                    + ctx.simulator.me().velocity().norm() * self.tick_time_interval
            );
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct WatchMeJump {
    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl WatchMeJump {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        ctx.stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
        ctx.stats.time_to_watch = ctx.simulator.current_time();

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action.jump_speed = self.jump_speed;

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> watch me move {}:{} velocity_y={}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().velocity().y().abs()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && ctx.simulator.me().velocity().y().abs() > 0.0 {

            log!(
                ctx.current_tick, "[{}] <{}> watch me move {}:{} velocity_y={}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().velocity().y().abs()
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

impl Robot {
    pub fn does_jump_hit_ball(&self, rules: &Rules, ball: &Ball) -> bool {
        use crate::my_strategy::physics::MoveEquation;
        use crate::my_strategy::optimization::optimize1d;

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
}
