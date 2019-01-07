use crate::model::{Action, Ball};
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
    pub history: Option<&'r mut Vec<State>>,
    pub stats: Option<&'r mut Stats>,
    pub time_to_ball: Option<&'r mut Option<f64>>,
}

impl Context<'_> {
    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize) {
        use crate::my_strategy::simulator::CollisionType;

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        if let Some(time_to_ball) = &mut self.time_to_ball {
            if self.simulator.me().ball_collision_type() != CollisionType::None && time_to_ball.is_none() {
                **time_to_ball = Some(self.simulator.current_time());
            }
        }

        if let Some(history) = &mut self.history {
            history.push(State::new(&self.simulator));
        }
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
            final_speed: self.my_max_speed,
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
    pub final_speed: f64,
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

        ctx.simulator.me_mut().action = Action::default();

        let initial_position = ctx.simulator.me().position();
        let to_target = self.target - initial_position;
        let velocity = if to_target.norm() > 1e-3 {
            to_target.normalized() * self.final_speed
        } else {
            Vec3::default()
        };
        ctx.simulator.me_mut().action.set_target_velocity(velocity);

        let action = ctx.simulator.me().action;

        let max_distance_to_target = self.final_speed * self.tick_time_interval;
        let max_distance_to_ball = ctx.simulator.rules().ball_distance_limit();

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

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();

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

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
            stats.time_to_watch = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();

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

        log!(
            ctx.current_tick, "[{}] <{}> jump to ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        if !self.does_jump_hit_ball(ctx.simulator.clone(), ctx.rng.clone()) {
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

    pub fn does_jump_hit_ball(&self, mut simulator: Simulator, mut rng: XorShiftRng) -> bool {
        use crate::my_strategy::physics::MoveEquation;
        use crate::my_strategy::optimization::optimize1d;

        simulator.me_mut().action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;

        simulator.tick(self.tick_time_interval, self.micro_ticks_per_tick_before_jump, &mut rng);

        let my_move_equation = MoveEquation::from_entity(simulator.me(), simulator.rules());
        let ball_move_equation = MoveEquation::from_entity(simulator.ball(), simulator.rules());
        let my_min_y = simulator.rules().ROBOT_MIN_RADIUS;
        let ball_min_y = simulator.rules().BALL_RADIUS;

        let get_my_position = |time| {
            my_move_equation.get_position(time).with_max_y(my_min_y)
        };

        let get_ball_position = |time| {
            ball_move_equation.get_position(time).with_max_y(ball_min_y)
        };

        let get_distance = |time| {
            get_my_position(time).distance(get_ball_position(time))
        };

        let time = optimize1d(0.0, self.max_time, 10, get_distance);

        get_distance(time) < simulator.rules().ROBOT_MAX_RADIUS + simulator.rules().BALL_RADIUS
            && my_move_equation.get_velocity(time).y() > -self.tick_time_interval * simulator.rules().GRAVITY
            && my_move_equation.get_position(time).y() < ball_move_equation.get_position(time).y()
            && ball_move_equation.get_position(time).y() > ball_min_y - self.tick_time_interval * simulator.rules().GRAVITY
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

        if let Some(stats) = &mut ctx.stats {
            stats.far_jump_simulation = true;
            stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();

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

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
            stats.time_to_watch = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();

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

pub struct DoNothing {
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl DoNothing {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> do nothing {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0 {

            log!(
                ctx.current_tick, "[{}] <{}> do nothing {}:{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct PassBallToPosition<'r> {
    pub ball: &'r Ball,
    pub ball_target: Vec3,
    pub kick_ball_time: f64,
    pub max_time: f64,
    pub max_iter: usize,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_ticks: i32,
}

impl PassBallToPosition<'_> {
    pub fn perform(&self, ctx: &mut Context) -> Option<Action> {
        use crate::my_strategy::physics::get_min_distance_between_spheres;
        use crate::my_strategy::kick::get_optimal_kick;
        use crate::my_strategy::simulator::CollisionType;
        use crate::my_strategy::optimization::optimize1d;
        use crate::my_strategy::entity::Entity;

        log!(
            ctx.current_tick, "[{}] <{}> pass ball to position {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        if self.ball.y > ctx.simulator.rules().max_robot_jump_height() {
            return None;
        }

        let kick = get_optimal_kick(
            self.ball_target,
            self.kick_ball_time,
            self.ball,
            ctx.simulator.me().base(),
            ctx.simulator.rules(),
            self.max_time,
            self.max_iter,
        );

        let before_move = ctx.simulator.current_time();

        let mut action = MoveMeToPosition {
            target: kick.jump_target.position,
            final_speed: kick.params.final_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

//        let get_velocity_distance_after_kick = |jump_speed: f64| {
//            use crate::my_strategy::physics::MoveEquation;
//
//            let mut jump_simulator = ctx.simulator.clone();
//            let mut rng = ctx.rng.clone();
//            let mut jump_ctx = Context {
//                current_tick: ctx.current_tick,
//                robot_id: ctx.robot_id,
//                action_id: ctx.action_id,
//                simulator: &mut jump_simulator,
//                rng: &mut rng,
//                history: None,
//                stats: None,
//                time_to_ball: None,
//            };
//
//            JumpUntil {
//                jump_speed,
//                max_time: self.max_time,
//                tick_time_interval: self.tick_time_interval,
//                micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
//                max_micro_ticks: self.max_micro_ticks,
//                predicate: |simulator| simulator.me().ball_collision_type() == CollisionType::None,
//            }.perform(&mut jump_ctx);
//
//            JumpWhile {
//                jump_speed,
//                max_time: self.max_time,
//                tick_time_interval: self.tick_time_interval,
//                micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
//                max_micro_ticks: self.max_micro_ticks,
//                predicate: |simulator| simulator.me().ball_collision_type() != CollisionType::None,
//            }.perform(&mut jump_ctx);
//
//            let move_equation = MoveEquation::from_entity(
//                jump_ctx.simulator.ball(),
//                jump_ctx.simulator.rules()
//            );
//
//            move_equation.get_closest_possible_distance_to_target(
//                self.ball_target,
//                jump_ctx.simulator.rules().BALL_RADIUS,
//                self.max_time,
//                10,
//            )
//        };
//
//        let jump_speed = optimize1d(
//            0.0,
//            ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
//            10,
//            get_velocity_distance_after_kick
//        );

        let jump_speed = kick.params.jump_speed;

        if before_move == ctx.simulator.current_time() {
            action.jump_speed = jump_speed;
        }

        JumpUntil {
            jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
            predicate: |simulator| simulator.me().ball_collision_type() == CollisionType::None,
        }.perform(ctx);

        if ctx.simulator.me().ball_collision_type() == CollisionType::None {
            return None;
        }

        JumpWhile {
            jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_ticks,
            predicate: |simulator| simulator.me().ball_collision_type() != CollisionType::None,
        }.perform(ctx);

        WatchBallMoveToPosition {
            target: self.ball_target,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        Some(action)
    }
}

pub struct JumpUntil<F>
    where F: Fn(&Simulator) -> bool {

    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
    pub predicate: F,
}

impl<F> JumpUntil<F>
    where F: Fn(&Simulator) -> bool {

    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::CollisionType;

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();
        ctx.simulator.me_mut().action.jump_speed = self.jump_speed;
        ctx.simulator.me_mut().action.set_target_velocity(stored_action.target_velocity());

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> jump until collide with ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

        log!(
            ctx.current_tick, "[{}] <{}> jump until collide with ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && (self.predicate)(&ctx.simulator) {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            log!(
                ctx.current_tick, "[{}] <{}> jump until collide with ball {}:{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
            );
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct JumpWhile<F>
    where F: Fn(&Simulator) -> bool {

    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
    pub predicate: F,
}

impl<F> JumpWhile<F>
    where F: Fn(&Simulator) -> bool {

    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::CollisionType;

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action = Action::default();
        ctx.simulator.me_mut().action.jump_speed = self.jump_speed;
        ctx.simulator.me_mut().action.set_target_velocity(stored_action.target_velocity());

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> jump until collide with ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && (self.predicate)(&ctx.simulator) {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            log!(
                ctx.current_tick, "[{}] <{}> jump until collide with ball {}:{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
            );
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}

pub struct KickBall {
    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

pub struct WatchBallMoveToPosition {
    pub target: Vec3,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl WatchBallMoveToPosition {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        if let Some(stats) = &mut ctx.stats {
            stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
            stats.time_to_watch = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action;

        ctx.simulator.me_mut().action.jump_speed = 0.0;
        ctx.simulator.me_mut().action.set_target_velocity(Vec3::default());

        let action = ctx.simulator.me().action;

        log!(
            ctx.current_tick, "[{}] <{}> watch ball move to position {}:{} ball_position={:?}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.ball().position()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && ctx.simulator.ball().position().distance(self.target) > 1.0 {

            log!(
                ctx.current_tick, "[{}] <{}> watch {}:{} distance={}/{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.ball().position().distance(self.target), 1.0
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        ctx.simulator.me_mut().action = stored_action;

        action
    }
}