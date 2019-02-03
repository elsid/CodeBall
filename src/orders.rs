use crate::model::{Robot, Action};
use crate::my_strategy::world::World;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::common::IdGenerator;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

#[cfg(feature = "enable_render")]
use crate::my_strategy::vec3::Vec3;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

const MAX_PLAN_MICRO_TICKS: usize = 40000;

pub enum Order {
    Idle(Idle),
    Play(Play),
    WalkToGoalkeeperPosition(WalkToGoalkeeperPosition),
    TakeNitroPack(TakeNitroPack),
    PushOpponent(PushOpponent),
}

impl Order {
    pub fn try_play(robot: &Robot, world: &World, other: &[Order], max_z: f64, ctx: &mut OrderContext) -> Order {
        if let Some(play) = Play::try_new(robot, world, other, max_z, ctx) {
            Order::Play(play)
        } else {
            Self::idle(robot, world, ctx.order_id_generator)
        }
    }

    pub fn walk_to_goalkeeper_position(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Order {
        Order::WalkToGoalkeeperPosition(
            WalkToGoalkeeperPosition::new(robot, world, order_id_generator)
        )
    }

    pub fn try_take_nitro_pack(robot: &Robot, world: &World, max_z: f64, order_id_generator: &mut IdGenerator) -> Order {
        if let Some(take_nitro_pack) = TakeNitroPack::try_new(robot, world, max_z, order_id_generator) {
            Order::TakeNitroPack(take_nitro_pack)
        } else {
            Self::idle(robot, world, order_id_generator)
        }
    }

    pub fn try_push_opponent(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Order {
        if let Some(push_opponent) = PushOpponent::try_new(robot, world, order_id_generator) {
            Order::PushOpponent(push_opponent)
        } else {
            Self::idle(robot, world, order_id_generator)
        }
    }

    pub fn idle(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Order {
        Order::Idle(Idle::new(robot, world, order_id_generator))
    }

    pub fn id(&self) -> i32 {
        match self {
            Order::Idle(v) => v.id,
            Order::Play(v) => v.id,
            Order::WalkToGoalkeeperPosition(v) => v.id,
            Order::TakeNitroPack(v) => v.id,
            Order::PushOpponent(v) => v.id,
        }
    }

    pub fn robot_id(&self) -> i32 {
        match self {
            Order::Idle(v) => v.robot_id,
            Order::Play(v) => v.robot_id,
            Order::WalkToGoalkeeperPosition(v) => v.robot_id,
            Order::TakeNitroPack(v) => v.robot_id,
            Order::PushOpponent(v) => v.robot_id,
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            Order::Idle(v) => v.score,
            Order::Play(v) => v.score,
            Order::WalkToGoalkeeperPosition(v) => v.score,
            Order::TakeNitroPack(v) => v.score,
            Order::PushOpponent(v) => v.score,
        }
    }

    pub fn action(&self) -> &Action {
        match self {
            Order::Idle(v) => &v.action,
            Order::Play(v) => v.actions.first().unwrap(),
            Order::WalkToGoalkeeperPosition(v) => &v.action,
            Order::TakeNitroPack(v) => &v.action,
            Order::PushOpponent(v) => &v.action,
        }
    }

    pub fn action_at(&self, tick: i32) -> Option<&Action> {
        if tick == 0 {
            Some(self.action())
        } else {
            match self {
                Order::Idle(_) => None,
                Order::Play(v) => v.action_at(tick),
                Order::WalkToGoalkeeperPosition(_) => None,
                Order::TakeNitroPack(_) => None,
                Order::PushOpponent(_) => None,
            }
        }
    }

    pub fn time_to_ball(&self) -> Option<f64> {
        match self {
            Order::Idle(_) => None,
            Order::Play(v) => v.time_to_ball,
            Order::WalkToGoalkeeperPosition(_) => None,
            Order::TakeNitroPack(_) => None,
            Order::PushOpponent(_) => None,
        }
    }

    pub fn is_idle(&self) -> bool {
        match self {
            Order::Idle(_) => true,
            _ => false,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Order::Idle(_) => self,
            Order::Play(v) => Order::Play(v.opposite()),
            Order::WalkToGoalkeeperPosition(v) => Order::WalkToGoalkeeperPosition(v.opposite()),
            Order::TakeNitroPack(v) => Order::TakeNitroPack(v.opposite()),
            Order::PushOpponent(v) => Order::PushOpponent(v.opposite()),
        }
    }

    #[cfg(feature = "enable_stats")]
    pub fn stats(&self) -> &Stats {
        match self {
            Order::Play(v) => &v.stats,
            Order::WalkToGoalkeeperPosition(v) => &v.stats,
            Order::TakeNitroPack(v) => &v.stats,
            Order::Idle(v) => &v.stats,
            Order::PushOpponent(v) => &v.stats,
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn name(&self) -> &'static str {
        match self {
            Order::Play(_) => "play",
            Order::WalkToGoalkeeperPosition(_) => "walk_to_goalkeeper_position",
            Order::TakeNitroPack(_) => "take_nitro_pack",
            Order::Idle(_) => "idle",
            Order::PushOpponent(_) => "push_opponent",
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
            "  order:\n    name: {} id: {} score: {}\n    speed: {} jump: {} nitro: {}\n",
            self.name(), self.id(), self.score(), self.action().target_velocity().norm(),
            self.action().jump_speed, self.action().use_nitro
        )));
    }

    #[cfg(feature = "enable_render")]
    pub fn render_action(&self, robot: &Robot, render: &mut Render) {
        self.action().render(robot, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_sub(&self, render: &mut Render) {
        match self {
            Order::Idle(_) => (),
            Order::Play(v) => v.render(render),
            _ => (),
        }
    }
}

pub struct Idle {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl Idle {
    pub fn new(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Self {
        Idle {
            id: order_id_generator.next(),
            robot_id: robot.id,
            action: Action::default(),
            score: 0,
            #[cfg(feature = "enable_stats")]
            stats: Stats::new(robot.player_id, robot.id, world.game.current_tick, "idle"),
        }
    }
}

pub struct Play {
    pub id: i32,
    pub robot_id: i32,
    pub score: i32,
    pub time_to_ball: Option<f64>,
    pub actions: Vec<Action>,
    #[cfg(feature = "enable_render")]
    pub position_to_jump: Option<Vec3>,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl Play {
    pub fn try_new(robot: &Robot, world: &World, other: &[Order], max_z: f64, ctx: &mut OrderContext) -> Option<Self> {
        use crate::my_strategy::plan::Plan;

        log!(
            world.game.current_tick,
            "[{}] try play robot_position={:?} robot_velocity={:?} ball_position={:?} ball_velocity={:?}",
            robot.id, robot.position(), robot.velocity(), world.game.ball.position(), world.game.ball.velocity()
        );

        let time_to_play = get_min_time_to_play_ball(other, world);
        let max_plan_micro_ticks = MAX_PLAN_MICRO_TICKS / world.rules.team_size as usize;

        let plan = Plan::new(
            world.game.current_tick,
            ctx.order_id_generator.next(),
            make_initial_simulator(robot, world),
            time_to_play,
            max_z,
            make_get_robot_action_at(other),
            max_plan_micro_ticks
                .min(world.get_micro_ticks_limit() - (*ctx.micro_ticks).min(world.get_micro_ticks_limit())),
        ).search(ctx.rng);

        *ctx.micro_ticks += plan.used_micro_ticks;

        if plan.actions.is_empty() {
            return None;
        }

        let mut order = Play {
            id: plan.order_id,
            robot_id: robot.id,
            score: plan.score,
            time_to_ball: plan.time_to_ball,
            actions: plan.actions,
            #[cfg(feature = "enable_render")]
            position_to_jump: None,
            #[cfg(feature = "enable_render")]
            history: plan.history,
            #[cfg(feature = "enable_stats")]
            stats: plan.stats,
        };

        #[cfg(feature = "enable_stats")]
        {
            order.stats.plan_micro_ticks = plan.used_micro_ticks;
            order.stats.game_micro_ticks = *ctx.micro_ticks;
            order.stats.game_micro_ticks_limit = world.get_micro_ticks_limit();
            order.stats.reached_plan_limit = plan.used_micro_ticks >= max_plan_micro_ticks;
            order.stats.reached_game_limit = world.is_micro_ticks_limit_reached(*ctx.micro_ticks);
            order.stats.other_number = other.len();
        }

        Some(order)
    }

    pub fn opposite(self) -> Self {
        Play {
            id: self.id,
            robot_id: self.robot_id,
            score: self.score,
            time_to_ball: self.time_to_ball,
            actions: self.actions.into_iter().map(|v| v.opposite()).collect(),
            #[cfg(feature = "enable_render")]
            position_to_jump: self.position_to_jump.map(|v| v.opposite()),
            #[cfg(feature = "enable_render")]
            history: self.history.into_iter().map(|v| v.opposite()).collect(),
            #[cfg(feature = "enable_stats")]
            stats: self.stats,
        }
    }

    pub fn action_at(&self, tick: i32) -> Option<&Action> {
        if 0 <= tick && (tick as usize) < self.actions.len() {
            Some(&self.actions[tick as usize])
        } else {
            None
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn render(&self, render: &mut Render) {
        self.render_position_to_jump(render);
        render_history(&self.history, render);
    }

    #[cfg(feature = "enable_render")]
    pub fn render_position_to_jump(&self, render: &mut Render) {
        use crate::my_strategy::render::{Object, Color};

        if let Some(position) = self.position_to_jump {
            render.add(Object::sphere(position, 1.0, Color::new(0.5, 0.0, 0.0, 0.8)));
        }
        render.add(Object::text(format!("    position_to_jump: {:?}", self.position_to_jump)));
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

    pub fn opposite(self) -> Self {
        WalkToGoalkeeperPosition {
            id: self.id,
            robot_id: self.robot_id,
            action: self.action.opposite(),
            score: self.score,
            #[cfg(feature = "enable_stats")]
            stats: self.stats,
        }
    }
}

pub struct OrderContext<'r> {
    pub rng: &'r mut XorShiftRng,
    pub order_id_generator: &'r mut IdGenerator,
    pub micro_ticks: &'r mut usize,
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

pub struct TakeNitroPack {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl TakeNitroPack {
    pub fn try_new(robot: &Robot, world: &World, max_z: f64, order_id_generator: &mut IdGenerator) -> Option<Self> {
        use crate::my_strategy::common::as_score;

        world.game.nitro_packs.iter()
            .filter(|v| {
                v.z < max_z && v.respawn_ticks.map(|v| v < 100).unwrap_or(true)
            })
            .map(|v| (v.position().distance(robot.position()), v))
            .filter(|(distance, _)| *distance < world.rules.arena.depth / 2.0)
            .min_by_key(|(distance, _)| as_score(*distance))
            .map(|(_, nitro_pack)| {
                let to_target = nitro_pack.position() - robot.position();
                let velocity = if to_target.norm() > world.rules.min_running_distance() {
                    to_target.normalized() * world.rules.ROBOT_MAX_GROUND_SPEED
                } else {
                    to_target * world.rules.ROBOT_MAX_GROUND_SPEED / world.rules.min_running_distance()
                };
                let mut action = Action::default();
                action.set_target_velocity(velocity);
                TakeNitroPack {
                    id: order_id_generator.next(),
                    robot_id: robot.id,
                    action,
                    score: 0,
                    #[cfg(feature = "enable_stats")]
                    stats: Stats::new(robot.player_id, robot.id, world.game.current_tick, "take_nitro_pack"),
                }
            })
    }

    pub fn opposite(self) -> Self {
        TakeNitroPack {
            id: self.id,
            robot_id: self.robot_id,
            action: self.action.opposite(),
            score: self.score,
            #[cfg(feature = "enable_stats")]
            stats: self.stats,
        }
    }
}

pub struct PushOpponent {
    pub id: i32,
    pub robot_id: i32,
    pub action: Action,
    pub score: i32,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl PushOpponent {
    pub fn try_new(robot: &Robot, world: &World, order_id_generator: &mut IdGenerator) -> Option<Self> {
        use crate::my_strategy::common::as_score;

        world.game.robots.iter()
            .filter(|v| {
                !v.is_teammate && v.position().distance(world.game.ball.position()) < 10.0
            })
            .min_by_key(|v| {
                as_score(v.position().distance(world.game.ball.position()))
            })
            .map(|opponent| {
                let to_target = opponent.position() - robot.position();
                let velocity = to_target.normalized() * world.rules.ROBOT_MAX_GROUND_SPEED;
                let mut action = Action::default();
                action.set_target_velocity(velocity);
                PushOpponent {
                    id: order_id_generator.next(),
                    robot_id: robot.id,
                    action,
                    score: 0,
                    #[cfg(feature = "enable_stats")]
                    stats: Stats::new(robot.player_id, robot.id, world.game.current_tick, "push_opponent"),
                }
            })
    }

    pub fn opposite(self) -> Self {
        PushOpponent {
            id: self.id,
            robot_id: self.robot_id,
            action: self.action.opposite(),
            score: self.score,
            #[cfg(feature = "enable_stats")]
            stats: self.stats,
        }
    }
}

fn make_get_robot_action_at<'r>(other: &'r [Order]) -> impl Clone + Fn(i32, i32) -> Option<&'r Action> {
    move |robot_id: i32, tick: i32| -> Option<&'r Action> {
        other.iter()
            .find(|v| v.robot_id() == robot_id)
            .map(|v| v.action_at(tick))
            .unwrap_or_default()
    }
}

fn get_min_time_to_play_ball(other: &[Order], world: &World) -> f64 {
    use crate::my_strategy::common::as_score;

    other.iter()
        .filter(|v| world.is_teammate(v.robot_id()))
        .map(|v| {
            v.time_to_ball().map(|v| {
                v + world.rules.tick_time_interval()
            }).unwrap_or_default()
        })
        .max_by_key(|v| as_score(*v))
        .unwrap_or_default()
}
