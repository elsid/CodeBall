use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;
#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

pub const MAX_TICKS: i32 = 100;
pub const NEAR_MICRO_TICKS_PER_TICK: usize = 25;
pub const FAR_MICRO_TICKS_PER_TICK: usize = 3;

pub struct Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub current_tick: i32,
    pub robot_id: i32,
    pub order_id: i32,
    pub simulator: &'r mut Simulator,
    pub rng: &'r mut XorShiftRng,
    pub time_to_ball: &'r mut Option<f64>,
    pub time_to_goal: &'r mut Option<f64>,
    pub get_robot_action_at: G,
    pub actions: &'r mut Vec<Action>,
    pub near_micro_ticks_per_tick: usize,
    pub far_micro_ticks_per_tick: usize,
    pub used_path_micro_ticks: &'r mut usize,
    pub max_path_micro_ticks: usize,
    #[cfg(feature = "enable_render")]
    pub history: &'r mut Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: &'r mut Stats,
}

#[derive(Copy, Clone)]
pub enum TickType {
    Near,
    Far,
}

pub enum Error {
    Goal,
    TicksLimit,
    MicroTicksLimit,
    BadCondition,
}

pub type Result = std::result::Result<(), Error>;

impl<'r, 'a, G> Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub fn tick(&mut self, tick_type: TickType) -> Result {
        use crate::my_strategy::simulator::RobotCollisionType;

        if self.simulator.score() != 0 {
            return Err(Error::Goal);
        }

        if self.simulator.current_tick() >= MAX_TICKS {
            return Err(Error::TicksLimit);
        }

        if *self.used_path_micro_ticks >= self.max_path_micro_ticks {
            return Err(Error::MicroTicksLimit);
        }

        let current_tick = self.simulator.current_tick();

        for robot in self.simulator.robots_mut().iter_mut() {
            if let Some(action) = (self.get_robot_action_at)(robot.id(), current_tick) {
                *robot.action_mut() = action.clone();
            }
        }

        let micro_ticks_per_tick = match tick_type {
            TickType::Near => self.near_micro_ticks_per_tick,
            TickType::Far => self.far_micro_ticks_per_tick,
        };

        let time_interval = self.simulator.rules().tick_time_interval();

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        *self.used_path_micro_ticks += micro_ticks_per_tick;

        if self.simulator.me().collision_type() != RobotCollisionType::None && self.time_to_ball.is_none() {
            *self.time_to_ball = Some(self.simulator.current_time());
        }

        if self.simulator.score() != 0 && self.time_to_goal.is_none() {
            *self.time_to_goal = Some(self.simulator.current_time());
        }

        self.actions.push(self.simulator.me().action().clone());

        #[cfg(feature = "enable_render")]
        self.history.push(self.simulator.clone());

        #[cfg(feature = "enable_stats")]
        {
            self.stats.reached_scenario_limit = *self.used_path_micro_ticks >= self.max_path_micro_ticks;
            self.stats.scenario_micro_ticks = *self.used_path_micro_ticks;
            self.stats.time_to_end = self.simulator.current_time();
            self.stats.time_to_score = if self.simulator.score() != self.stats.game_score {
                Some(self.simulator.current_time())
            } else {
                None
            };
            self.stats.game_score = self.simulator.score();

            if micro_ticks_per_tick == NEAR_MICRO_TICKS_PER_TICK {
                self.stats.ticks_with_near_micro_ticks += 1;
            } else {
                self.stats.ticks_with_far_micro_ticks += 1;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct JumpAtPosition {
    pub position: Vec3,
    pub my_max_speed: f64,
}

impl JumpAtPosition {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        log!(
            ctx.current_tick, "[{}] <{}> jump at position {}:{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks
        );

        MoveToArena {
        }.perform(ctx)?;

        WalkToPosition {
            target: self.position,
            max_speed: self.my_max_speed,
        }.perform(ctx)?;

        Jump {
        }.perform(ctx)?;

        WatchMeJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            allow_nitro: false,
        }.perform(ctx)?;

        WatchBallMove {
        }.perform(ctx)?;

        Ok(())
    }

    pub fn opposite(&self) -> Self {
        JumpAtPosition {
            position: self.position.opposite(),
            my_max_speed: self.my_max_speed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalkToPosition {
    pub target: Vec3,
    pub max_speed: f64,
}

impl WalkToPosition {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::RobotCollisionType;

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let max_distance_to_target = self.max_speed * ctx.simulator.rules().tick_time_interval();
        let max_distance_to_ball = ctx.simulator.rules().ball_distance_limit()
            + self.max_speed * ctx.simulator.rules().tick_time_interval();

        log!(
            ctx.current_tick, "[{}] <{}> move to position {}:{} target={}/{} ball={}/{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(self.target), max_distance_to_target,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            max_distance_to_ball
        );

        while ctx.simulator.me().position().distance(self.target)
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

            ctx.tick(TickType::Far)?;

            log!(
                ctx.current_tick, "[{}] <{}> move {}:{} target={}/{} ball={}/{}",
                ctx.robot_id, ctx.order_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.me().position().distance(self.target), max_distance_to_target,
                ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
                max_distance_to_ball
            );
        }

        Ok(())
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

#[derive(Debug, Clone)]
pub struct Jump {
}

impl Jump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();

        log!(
            ctx.current_tick, "[{}] <{}> jump {}:{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks
        );

        ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;
        let target_velocity = ctx.simulator.rules().arena.projected_at(
            ctx.simulator.ball().position(),
            ctx.simulator.ball().position() - ctx.simulator.me().position()
        ).normalized() * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
        ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

        ctx.tick(TickType::Near)?;

        log!(
            ctx.current_tick, "[{}] <{}> jump {}:{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks
        );

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WatchBallMove {
}

impl WatchBallMove {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.time_to_watch = ctx.simulator.current_time();
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();

        log!(
            ctx.current_tick, "[{}] <{}> watch ball move {}:{} ball_position={:?}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.ball().position()
        );

        loop {
            log!(
                ctx.current_tick, "[{}] <{}> watch ball move {}:{} ball_position={:?}",
                ctx.robot_id, ctx.order_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.ball().position()
            );

            ctx.tick(TickType::Far)?;
        }
    }
}

pub struct JumpToBall {
    pub allow_nitro: bool,
}

impl JumpToBall {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        log!(
            ctx.current_tick, "[{}] <{}> jump to ball {}:{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks
        );

        FarJump {
            allow_nitro: self.allow_nitro,
        }.perform(ctx)?;

        WatchMeJump {
            jump_speed: ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED,
            allow_nitro: self.allow_nitro,
        }.perform(ctx)?;

        WatchBallMove {
        }.perform(ctx)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FarJump {
    pub allow_nitro: bool,
}

impl FarJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        if !does_jump_hit_ball(ctx) {
            return Err(Error::BadCondition);
        }

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();

        ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;

        if self.allow_nitro && ctx.simulator.me().nitro_amount() > 0.0 {
            let target_velocity = (ctx.simulator.ball().position() - ctx.simulator.me().position())
                .normalized() * ctx.simulator.rules().MAX_ENTITY_SPEED;
            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
            ctx.simulator.me_mut().action_mut().use_nitro = true;
        } else {
            let velocity = ctx.simulator.me().velocity();
            if velocity.norm() > 0.0 {
                let target_velocity = velocity.normalized() * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
                ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
            }
            ctx.simulator.me_mut().action_mut().use_nitro = false;
        }

        log!(
            ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval()
        );

        ctx.tick(TickType::Near)?;

        log!(
            ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval()
        );

        while ctx.simulator.me().position().distance(ctx.simulator.ball().position())
                > ctx.simulator.rules().ball_distance_limit()
                    + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval() {

            ctx.tick(TickType::Far)?;

            log!(
                ctx.current_tick, "[{}] <{}> far jump {}:{} ball={}/{}",
                ctx.robot_id, ctx.order_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
                ctx.simulator.rules().ball_distance_limit()
                    + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval()
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WatchMeJump {
    pub jump_speed: f64,
    pub allow_nitro: bool,
}

impl WatchMeJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {
        use crate::my_strategy::simulator::{Solid, RobotCollisionType};
        use crate::my_strategy::entity::Entity;

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let mut collided_with_ball = false;

        log!(
            ctx.current_tick, "[{}] <{}> watch me jump {}:{} distance_to_arena={}/{}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().distance_to_arena(), ctx.simulator.me().radius()
        );

        while ctx.simulator.me().distance_to_arena() - ctx.simulator.me().radius() > 1e-3
            && !(
                collided_with_ball
                && ctx.simulator.me().collision_type() == RobotCollisionType::None
        ) {
            if !collided_with_ball {
                collided_with_ball = ctx.simulator.me().collision_type() != RobotCollisionType::None;
            }

            ctx.simulator.me_mut().action_mut().jump_speed = self.jump_speed;
            if self.allow_nitro && ctx.simulator.me().nitro_amount() > 0.0 {
                let target_velocity = (ctx.simulator.ball().position() - ctx.simulator.me().position())
                    .normalized() * ctx.simulator.rules().MAX_ENTITY_SPEED;
                ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
                ctx.simulator.me_mut().action_mut().use_nitro = true;
            } else {
                ctx.simulator.me_mut().action_mut().use_nitro = false;
            }

            log!(
                ctx.current_tick, "[{}] <{}> watch me jump {}:{} distance_to_arena={}/{}",
                ctx.robot_id, ctx.order_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.me().distance_to_arena(), ctx.simulator.me().radius()
            );

            ctx.tick(TickType::Near)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ContinueJump {
    pub jump_speed: f64,
    pub allow_nitro: bool,
}

impl ContinueJump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        WatchMeJump {
            jump_speed: self.jump_speed,
            allow_nitro: self.allow_nitro,
        }.perform(ctx)?;

        WatchBallMove {
        }.perform(ctx)?;

        Ok(())
    }
}

pub fn does_jump_hit_ball<'r, 'a, G>(ctx: &mut Context<'r, 'a, G>) -> bool
    where G: Fn(i32, i32) -> Option<&'a Action> {

    use crate::my_strategy::physics::MoveEquation;
    use crate::my_strategy::optimization::minimize1d;

    let mut simulator = ctx.simulator.clone();
    let mut rng = ctx.rng.clone();

    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;

    simulator.tick(ctx.simulator.rules().tick_time_interval(), ctx.near_micro_ticks_per_tick, &mut rng);

    *ctx.used_path_micro_ticks += ctx.near_micro_ticks_per_tick;

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

    let time = minimize1d(
        0.0,
        MAX_TICKS as f64 * simulator.rules().tick_time_interval(),
        10,
        get_distance
    );

    get_distance(time) < simulator.rules().ROBOT_MAX_RADIUS + simulator.rules().BALL_RADIUS
        && my_move_equation.get_velocity(time).y() > -simulator.rules().tick_time_interval() * simulator.rules().GRAVITY
        && my_move_equation.get_position(time).y() < ball_move_equation.get_position(time).y()
        && ball_move_equation.get_position(time).y() > ball_min_y - simulator.rules().tick_time_interval() * simulator.rules().GRAVITY
}

#[derive(Debug, Clone)]
pub struct MoveToArena {}

impl MoveToArena {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        *ctx.simulator.me_mut().action_mut() = Action::default();

        log!(
            ctx.current_tick, "[{}] <{}> go down {}:{} distance_to_arena={}",
            ctx.robot_id, ctx.order_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().distance_to_arena()
        );

        while !ctx.simulator.me().base().touch {
            ctx.simulator.me_mut().action_mut().jump_speed = 0.0;
            let target_velocity = -ctx.simulator.me().normal_to_arena()
                * ctx.simulator.rules().ROBOT_MAX_GROUND_SPEED;
            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);
            ctx.simulator.me_mut().action_mut().use_nitro = true;

            log!(
                ctx.current_tick, "[{}] <{}> go down {}:{} distance_to_arena={}",
                ctx.robot_id, ctx.order_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.me().distance_to_arena()
            );

            ctx.tick(TickType::Far)?;
        }

        Ok(())
    }
}
