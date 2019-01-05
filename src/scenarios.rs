use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

#[derive(Default)]
pub struct Output {
    pub time_to_ball: Option<f64>,
    pub hit_ball_position: Option<Vec3>,
    pub min_distance_to_ball: Option<f64>,
}

pub struct Context<'r> {
    pub current_tick: i32,
    pub robot_id: i32,
    pub action_id: i32,
    pub simulator: &'r mut Simulator,
    pub rng: &'r mut XorShiftRng,
    pub output: &'r mut Output,
    #[cfg(feature = "enable_render")]
    pub history: Option<&'r mut Vec<Simulator>>,
    #[cfg(feature = "enable_stats")]
    pub stats: Option<&'r mut Stats>,
}

impl Context<'_> {
    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize) {
        use crate::my_strategy::simulator::CollisionType;
        use crate::my_strategy::entity::Entity;

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        self.update_min_distance_to_ball();

        if self.simulator.me().ball_collision_type() != CollisionType::None
            && self.output.time_to_ball.is_none() {

            self.output.time_to_ball = Some(self.simulator.current_time());
            self.output.hit_ball_position = Some(self.simulator.me().position());
        }

        #[cfg(feature = "enable_render")]
        {
            if let Some(history) = &mut self.history {
                history.push(self.simulator.clone());
            }
        }
    }

    pub fn update_min_distance_to_ball(&mut self) {
        use crate::my_strategy::entity::Entity;

        let distance = self.simulator.me().position()
            .distance(self.simulator.ball().position());

        self.output.min_distance_to_ball = Some(
            self.output.min_distance_to_ball
                .map_or(distance, |v| v.min(distance))
        );
    }
}

macro_rules! ctx_log {
    ($ctx:expr, $max_time:expr, $scenario:tt) => {
        if $ctx.action_id >= 0 {
            verbose_log!(
                $ctx.current_tick, "[{}] <{}> {} {}:{}/{}",
                $ctx.robot_id, $ctx.action_id, $scenario,
                $ctx.simulator.current_micro_tick(), $ctx.simulator.current_time(), $max_time
            );
        }
    };

    ($ctx:expr, $max_time:expr, $scenario:tt, $message:tt) => {
        if $ctx.action_id >= 0 {
            verbose_log!(
                $ctx.current_tick, "[{}] <{}> {} {}:{}/{} {}",
                $ctx.robot_id, $ctx.action_id, $scenario,
                $ctx.simulator.current_micro_tick(), $ctx.simulator.current_time(), $max_time,
                $message
            );
        }
    };

    ($ctx:expr, $max_time:expr, $scenario:tt, $format:tt, $($value:expr),*) => {
        ctx_log!($ctx, $max_time, $scenario, { format!($format, $($value),*) });
    };
}

pub struct JumpAtPosition {
    pub jump_position: Vec3,
    pub speed_when_jump: f64,
    pub jump_speed: f64,
    pub time_to_jump: f64,
    pub ball_target: Vec3,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
}

impl JumpAtPosition {
    pub fn perform(&self, ctx: &mut Context) -> Option<Action> {
        use crate::my_strategy::simulator::CollisionType;

        ctx_log!(ctx, self.max_time, "jump at position");

        let before_move = ctx.simulator.current_time();

        let mut action = WalkToPosition {
            target: self.jump_position,
            final_speed: self.speed_when_jump,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
        }.perform(ctx);

        if before_move == ctx.simulator.current_time() {
            action.jump_speed = self.jump_speed;
            ctx_log!(ctx, self.max_time, "jump now");
        }

        JumpUntil {
            jump_speed: self.jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            predicate: |simulator| simulator.me().ball_collision_type() == CollisionType::None,
        }.perform(ctx);

        if ctx.simulator.me().ball_collision_type() == CollisionType::None {
            return None;
        }

        JumpWhile {
            jump_speed: self.jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            predicate: |simulator| simulator.me().ball_collision_type() != CollisionType::None,
        }.perform(ctx);

        WatchBallMoveToPosition {
            target: self.ball_target,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
        }.perform(ctx);

        Some(action)
    }
}

pub struct WalkToPosition {
    pub target: Vec3,
    pub final_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
}

impl WalkToPosition {
    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let target_velocity = Self::get_target_velocity(
            self.target,
            self.final_speed,
            ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED,
            ctx.simulator.rules().ROBOT_ACCELERATION,
            self.tick_time_interval,
            self.max_time - ctx.simulator.current_time(),
            ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED,
            ctx.simulator.me().position(),
            ctx.simulator.me().velocity(),
            ctx.simulator.me().normal_to_arena(),
        );

        ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

        let action = ctx.simulator.me().action().clone();

        let max_distance_to_target = self.final_speed * self.tick_time_interval;

        ctx_log!(
            ctx, self.max_time, "walk to position", "target={}/{} target_speed={} speed={}",
            ctx.simulator.me().position().distance(self.target), max_distance_to_target,
            target_velocity.norm(), ctx.simulator.me().speed()
        );

        while ctx.simulator.current_time() < self.max_time + self.tick_time_interval
            && ctx.simulator.me().position().distance(self.target)
                > max_distance_to_target {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            let target_velocity = Self::get_target_velocity(
                self.target,
                self.final_speed,
                ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED,
                ctx.simulator.rules().ROBOT_ACCELERATION,
                self.tick_time_interval,
                self.max_time - ctx.simulator.current_time(),
                ctx.simulator.me().action().target_velocity().norm(),
                ctx.simulator.me().position(),
                ctx.simulator.me().velocity(),
                ctx.simulator.me().normal_to_arena(),
            );

            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

            ctx_log!(
                ctx, self.max_time, "walk to position", "target={}/{} target_speed={} speed={}",
                ctx.simulator.me().position().distance(self.target), max_distance_to_target,
                target_velocity.norm(), ctx.simulator.me().speed()
            );
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }

    pub fn get_target_velocity(final_position: Vec3, final_speed: f64, max_speed: f64,
                               acceleration: f64, time_interval: f64, time_left: f64,
                               prev_target_speed: f64, position: Vec3, velocity: Vec3,
                               normal: Vec3) -> Vec3 {
        use crate::my_strategy::plane::Plane;

        let to_target = Plane::projected(final_position - position, normal);
        if to_target.norm() < std::f64::EPSILON {
            return velocity.normalized() * final_speed;
        }
        let time_left_to_position = to_target.norm() / velocity.norm();
        let approximate_time_left = time_left.min(time_left_to_position) - time_interval;
        let target_speed = if velocity.norm() - final_speed < acceleration * approximate_time_left {
            max_speed.min(prev_target_speed)
        } else {
            final_speed
        };
        to_target.normalized() * target_speed
    }
}

pub struct JumpToBall {
    pub jump_speed: f64,
    pub ball_target: Vec3,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
}

impl JumpToBall {
    pub fn perform(&self, ctx: &mut Context) -> Option<Action> {
        use crate::my_strategy::simulator::CollisionType;

        ctx_log!(ctx, self.max_time, "jump to ball");

        let before_move = ctx.simulator.current_time();

        let action = JumpUntil {
            jump_speed: self.jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            predicate: |simulator| simulator.me().ball_collision_type() == CollisionType::None,
        }.perform(ctx);

        if ctx.simulator.me().ball_collision_type() == CollisionType::None {
            return None;
        }

        JumpWhile {
            jump_speed: self.jump_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            predicate: |simulator| simulator.me().ball_collision_type() != CollisionType::None,
        }.perform(ctx);

        WatchBallMoveToPosition {
            target: self.ball_target,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
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
    pub predicate: F,
}

impl<F> JumpUntil<F>
    where F: Fn(&Simulator) -> bool {

    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();
        ctx.simulator.me_mut().action_mut().jump_speed = self.jump_speed;
        ctx.simulator.me_mut().action_mut().set_target_velocity(stored_action.target_velocity());

        let action = ctx.simulator.me().action().clone();

        ctx_log!(
            ctx, self.max_time, "jump until", "distance_to_ball={}",
            ctx.simulator.me().position().distance(ctx.simulator.ball().position())
        );

        ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

        ctx_log!(
            ctx, self.max_time, "jump until", "distance_to_ball={}",
            ctx.simulator.me().position().distance(ctx.simulator.ball().position())
        );

        while ctx.simulator.current_time() < self.max_time + self.tick_time_interval
            && (self.predicate)(&ctx.simulator) {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            ctx_log!(
                ctx, self.max_time, "jump until", "distance_to_ball={}",
                ctx.simulator.me().position().distance(ctx.simulator.ball().position())
            );
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }
}

pub struct JumpWhile<F>
    where F: Fn(&Simulator) -> bool {

    pub jump_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub predicate: F,
}

impl<F> JumpWhile<F>
    where F: Fn(&Simulator) -> bool {

    pub fn perform(&self, ctx: &mut Context) -> Action {
        use crate::my_strategy::entity::Entity;

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();
        ctx.simulator.me_mut().action_mut().jump_speed = self.jump_speed;
        ctx.simulator.me_mut().action_mut().set_target_velocity(stored_action.target_velocity());

        let action = ctx.simulator.me().action().clone();

        ctx_log!(
            ctx, self.max_time, "jump while", "distance_to_ball={}",
            ctx.simulator.me().position().distance(ctx.simulator.ball().position())
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && (self.predicate)(&ctx.simulator) {

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            ctx_log!(
                ctx, self.max_time, "jump while", "distance_to_ball={}",
                ctx.simulator.me().position().distance(ctx.simulator.ball().position())
            );
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }
}

pub struct WatchBallMoveToPosition {
    pub target: Vec3,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
}

impl WatchBallMoveToPosition {
    pub fn perform(&self, ctx: &mut Context) {
        use crate::my_strategy::entity::Entity;

        ctx_log!(
            ctx, self.max_time, "watch ball move to position", "distance={}/{}",
            ctx.simulator.ball().position().distance(self.target),
            ctx.simulator.ball().velocity().norm() * self.tick_time_interval
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.score() == 0
            && ctx.simulator.ball().velocity().norm() > std::f64::EPSILON
            && ctx.simulator.ball().position().distance(self.target)
                > ctx.simulator.ball().velocity().norm() * self.tick_time_interval {

            ctx_log!(
                ctx, self.max_time, "watch ball move to position", "distance={}/{}",
                ctx.simulator.ball().position().distance(self.target),
                ctx.simulator.ball().velocity().norm() * self.tick_time_interval
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }
    }
}
