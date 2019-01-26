use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;
#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

pub struct Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub current_tick: i32,
    pub robot_id: i32,
    pub action_id: i32,
    pub simulator: &'r mut Simulator,
    pub rng: &'r mut XorShiftRng,
    pub time_to_ball: &'r mut Option<f64>,
    pub time_to_goal: &'r mut Option<f64>,
    pub get_robot_action_at: G,
    pub actions: &'r mut Vec<Action>,
    #[cfg(feature = "enable_render")]
    pub history: &'r mut Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: &'r mut Stats,
}

impl<'r, 'a, G> Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize) {
        use crate::my_strategy::simulator::RobotCollisionType;

        let current_tick = self.simulator.current_tick();

        for robot in self.simulator.robots_mut().iter_mut() {
            if let Some(action) = (self.get_robot_action_at)(robot.id(), current_tick) {
                *robot.action_mut() = action.clone();
            }
        }

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        if self.simulator.me().collision_type() != RobotCollisionType::None && self.time_to_ball.is_none() {
            *self.time_to_ball = Some(self.simulator.current_time());
        }

        if self.simulator.score() != 0 && self.time_to_goal.is_none() {
            *self.time_to_goal = Some(self.simulator.current_time());
        }

        self.actions.push(self.simulator.me().action().clone());

        #[cfg(feature = "enable_render")]
        self.history.push(self.simulator.clone());
    }
}

pub struct JumpAtPosition {
    pub position: Vec3,
    pub my_max_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_tick: i32,
}

impl JumpAtPosition {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        log!(
            ctx.current_tick, "[{}] <{}> jump at position {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        let before_move = ctx.simulator.current_time();

        let mut action = WalkToPosition {
            target: self.position,
            max_speed: self.my_max_speed,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_tick,
        }.perform(ctx);

        if before_move == ctx.simulator.current_time() {
            log!(
                ctx.current_tick, "[{}] <{}> jump now {}:{} kick_ball_position={} ball={}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().position().distance(self.position),
                ctx.simulator.me().position().distance(ctx.simulator.ball().position())
            );
        }

        action = action.or(Jump {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_tick,
        }.perform(ctx));

        action = action.or(WatchMeJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            use_nitro: false,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_jump,
            max_micro_ticks: self.max_micro_tick,
        }.perform(ctx));

        action = action.or(WatchBallMove {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_tick,
            stop: true,
        }.perform(ctx));

        action
    }
}

pub struct WalkToPosition {
    pub target: Vec3,
    pub max_speed: f64,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl WalkToPosition {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::RobotCollisionType;

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let mut action = None;

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
            && ctx.simulator.me().collision_type() == RobotCollisionType::None {

            let target_velocity = self.get_target_velocity(
                ctx.simulator.me().position(),
                ctx.simulator.me().normal_to_arena(),
                ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED,
            );
            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

            if action.is_none() {
                action = Some(ctx.simulator.me().action().clone());
            }

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

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }

    fn get_target_velocity(&self, position: Vec3, normal: Vec3, max_speed: f64) -> Vec3 {
        use crate::my_strategy::plane::Plane;

        let to_target = Plane::projected(self.target - position, normal);
        if to_target.norm() > 1e-3 {
            to_target.normalized() * self.max_speed.min(max_speed)
        } else {
            Vec3::default()
        }
    }
}

pub struct Jump {
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl Jump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let mut action = None;

        log!(
            ctx.current_tick, "[{}] <{}> jump {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        if ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0 {

            ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;
            let target_velocity = ctx.simulator.rules().arena.projected_at(
                ctx.simulator.ball().position(),
                ctx.simulator.ball().position() - ctx.simulator.me().position()
            ).normalized() * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

            if action.is_none() {
                action = Some(ctx.simulator.me().action().clone());
            }

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);

            log!(
                ctx.current_tick, "[{}] <{}> jump {}:{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
            );
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

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
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.micro_ticks_to_watch = ctx.simulator.current_micro_tick();
            ctx.stats.time_to_watch = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        if self.stop {
            ctx.simulator.me_mut().action_mut().jump_speed = 0.0;
            ctx.simulator.me_mut().action_mut().set_target_velocity(Vec3::default());
        }

        let mut action = None;

        log!(
            ctx.current_tick, "[{}] <{}> watch ball move {}:{} ball_position={:?}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.ball().position()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0 {

            if action.is_none() {
                action = Some(ctx.simulator.me().action().clone());
            }

            log!(
                ctx.current_tick, "[{}] <{}> watch ball move {}:{} ball_position={:?}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.ball().position()
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

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
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        log!(
            ctx.current_tick, "[{}] <{}> jump to ball {}:{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick()
        );

        if !self.does_jump_hit_ball(ctx) {
            return None;
        }

        log!(
            ctx.current_tick, "[{}] <{}> jump now {}:{} ball={}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().position().distance(ctx.simulator.ball().position())
        );

        let action = FarJump {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick_before_jump: self.micro_ticks_per_tick_before_jump,
            micro_ticks_per_tick_after_jump: self.micro_ticks_per_tick_after_jump,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        WatchMeJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            use_nitro: false,
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

    pub fn does_jump_hit_ball<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> bool
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::physics::MoveEquation;
        use crate::my_strategy::optimization::minimize1d;

        let mut simulator = ctx.simulator.clone();
        let mut rng = ctx.rng.clone();

        simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;

        simulator.tick(self.tick_time_interval, self.micro_ticks_per_tick_before_jump, &mut rng);

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.total_micro_ticks += self.micro_ticks_per_tick_before_jump as i32;
        }

        let my_move_equation = MoveEquation::from_robot(simulator.me().base(), simulator.rules());
        let ball_move_equation = MoveEquation::from_ball(simulator.ball().base(), simulator.rules());
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

        let time = minimize1d(0.0, self.max_time, 10, get_distance);

        get_distance(time) < simulator.rules().ROBOT_MAX_RADIUS + simulator.rules().BALL_RADIUS
            && my_move_equation.get_velocity(time).y() > -self.tick_time_interval * simulator.rules().GRAVITY
            && my_move_equation.get_position(time).y() < ball_move_equation.get_position(time).y()
            && ball_move_equation.get_position(time).y() > ball_min_y - self.tick_time_interval * simulator.rules().GRAVITY
    }
}

pub struct FarJump {
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_jump: usize,
    pub micro_ticks_per_tick_after_jump: usize,
    pub max_micro_ticks: i32,
}

impl FarJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Action
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.micro_ticks_to_jump = ctx.simulator.current_micro_tick();
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;

        let velocity = ctx.simulator.me().velocity();
        if velocity.norm() > 0.0 {
            let target_velocity = velocity.normalized() * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
        }

        let action = ctx.simulator.me().action().clone();

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

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }
}

pub struct WatchMeJump {
    pub jump_speed: f64,
    pub use_nitro: bool,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick: usize,
    pub max_micro_ticks: i32,
}

impl WatchMeJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {
        use crate::my_strategy::simulator::{Solid, RobotCollisionType};
        use crate::my_strategy::entity::Entity;

        let stored_action = ctx.simulator.me().action().clone();

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let mut action = None;
        let mut collided_with_ball = false;

        log!(
            ctx.current_tick, "[{}] <{}> watch me jump {}:{} distance_to_arena={}/{}",
            ctx.robot_id, ctx.action_id,
            ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
            ctx.simulator.me().distance_to_arena(), ctx.simulator.me().radius()
        );

        while ctx.simulator.current_time() + self.tick_time_interval < self.max_time
            && ctx.simulator.current_micro_tick() < self.max_micro_ticks
            && ctx.simulator.score() == 0
            && ctx.simulator.me().distance_to_arena() - ctx.simulator.me().radius() > 1e-3
            && !(
                collided_with_ball
                && ctx.simulator.me().collision_type() == RobotCollisionType::None
        ) {
            if !collided_with_ball {
                collided_with_ball = ctx.simulator.me().collision_type() != RobotCollisionType::None;
            }

            ctx.simulator.me_mut().action_mut().jump_speed = self.jump_speed;
            if self.use_nitro && ctx.simulator.me().nitro_amount() > 0.0 {
                let target_velocity = (ctx.simulator.ball().position() - ctx.simulator.me().position())
                    .normalized() * ctx.simulator.rules().MAX_ENTITY_SPEED;
                ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
                ctx.simulator.me_mut().action_mut().use_nitro = true;
            } else {
                ctx.simulator.me_mut().action_mut().use_nitro = false;
            }

            if action.is_none() {
                action = Some(ctx.simulator.me().action().clone());
            }

            log!(
                ctx.current_tick, "[{}] <{}> watch me jump {}:{} distance_to_arena={}/{}",
                ctx.robot_id, ctx.action_id,
                ctx.simulator.current_time(), ctx.simulator.current_micro_tick(),
                ctx.simulator.me().distance_to_arena(), ctx.simulator.me().radius()
            );

            ctx.tick(self.tick_time_interval, self.micro_ticks_per_tick);
        }

        *ctx.simulator.me_mut().action_mut() = stored_action;

        action
    }
}

pub struct ContinueJump {
    pub jump_speed: f64,
    pub use_nitro: bool,
    pub max_time: f64,
    pub tick_time_interval: f64,
    pub micro_ticks_per_tick_before_land: usize,
    pub micro_ticks_per_tick_after_land: usize,
    pub max_micro_ticks: i32,
}

impl ContinueJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Option<Action>
        where G: Fn(i32, i32) -> Option<&'a Action> {

        let mut action = WatchMeJump {
            jump_speed: self.jump_speed,
            use_nitro: self.use_nitro,
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_before_land,
            max_micro_ticks: self.max_micro_ticks,
        }.perform(ctx);

        action = action.or(WatchBallMove {
            max_time: self.max_time,
            tick_time_interval: self.tick_time_interval,
            micro_ticks_per_tick: self.micro_ticks_per_tick_after_land,
            max_micro_ticks: self.max_micro_ticks,
            stop: true,
        }.perform(ctx));

        action
    }
}
