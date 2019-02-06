use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::search::{Search, Visitor, Identifiable};
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::common::IdGenerator;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::scenarios::{Jump, FarJump, WatchMeJump, WalkToPosition, Observe, PushRobot,
                                    WatchBallMove, Context as ScenarioContext, Result as ScenarioResult};

const MAX_PATH_MICRO_TICKS: usize = 1100;
const MAX_ITERATIONS: usize = 100;

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

#[derive(Clone)]
pub struct Plan<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub current_tick: i32,
    pub order_id: i32,
    pub simulator: Simulator,
    pub time_to_play: f64,
    pub max_z: f64,
    pub get_robot_action_at: G,
    pub time_to_ball: Option<f64>,
    pub time_to_goal: Option<f64>,
    pub position_to_jump: Option<Vec3>,
    pub actions: Vec<Action>,
    pub path_micro_ticks: usize,
    pub max_plan_micro_ticks: usize,
    pub adaptive_near_micro_ticks_per_tick: usize,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

pub struct Result {
    pub transitions: Vec<Transition>,
    pub order_id: i32,
    pub score: i32,
    pub simulator: Simulator,
    pub time_to_ball: Option<f64>,
    pub time_to_goal: Option<f64>,
    pub actions: Vec<Action>,
    pub used_micro_ticks: usize,
    #[cfg(feature = "enable_render")]
    pub history: Vec<Simulator>,
    #[cfg(feature = "enable_stats")]
    pub stats: Stats,
}

impl<'a, G> Plan<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub fn new(current_tick: i32, order_id: i32, simulator: Simulator, time_to_play: f64,
               max_z: f64, get_robot_action_at: G, max_plan_micro_ticks: usize) -> Self {
        use crate::my_strategy::scenarios::NEAR_MICRO_TICKS_PER_TICK;

        Plan {
            #[cfg(feature = "enable_stats")]
            stats: {
                let player_id = simulator.me().player_id();
                let robot_id = simulator.me().id();
                Stats::new(player_id, robot_id, current_tick, "play")
            },
            adaptive_near_micro_ticks_per_tick: NEAR_MICRO_TICKS_PER_TICK,
            max_plan_micro_ticks,
            current_tick,
            order_id,
            simulator,
            time_to_play,
            max_z,
            get_robot_action_at,
            time_to_ball: None,
            time_to_goal: None,
            position_to_jump: None,
            actions: Vec::new(),
            path_micro_ticks: 0,
            #[cfg(feature = "enable_render")]
            history: Vec::new(),
        }
    }

    pub fn search(&self, rng: &mut XorShiftRng) -> Result
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        let mut visitor = VisitorImpl::new(rng);

        let initial_state = visitor.make_initial_state(self.clone());

        let (transitions, final_state, iterations) = Search {
            max_iterations: MAX_ITERATIONS,
        }.perform(initial_state, &mut visitor);

        let plan = final_state.map(|v| v.take_plan())
            .unwrap_or(self.clone());
        let score = plan.get_score();

        Result {
            transitions,
            score,
            order_id: plan.order_id,
            simulator: plan.simulator,
            time_to_ball: plan.time_to_ball,
            time_to_goal: plan.time_to_goal,
            actions: plan.actions,
            used_micro_ticks: visitor.used_micro_ticks,
            #[cfg(feature = "enable_render")]
            history: plan.history,
            #[cfg(feature = "enable_stats")]
            stats: {
                let mut stats = plan.stats;
                stats.order_score = score;
                stats.total_iterations = iterations;
                stats
            },
        }
    }

    pub fn get_score(&self) -> i32 {
        get_score(&self.simulator, self.time_to_ball, self.time_to_goal)
    }
}

pub struct VisitorImpl<'r> {
    rng: &'r mut XorShiftRng,
    state_id_generator: IdGenerator,
    used_micro_ticks: usize,
}

impl<'r> VisitorImpl<'r> {
    pub fn new(rng: &'r mut XorShiftRng) -> Self {
        VisitorImpl {
            rng,
            state_id_generator: IdGenerator::new(),
            used_micro_ticks: 0,
        }
    }

    pub fn make_initial_state<'a, G>(&mut self, plan: Plan<'a, G>) -> State<'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        State::initial(self.state_id_generator.next(), plan)
    }

    pub fn get_transitions_for_initial_state<'a, G>(state: &Final<'a, G>) -> Vec<Transition>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action>  {

        use crate::my_strategy::entity::Entity;

        if state.plan.simulator.rules().is_flying(state.plan.simulator.me().base()) {
            let mut result = vec![
                Transition::watch_me_jump(0.0, false),
                Transition::watch_me_jump(state.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, false),
            ];

            if state.plan.simulator.me().nitro_amount() > 0.0 {
                result.push(Transition::watch_me_jump(0.0, true));
                result.push(Transition::watch_me_jump(state.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, true));
            }

            result
        } else {
            let mut result = Vec::new();

            if state.plan.time_to_play == 0.0 {
                if state.plan.simulator.me().nitro_amount() > 0.0
                    && state.plan.simulator.ball().position().y() < state.plan.simulator.rules().arena.goal_height + 5.0
                    && state.plan.simulator.me().position().distance(state.plan.simulator.ball().position()) < 15.0 {
                    result.push(Transition::far_jump(true));
                }

                result.push(Transition::far_jump(false));
            }

            result.push(Transition::observe(0, state.plan.time_to_play, state.plan.max_z));

            Self::try_add_push_robot(&state.plan, &mut result);

            result
        }
    }

    pub fn try_add_push_robot<'a, G>(plan: &Plan<'a, G>, transitions: &mut Vec<Transition>)
        where G: Clone + Fn(i32, i32) -> Option<&'a Action>  {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::common::as_score;

        let ball = plan.simulator.ball();
        let me = plan.simulator.me();
        let rules = plan.simulator.rules();

        if me.nitro_amount() <= rules.START_NITRO_AMOUNT
            || plan.time_to_play == 0.0
            || plan.simulator.current_tick() > 0 {
            return;
        }

        plan.simulator.robots().iter()
            .filter(|v| {
                !v.is_teammate()
                && v.position().z() < plan.max_z
                && v.position().distance(ball.position()) < rules.arena.depth / 8.0
                && v.position().distance(me.position()) < rules.arena.depth / 8.0
            })
            .min_by_key(|v| {
                as_score(v.position().distance(ball.position()))
            })
            .map(|v| {
                transitions.push(Transition::push_robot(
                    v.id(),
                    true,
                    plan.time_to_play.max(20.0 * rules.tick_time_interval())
                ));
            });
    }

    pub fn get_transitions_for_forked_state<'a, G>(&mut self, state: &Forked<'a, G>) -> Vec<Transition>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        let observe_simulator = &state.observe_simulator;
        let rules = observe_simulator.rules();
        let time_interval = rules.tick_time_interval();

        log!(
            state.plan.current_tick, "[{}] <{}> <{}> use time point {}:{}:{}",
            observe_simulator.me().id(), state.plan.order_id, state.id,
            observe_simulator.current_time(), observe_simulator.current_tick(),
            observe_simulator.current_micro_tick()
        );

        let mut result = get_points(observe_simulator, state.plan.current_tick, self.rng).into_iter()
            .map(|point| {
                let position_to_jump = {
                    let mut robot = observe_simulator.me().clone();
                    robot.set_position(point);
                    rules.arena.collide(&mut robot);
                    robot.position()
                };
                let to_target = position_to_jump - observe_simulator.me().position();
                let distance_to_target = to_target.norm();
                let max_speed = if observe_simulator.current_time() > 0.0 {
                    if distance_to_target > rules.ROBOT_MAX_GROUND_SPEED * 20.0 * time_interval {
                        rules.ROBOT_MAX_GROUND_SPEED
                    } else {
                        distance_to_target / observe_simulator.current_time()
                    }
                } else {
                    rules.ROBOT_MAX_GROUND_SPEED
                };

                Transition::walk_to_position(position_to_jump, max_speed)
            })
            .collect();

        Self::try_add_push_robot(&state.plan, &mut result);

        result
    }

    pub fn fork<'a, G>(&mut self, state: &State<'a, G>) -> State<'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        if let State::Observed(state) = state {
            let plan = &state.plan;

            log!(
                plan.current_tick, "[{}] <{}> <{}> fork {}:{}:{}",
                plan.simulator.me().id(), plan.order_id, state.id,
                plan.simulator.current_time(), plan.simulator.current_tick(),
                plan.simulator.current_micro_tick()
            );

            let mut initial_plan = state.initial_plan.clone();

            #[cfg(feature = "enable_stats")]
            initial_plan.stats.update(&plan.stats);

            State::forked(self.state_id_generator.next(), initial_plan, state.plan.simulator.clone())
        } else {
            unimplemented!()
        }
    }

    pub fn use_scenario<'a, G>(&mut self, state: &State<'a, G>, transition: &Transition) -> State<'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::scenarios::{NEAR_MICRO_TICKS_PER_TICK, FAR_MICRO_TICKS_PER_TICK};

        let mut plan = state.plan().clone();

        log!(
            plan.current_tick, "[{}] <{}> <{}> use_scenario {}:{}:{} {:?}",
            plan.simulator.me().id(), plan.order_id, state.id(),
            plan.simulator.current_time(), plan.simulator.current_tick(),
            plan.simulator.current_micro_tick(), transition
        );

        let path_micro_ticks_before = plan.path_micro_ticks;

        plan.adaptive_near_micro_ticks_per_tick = if let Transition::WalkToPosition(v) = transition {
            let time_interval = plan.simulator.rules().tick_time_interval();
            let distance_to_ball = plan.simulator.me().position()
                .distance(plan.simulator.ball().position());
            let ball_distance_limit = plan.simulator.rules().ball_distance_limit()
                + v.max_speed * time_interval;
            if distance_to_ball > ball_distance_limit {
                FAR_MICRO_TICKS_PER_TICK
            } else {
                NEAR_MICRO_TICKS_PER_TICK
            }
        } else {
            plan.adaptive_near_micro_ticks_per_tick
        };

        if let Transition::WalkToPosition(v) = transition {
            plan.position_to_jump = Some(v.target);
            log!(
                plan.current_tick, "[{}] <{}> <{}> use position to jump {}:{}:{} {:?}",
                plan.simulator.me().id(), plan.order_id, state.id(),
                plan.simulator.current_time(), plan.simulator.current_tick(),
                plan.simulator.current_micro_tick(), plan.position_to_jump
            );
        };

        let mut ctx = ScenarioContext {
            first: true,
            current_tick: plan.current_tick,
            robot_id: plan.simulator.me().id(),
            order_id: plan.order_id,
            state_id: state.id(),
            simulator: &mut plan.simulator,
            rng: self.rng,
            time_to_ball: &mut plan.time_to_ball,
            time_to_goal: &mut plan.time_to_goal,
            get_robot_action_at: plan.get_robot_action_at.clone(),
            actions: &mut plan.actions,
            near_micro_ticks_per_tick: plan.adaptive_near_micro_ticks_per_tick,
            far_micro_ticks_per_tick: FAR_MICRO_TICKS_PER_TICK,
            used_path_micro_ticks: &mut plan.path_micro_ticks,
            max_path_micro_ticks: match transition {
                Transition::Observe(_) => plan.max_plan_micro_ticks - self.used_micro_ticks,
                _ => MAX_PATH_MICRO_TICKS,
            },
            #[cfg(feature = "enable_render")]
            history: &mut plan.history,
            #[cfg(feature = "enable_stats")]
            stats: &mut plan.stats,
        };

        let result = transition.perform(&mut ctx);

        self.used_micro_ticks += *ctx.used_path_micro_ticks - path_micro_ticks_before;

        if self.used_micro_ticks >= plan.max_plan_micro_ticks {
            State::end(self.state_id_generator.next(), plan)
        } else {
            match result {
                Ok(_) => match transition {
                    Transition::Observe(v) => {
                        let initial_plan = match state {
                            State::Observed(v) => v.initial_plan.clone(),
                            _ => state.plan().clone(),
                        };
                        State::observed(self.state_id_generator.next(), v.number, plan, initial_plan)
                    },
                    Transition::WalkToPosition(_) => State::walked(self.state_id_generator.next(), plan),
                    Transition::Jump(_) => State::jumped(self.state_id_generator.next(), plan),
                    Transition::FarJump(_) => State::far_jumped(self.state_id_generator.next(), plan),
                    Transition::WatchMeJump(_) => State::hit(self.state_id_generator.next(), plan),
                    Transition::WatchBallMove(_) => State::end(self.state_id_generator.next(), plan),
                    Transition::PushRobot(_) => State::initial(self.state_id_generator.next(), plan),
                    Transition::Fork(_) => unimplemented!(),
                },
                Err(_) => State::end(self.state_id_generator.next(), plan),
            }
        }
    }
}

impl<'r, 'a, G> Visitor<State<'a, G>, Transition> for VisitorImpl<'r>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn is_final(&self, state: &State<'a, G>) -> bool {
        state.is_final()
    }

    fn get_transitions(&mut self, state: &State<'a, G>) -> Vec<Transition> {
        match state {
            State::Initial(v) => Self::get_transitions_for_initial_state(v),
            State::Observed(v) => vec![
                Transition::fork(),
                Transition::observe(v.number + 1, v.plan.time_to_play, v.plan.max_z),
            ],
            State::Forked(v) => self.get_transitions_for_forked_state(v),
            State::Walked(_) => vec![Transition::jump(false)],
            State::Jumped(v) => vec![
                Transition::watch_me_jump(v.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, false)
            ],
            State::FarJumped(v) => vec![
                Transition::watch_me_jump(v.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, false)
            ],
            State::Hit(_) => vec![Transition::watch_ball_move()],
            State::End(_) => Vec::new(),
        }
    }

    fn apply(&mut self, iteration: usize, state: &State<'a, G>, transition: &Transition) -> State<'a, G> {
        let mut result = match transition {
            Transition::Fork(_) => self.fork(state),
            _ => self.use_scenario(state, transition),
        };

        log!(
            state.plan().current_tick, "[{}] <{}> <{}> transition {}:{}:{} {:?} -> <{}>",
            state.plan().simulator.me().id(), state.plan().order_id, state.id(),
            state.plan().simulator.current_time(), state.plan().simulator.current_tick(),
            state.plan().simulator.current_micro_tick(), transition, result.id()
        );

        #[cfg(feature = "enable_stats")]
        {
            result.plan_mut().stats.iteration = iteration;

            if result.plan_mut().stats.path_type.is_none() {
                result.plan_mut().stats.path_type = match transition {
                    Transition::WalkToPosition(_) => Some("walk_to_position"),
                    Transition::Jump(_) => Some("jump"),
                    Transition::FarJump(_) => Some("far_jump"),
                    Transition::WatchMeJump(_) => Some("watch_me_jump"),
                    Transition::WatchBallMove(_) => Some("watch_ball_move"),
                    _ => result.plan_mut().stats.path_type,
                };
            }
        }

        result
    }

    fn get_transition_cost(&mut self, source_state: &State<'a, G>, destination_state: &State<'a, G>, transition: &Transition) -> i32 {
        match transition {
            Transition::Fork(_) => 0,
            Transition::Observe(_) => 1,
            _ => source_state.score() - destination_state.score(),
        }
    }

    fn get_score(&self, state: &State<'a, G>) -> i32 {
        state.plan().get_score()
    }
}

#[derive(Clone)]
pub enum State<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    Initial(Final<'a, G>),
    Observed(Observed<'a, G>),
    Forked(Forked<'a, G>),
    Walked(Final<'a, G>),
    Jumped(Final<'a, G>),
    FarJumped(Final<'a, G>),
    Hit(Final<'a, G>),
    End(Final<'a, G>),
}

impl<'a, G> State<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub fn initial(id: i32, plan: Plan<'a, G>) -> Self {
        State::Initial(Final { id, score: 0, plan })
    }

    pub fn observed(id: i32, number: usize, plan: Plan<'a, G>, initial_plan: Plan<'a, G>) -> Self {
        State::Observed(Observed { id, number, score: 0, plan, initial_plan })
    }

    pub fn forked(id: i32, plan: Plan<'a, G>, observe_simulator: Simulator) -> Self {
        State::Forked(Forked { id, score: plan.get_score(), plan, observe_simulator })
    }

    pub fn walked(id: i32, plan: Plan<'a, G>) -> Self {
        State::Walked(Final { id, score: plan.get_score(), plan })
    }

    pub fn jumped(id: i32, plan: Plan<'a, G>) -> Self {
        State::Jumped(Final { id, score: plan.get_score(), plan })
    }

    pub fn far_jumped(id: i32, plan: Plan<'a, G>) -> Self {
        State::FarJumped(Final { id, score: plan.get_score(), plan })
    }

    pub fn hit(id: i32, plan: Plan<'a, G>) -> Self {
        State::Hit(Final { id, score: plan.get_score(), plan })
    }

    pub fn end(id: i32, plan: Plan<'a, G>) -> Self {
        State::End(Final { id, score: plan.get_score(), plan })
    }

    pub fn id(&self) -> i32 {
        match self {
            State::Initial(v) => v.id,
            State::Observed(v) => v.id,
            State::Forked(v) => v.id,
            State::Walked(v) => v.id,
            State::Jumped(v) => v.id,
            State::FarJumped(v) => v.id,
            State::Hit(v) => v.id,
            State::End(v) => v.id,
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            State::Initial(v) => v.score,
            State::Observed(v) => v.score,
            State::Forked(v) => v.score,
            State::Walked(v) => v.score,
            State::Jumped(v) => v.score,
            State::FarJumped(v) => v.score,
            State::Hit(v) => v.score,
            State::End(v) => v.score,
        }
    }

    pub fn plan(&self) -> &Plan<'a, G> {
        match self {
            State::Initial(v) => &v.plan,
            State::Observed(v) => &v.plan,
            State::Forked(v) => &v.plan,
            State::Walked(v) => &v.plan,
            State::Jumped(v) => &v.plan,
            State::FarJumped(v) => &v.plan,
            State::Hit(v) => &v.plan,
            State::End(v) => &v.plan,
        }
    }

    pub fn plan_mut(&mut self) -> &mut Plan<'a, G> {
        match self {
            State::Initial(v) => &mut v.plan,
            State::Observed(v) => &mut v.plan,
            State::Forked(v) => &mut v.plan,
            State::Walked(v) => &mut v.plan,
            State::Jumped(v) => &mut v.plan,
            State::FarJumped(v) => &mut v.plan,
            State::Hit(v) => &mut v.plan,
            State::End(v) => &mut v.plan,
        }
    }

    pub fn take_plan(self) -> Plan<'a, G> {
        match self {
            State::Initial(v) => v.plan,
            State::Observed(v) => v.plan,
            State::Forked(v) => v.plan,
            State::Walked(v) => v.plan,
            State::Jumped(v) => v.plan,
            State::FarJumped(v) => v.plan,
            State::Hit(v) => v.plan,
            State::End(v) => v.plan,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            State::Initial(_) => "Initial",
            State::Observed(_) => "Observed",
            State::Forked(_) => "Forked",
            State::Walked(_) => "Walked",
            State::Jumped(_) => "Jumped",
            State::FarJumped(_) => "FarJumped",
            State::Hit(_) => "Hit",
            State::End(_) => "End",
        }
    }

    pub fn is_final(&self) -> bool {
        match self {
            State::End(v) => !v.plan.actions.is_empty(),
            _ => false,
        }
    }
}

impl<'a, G> std::fmt::Debug for State<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a, G> Identifiable for State<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn id(&self) -> i32 {
        Self::id(self)
    }
}

#[derive(Clone)]
pub struct Final<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub plan: Plan<'a, G>,
}

#[derive(Clone)]
pub struct Observed<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub number: usize,
    pub plan: Plan<'a, G>,
    pub initial_plan: Plan<'a, G>,
}

#[derive(Clone)]
pub struct Forked<'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub plan: Plan<'a, G>,
    pub observe_simulator: Simulator,
}

#[derive(Clone, Debug)]
pub enum Transition {
    Observe(Observe),
    Fork(Fork),
    WalkToPosition(WalkToPosition),
    Jump(Jump),
    FarJump(FarJump),
    WatchMeJump(WatchMeJump),
    WatchBallMove(WatchBallMove),
    PushRobot(PushRobot),
}

impl Transition {
    pub fn observe(number: usize, wait_until: f64, max_ball_z: f64) -> Self {
        Transition::Observe(Observe { number, wait_until, max_ball_z })
    }

    pub fn fork() -> Self {
        Transition::Fork(Fork {})
    }

    pub fn walk_to_position(target: Vec3, max_speed: f64) -> Self {
        Transition::WalkToPosition(WalkToPosition { target, max_speed })
    }

    pub fn jump(allow_nitro: bool) -> Self {
        Transition::Jump(Jump { allow_nitro })
    }

    pub fn far_jump(allow_nitro: bool) -> Self {
        Transition::FarJump(FarJump { allow_nitro })
    }

    pub fn watch_me_jump(jump_speed: f64, allow_nitro: bool) -> Self {
        Transition::WatchMeJump(WatchMeJump { jump_speed, allow_nitro })
    }

    pub fn watch_ball_move() -> Self {
        Transition::WatchBallMove(WatchBallMove {})
    }

    pub fn push_robot(robot_id: i32, allow_nitro: bool, until_time: f64) -> Self {
        Transition::PushRobot(PushRobot { robot_id, allow_nitro, until_time })
    }

    pub fn perform<'r, 'a, G>(&self, ctx: &mut ScenarioContext<'r, 'a, G>) -> ScenarioResult
        where G: Fn(i32, i32) -> Option<&'a Action> {

        match self {
            Transition::Observe(v) => v.perform(ctx),
            Transition::WalkToPosition(v) => v.perform(ctx),
            Transition::Jump(v) => v.perform(ctx),
            Transition::FarJump(v) => v.perform(ctx),
            Transition::WatchMeJump(v) => v.perform(ctx),
            Transition::WatchBallMove(v) => v.perform(ctx),
            Transition::PushRobot(v) => v.perform(ctx),
            Transition::Fork(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fork;

pub fn get_score(simulator: &Simulator, time_to_ball: Option<f64>, time_to_goal: Option<f64>) -> i32 {
    use crate::my_strategy::common::as_score;
    use crate::my_strategy::scenarios::MAX_TICKS;
    use crate::my_strategy::vec2::Vec2;
    use crate::my_strategy::entity::Entity;

    let rules = simulator.rules();
    let max_time = (MAX_TICKS + 1) as f64 * rules.tick_time_interval();
    let ball = simulator.ball();
    let to_goal = rules.get_goal_target() - ball.position();

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

    let time_to_ball_score = if let Some(v) = time_to_ball {
        1.0 - v / max_time
    } else {
        0.0
    };

    let time_to_goal_score = if let Some(v) = time_to_goal {
        if simulator.score() > 0 {
            1.0 - v / max_time
        } else {
            v / max_time
        }
    } else {
        0.0
    };

    let total = 0.0
        + ball_goal_distance_score
        + 0.1 * ball_goal_direction_score
        + 0.5 * time_to_ball_score
        + 0.25 * time_to_goal_score;

    as_score(total)
}

pub fn get_points(simulator: &Simulator, current_tick: i32, rng: &mut XorShiftRng) -> Vec<Vec3> {
    use crate::my_strategy::physics::get_min_distance_between_spheres;
    use crate::my_strategy::common::Clamp;
    use crate::my_strategy::plane::Plane;
    use crate::my_strategy::mat3::Mat3;
    use crate::my_strategy::entity::Entity;
    use crate::my_strategy::random::Rng;

    let ball = simulator.ball();
    let robot = simulator.me();
    let rules = simulator.rules();

    let distance_to_ball = ball.position().distance(robot.position());
    let time_to_ball = rules.time_for_distance(rules.ROBOT_MAX_GROUND_SPEED, distance_to_ball);
    let max_time_diff = 2.0 * (rules.ROBOT_RADIUS + rules.BALL_RADIUS) / rules.ROBOT_MAX_GROUND_SPEED;
    let number = if time_to_ball < simulator.current_time() + max_time_diff {
        if time_to_ball < rules.tick_time_interval() * 10.0 {
            if simulator.rules().team_size <= 2 {
                9
            } else {
                7
            }
        } else {
            3
        }
    } else {
        1
    };
    let base_position = ball.projected_to_arena_position_with_shift(rules.ROBOT_MIN_RADIUS);
    let to_robot = (robot.position() - base_position).normalized();
    let min_distance = get_min_distance_between_spheres(
        ball.distance_to_arena(),
        rules.BALL_RADIUS,
        rules.ROBOT_MIN_RADIUS,
    ).unwrap_or(0.0);
    let max_distance = base_position.distance(robot.position())
        .clamp(min_distance + 1e-3, rules.BALL_RADIUS + rules.ROBOT_MAX_RADIUS);
    let base_direction = Plane::projected(to_robot, ball.normal_to_arena()).normalized();
    log!(
        current_tick,
        "[{}] get_points base_position={:?} base_direction={:?} min_distance={} max_distance={}",
        robot.id(), base_position, base_direction, min_distance, max_distance
    );
    let mut result = Vec::new();

    if rules.is_near_my_goal(ball.position()) {
        result.push(
            ball.position().with_y(rules.ROBOT_RADIUS)
                .with_max_z(-rules.arena.depth / 2.0 - rules.BALL_RADIUS)
        );
    }

    let mean_distance = (max_distance + min_distance) / 2.0;
    for i in 0..number {
        let (angle, distance) = if i % 2 == 0 {
            if i % 4 == 0 {
                (std::f64::consts::PI * i as f64 / number as f64, mean_distance)
            } else {
                (
                    rng.gen_range(
                        std::f64::consts::PI * (i - 1) as f64 / number as f64,
                        std::f64::consts::PI * (i + 1) as f64 / number as f64
                    ),
                    rng.gen_range(min_distance, max_distance)
                )
            }
        } else {
            if i + 1 % 4 == 0 {
                (-std::f64::consts::PI * i as f64 / number as f64, mean_distance)
            } else {
                (
                    rng.gen_range(
                        -std::f64::consts::PI * (i as f64 + 1.5) / number as f64,
                        -std::f64::consts::PI * (i as f64 - 0.5) / number as f64
                    ),
                    rng.gen_range(min_distance, max_distance)
                )
            }
        };
        let rotation = Mat3::rotation(ball.normal_to_arena(), angle);
        let position = base_position + rotation * base_direction * mean_distance;
        let projected = rules.arena.projected_with_shift(position, rules.ROBOT_MAX_RADIUS);
        log!(
            current_tick,
            "[{}] get_points distance={} angle={} position={:?} projected={:?} distance_to_ball={}",
            robot.id(), distance, angle, position, projected, projected.distance(ball.position())
        );
        result.push(projected);
    }

    result
}
