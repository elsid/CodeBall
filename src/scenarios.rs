use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::config::Config;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

pub struct Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub first: bool,
    pub current_tick: i32,
    pub robot_id: i32,
    pub order_id: i32,
    pub state_id: i32,
    pub simulator: &'r mut Simulator,
    pub rng: &'r mut XorShiftRng,
    pub my_time_to_ball: &'r mut Option<f64>,
    pub opponent_time_to_ball: &'r mut Option<f64>,
    pub time_to_goal: &'r mut Option<f64>,
    pub get_robot_action_at: G,
    pub actions: &'r mut Vec<Action>,
    pub near_micro_ticks_per_tick: usize,
    pub far_micro_ticks_per_tick: usize,
    pub used_path_micro_ticks: &'r mut usize,
    pub max_path_micro_ticks: usize,
    pub config: &'r Config,
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

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Goal,
    TicksLimit,
    MicroTicksLimit,
    BadCondition,
}

const GOAL: usize = 1 << 0;
const TICK_LIMIT: usize = 1 << 1;
const MICRO_TICKS_LIMIT: usize = 1 << 2;
const LIMITS: usize = TICK_LIMIT | MICRO_TICKS_LIMIT;
const ALL: usize = GOAL | LIMITS;

pub type Result = std::result::Result<(), Error>;

impl<'r, 'a, G> Context<'r, 'a, G>
    where G: Fn(i32, i32) -> Option<&'a Action> {

    pub fn tick(&mut self, tick_type: TickType, checks: usize) -> Result {
        if checks & GOAL != 0 && self.simulator.score() != 0 {
            return Err(Error::Goal);
        }

        if checks & TICK_LIMIT != 0 && self.simulator.current_tick() >= self.config.max_ticks {
            return Err(Error::TicksLimit);
        }

        if checks & MICRO_TICKS_LIMIT != 0 && *self.used_path_micro_ticks >= self.max_path_micro_ticks {
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

        if self.first {
            self.first = false;
            self.update();
        }

        self.simulator.tick(time_interval, micro_ticks_per_tick, self.rng);

        self.update();

        *self.used_path_micro_ticks += micro_ticks_per_tick;

        if !self.simulator.ignore_me() {
            self.actions.push(self.simulator.me().action().clone());
        }

        #[cfg(feature = "enable_render")]
        self.history.push(self.simulator.clone());

        #[cfg(feature = "enable_stats")]
        {
            self.stats.reached_path_limit = *self.used_path_micro_ticks >= self.max_path_micro_ticks;
            self.stats.path_micro_ticks = *self.used_path_micro_ticks;

            if micro_ticks_per_tick == self.config.near_micro_ticks_per_tick {
                self.stats.ticks_with_near_micro_ticks += 1;
            } else {
                self.stats.ticks_with_far_micro_ticks += 1;
            }
        }

        Ok(())
    }

    fn update(&mut self) {
        use crate::my_strategy::simulator::RobotCollisionType;

        if self.my_time_to_ball.is_none() {
            *self.opponent_time_to_ball = self.simulator.robots().iter()
                .find(|v| {
                    !v.is_teammate() && v.collision_type() != RobotCollisionType::None
                })
                .map(|_| self.simulator.current_time());
        }

        if self.simulator.score() != 0 && self.time_to_goal.is_none() {
            *self.time_to_goal = Some(self.simulator.current_time());
        }

        if !self.simulator.ignore_me()
            && self.simulator.me().collision_type() != RobotCollisionType::None
            && self.my_time_to_ball.is_none() {
            *self.my_time_to_ball = Some(self.simulator.current_time());
        }

        #[cfg(feature = "enable_stats")]
        {
            self.stats.time_to_end = self.simulator.current_time();
            self.stats.time_to_score = if self.simulator.score() != self.stats.game_score {
                Some(self.simulator.current_time())
            } else {
                None
            };
            self.stats.game_score = self.simulator.score();
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
            ctx.current_tick, "[{}] <{}> <{}> move to position {}:{} target={}/{} ball={}/{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
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

            ctx.tick(TickType::Far, ALL)?;

            log!(
                ctx.current_tick, "[{}] <{}> <{}> move {}:{} target={}/{} ball={}/{}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
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
    pub allow_nitro: bool,
}

impl Jump {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();

        log!(
            ctx.current_tick, "[{}] <{}> <{}> jump {}:{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks
        );

        ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;
        let use_nitro = self.allow_nitro && ctx.simulator.me().nitro_amount() > 0.0;
        ctx.simulator.me_mut().action_mut().use_nitro = use_nitro;
        let target_velocity = get_target_velocity_for_jump(use_nitro, &ctx.simulator);
        ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

        ctx.tick(TickType::Near, ALL)?;

        log!(
            ctx.current_tick, "[{}] <{}> <{}> jump {}:{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
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
            ctx.current_tick, "[{}] <{}> <{}> watch ball move {}:{} ball_position={:?}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.ball().position()
        );

        loop {
            log!(
                ctx.current_tick, "[{}] <{}> <{}> watch ball move {}:{} ball_position={:?}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.ball().position()
            );

            ctx.tick(TickType::Far, ALL)?;
        }
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

        if !does_jump_hit_ball(self.allow_nitro, ctx) {
            return Err(Error::BadCondition);
        }

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.time_to_jump = ctx.simulator.current_time();
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();

        ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;
        let use_nitro = self.allow_nitro && ctx.simulator.me().nitro_amount() > 0.0;
        ctx.simulator.me_mut().action_mut().use_nitro = use_nitro;
        let target_velocity = get_target_velocity_for_jump(use_nitro, &ctx.simulator);
        ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

        log!(
            ctx.current_tick, "[{}] <{}> <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval()
        );

        ctx.tick(TickType::Near, ALL)?;

        log!(
            ctx.current_tick, "[{}] <{}> <{}> far jump {}:{} ball={}/{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(ctx.simulator.ball().position()),
            ctx.simulator.rules().ball_distance_limit()
                + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval()
        );

        while ctx.simulator.me().position().distance(ctx.simulator.ball().position())
                > ctx.simulator.rules().ball_distance_limit()
                    + ctx.simulator.me().velocity().norm() * ctx.simulator.rules().tick_time_interval() {

            ctx.tick(TickType::Far, ALL)?;

            log!(
                ctx.current_tick, "[{}] <{}> <{}> far jump {}:{} ball={}/{}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
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
            ctx.current_tick, "[{}] <{}> <{}> watch me jump {}:{} distance_to_arena={}/{}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
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
                ctx.current_tick, "[{}] <{}> <{}> watch me jump {}:{} distance_to_arena={}/{}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.me().distance_to_arena(), ctx.simulator.me().radius()
            );

            ctx.tick(TickType::Near, ALL)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Observe {
    pub number: usize,
    pub wait_until: f64,
    pub max_ball_z: f64,
}

impl Observe {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::RobotCollisionType;

        if self.number >= ctx.config.max_observations {
            return Err(Error::BadCondition);
        }

        *ctx.simulator.me_mut().action_mut() = Action::default();
        ctx.simulator.set_ignore_me(true);

        let step = ctx.config.ticks_per_steps[self.number.min(ctx.config.ticks_per_steps.len() - 1)];

        #[cfg(feature = "enable_stats")]
        {
            ctx.stats.current_step = step;
        }

        log!(
            ctx.current_tick, "[{}] <{}> <{}> observe {}:{} ball_position={:?}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.ball().position()
        );

        let mut first = true;

        loop {
            let rules = ctx.simulator.rules();
            let ball_position = ctx.simulator.ball().position();
            let (distance, normal) = rules.arena.distance_and_normal(ball_position);
            let opponent_collided = ctx.simulator.robots().iter()
                .find(|v| {
                    !v.is_teammate() && v.collision_type() != RobotCollisionType::None
                })
                .is_some();

            if (self.number == 0 || !first)
                && ctx.simulator.current_time() >= self.wait_until
                && ball_position.z() < self.max_ball_z
                && (
                    ball_position.y() < rules.max_robot_jump_height() || (
                        distance < rules.max_robot_jump_height()
                        && ball_position.y() < rules.max_robot_wall_walk_height()
                        && Vec3::j().cos(normal) >= 0.0
                    )
                    || (
                        opponent_collided
                        && ball_position.y() > rules.BALL_RADIUS
                    )
            ) {
                break;
            }

            first = false;

            log!(
                ctx.current_tick, "[{}] <{}> <{}> observe {}:{} ball_position={:?}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                ctx.simulator.ball().position()
            );

            for _ in 0..step {
                ctx.tick(TickType::Near, LIMITS)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PushRobot {
    pub robot_id: i32,
    pub allow_nitro: bool,
    pub until_time: f64,
}

impl PushRobot {
    pub fn perform<'r, 'a, G>(&self, ctx: &mut Context<'r, 'a, G>) -> Result
        where G: Fn(i32, i32) -> Option<&'a Action> {
        use crate::my_strategy::entity::Entity;

        *ctx.simulator.me_mut().action_mut() = Action::default();

        let robot = ctx.simulator.robots().iter()
            .find(|v| v.id() == self.robot_id)
            .unwrap();

        log!(
            ctx.current_tick, "[{}] <{}> <{}> push robot {}:{} distance_to_robot={}",
            ctx.robot_id, ctx.order_id, ctx.state_id,
            ctx.simulator.current_time(), ctx.used_path_micro_ticks,
            ctx.simulator.me().position().distance(robot.position())
        );

        while ctx.simulator.current_time() < self.until_time {
            let robot = ctx.simulator.robots().iter()
                .find(|v| v.id() == self.robot_id)
                .unwrap();

            let to_robot = robot.position() - ctx.simulator.me().position();
            let target_velocity = to_robot.normalized() * ctx.simulator.rules().MAX_ENTITY_SPEED;

            ctx.simulator.me_mut().action_mut().set_target_velocity(target_velocity);

            let use_nitro = self.allow_nitro
                && ctx.simulator.me().nitro_amount() > 0.0
                && to_robot.norm() < 2.5 * ctx.simulator.rules().ROBOT_RADIUS;

            ctx.simulator.me_mut().action_mut().use_nitro = use_nitro;

            if ctx.simulator.rules().is_flying(ctx.simulator.me().base()) {
                ctx.simulator.me_mut().action_mut().jump_speed = ctx.simulator.rules().ROBOT_MAX_JUMP_SPEED;
            }

            log!(
                ctx.current_tick, "[{}] <{}> <{}> push robot {}:{} distance_to_robot={}",
                ctx.robot_id, ctx.order_id, ctx.state_id,
                ctx.simulator.current_time(), ctx.used_path_micro_ticks,
                to_robot.norm()
            );

            ctx.tick(TickType::Far, LIMITS)?;
        }

        Ok(())
    }
}

pub fn get_target_velocity_for_jump(use_nitro: bool, simulator: &Simulator) -> Vec3 {

    use crate::my_strategy::entity::Entity;

    if use_nitro {
        (simulator.ball().position() - simulator.me().position())
            .normalized() * simulator.rules().MAX_ENTITY_SPEED
    } else {
        let velocity = simulator.me().velocity();
        if velocity.norm() > 0.0 {
            velocity.normalized() * simulator.rules().ROBOT_MAX_GROUND_SPEED
        } else {
            velocity
        }
    }
}

pub fn does_jump_hit_ball<'r, 'a, G>(allow_nitro: bool, ctx: &mut Context<'r, 'a, G>) -> bool
    where G: Fn(i32, i32) -> Option<&'a Action> {

    use crate::my_strategy::physics::MoveEquation;
    use crate::my_strategy::optimization::minimize1d;

    let mut simulator = ctx.simulator.clone();
    let mut rng = ctx.rng.clone();

    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    let use_nitro = allow_nitro && ctx.simulator.me().nitro_amount() > 0.0;
    simulator.me_mut().action_mut().use_nitro = use_nitro;
    let target_velocity = get_target_velocity_for_jump(use_nitro, &simulator);
    simulator.me_mut().action_mut().set_target_velocity(target_velocity);

    simulator.tick(ctx.simulator.rules().tick_time_interval(), ctx.near_micro_ticks_per_tick, &mut rng);

    *ctx.used_path_micro_ticks += ctx.near_micro_ticks_per_tick;

    let my_move_equation = if allow_nitro {
        MoveEquation::from_robot_with_nitro(simulator.me().base(), simulator.rules())
    } else {
        MoveEquation::from_robot(simulator.me().base(), simulator.rules())
    };
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
        simulator.rules().jump_to_max_height_time(),
        10,
        get_distance
    );

    get_distance(time) < simulator.rules().ROBOT_MAX_RADIUS + simulator.rules().BALL_RADIUS
        && my_move_equation.get_velocity(time).y() > -simulator.rules().tick_time_interval() * simulator.rules().GRAVITY
        && my_move_equation.get_position(time).y() < ball_move_equation.get_position(time).y()
        && ball_move_equation.get_position(time).y() > ball_min_y - simulator.rules().tick_time_interval() * simulator.rules().GRAVITY
}
