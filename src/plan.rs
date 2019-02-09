use crate::model::Action;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::search::{Search, Visitor, Identifiable};
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::common::IdGenerator;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::config::Config;
use crate::my_strategy::scenarios::{
    Jump,
    FarJump,
    WatchMeJump,
    WalkToPosition,
    Observe,
    PushRobot,
    WatchBallMove,
    WalkToBall,
    WalkToRobot,
    Context as ScenarioContext,
    Result as ScenarioResult,
    Error as ScenarioError
};

#[cfg(feature = "enable_stats")]
use crate::my_strategy::stats::Stats;

#[derive(Clone)]
pub struct Plan<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub config: &'c Config,
    pub current_tick: i32,
    pub order_id: i32,
    pub simulator: Simulator,
    pub time_to_play: f64,
    pub max_z: f64,
    pub get_robot_action_at: G,
    pub my_time_to_ball: Option<f64>,
    pub opponent_time_to_ball: Option<f64>,
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

impl<'c, 'a, G> Plan<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub fn new(config: &'c Config, current_tick: i32, order_id: i32, simulator: Simulator,
               time_to_play: f64, max_z: f64, get_robot_action_at: G,
               max_plan_micro_ticks: usize) -> Self {
        Plan {
            #[cfg(feature = "enable_stats")]
            stats: {
                let player_id = simulator.me().player_id();
                let robot_id = simulator.me().id();
                Stats::new(player_id, robot_id, current_tick, "play")
            },
            adaptive_near_micro_ticks_per_tick: config.near_micro_ticks_per_tick,
            config,
            max_plan_micro_ticks,
            current_tick,
            order_id,
            simulator,
            time_to_play,
            max_z,
            get_robot_action_at,
            my_time_to_ball: None,
            opponent_time_to_ball: None,
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
            max_iterations: self.config.max_iterations,
        }.perform(initial_state, &mut visitor);

        let plan = final_state.map(|v| v.take_plan())
            .unwrap_or(self.clone());
        let score = plan.get_score();

        Result {
            transitions,
            score,
            order_id: plan.order_id,
            simulator: plan.simulator,
            time_to_ball: plan.my_time_to_ball,
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
        use crate::my_strategy::common::as_score;
        use crate::my_strategy::entity::Entity;

        let rules = self.simulator.rules();
        let max_time = (self.config.max_ticks + 1) as f64 * rules.tick_time_interval();
        let ball = self.simulator.ball();
        let me = self.simulator.me();
        let to_goal = rules.get_goal_target() - ball.position();

        let ball_goal_distance_score = if self.simulator.score() == 0 {
            1.0 - to_goal.norm() / rules.arena.max_distance()
        } else if self.simulator.score() > 0 {
            2.0
        } else {
            -1.0
        };

        let ball_goal_direction_score = if ball.velocity().norm() > 0.0 {
            (to_goal.cos(ball.velocity()) + 1.0) / 2.0
        } else {
            0.0
        };

        let my_time_to_ball_score = if let Some(v) = self.my_time_to_ball {
            1.0 - v / max_time
        } else {
            0.0
        };

        let time_to_goal_score = if let Some(v) = self.time_to_goal {
            if self.simulator.score() > 0 {
                1.0 - v / max_time
            } else {
                v / max_time
            }
        } else {
            0.0
        };

        let opponent_time_to_ball_penalty = if let Some(v) = self.opponent_time_to_ball {
            1.0 - v / max_time
        } else {
            0.0
        };

        let nitro_amount_score = me.nitro_amount() / rules.MAX_NITRO_AMOUNT;

        let total = 0.0
            + ball_goal_distance_score * self.config.ball_goal_distance_score_weight
            + ball_goal_direction_score * self.config.ball_goal_direction_score_weight
            + my_time_to_ball_score * self.config.my_time_to_ball_score_weight
            + time_to_goal_score * self.config.time_to_goal_score_weight
            - opponent_time_to_ball_penalty * self.config.opponent_time_to_ball_penalty_weight
            + nitro_amount_score * self.config.nitro_amount_score_weight;

        as_score(total)
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

    pub fn make_initial_state<'c, 'a, G>(&mut self, plan: Plan<'c, 'a, G>) -> State<'c, 'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        State::initial(self.state_id_generator.next(), plan)
    }

    pub fn get_transitions_for_initial_state<'c, 'a, G>(state: &Final<'c, 'a, G>) -> Vec<Transition>
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

            Self::try_add_push_robot(&state.plan.simulator, &state.plan, &mut result);
            Self::try_add_take_nitro_pack(&state.plan, &mut result);

            result
        }
    }

    pub fn try_add_push_robot<'c, 'a, G>(simulator: &Simulator, plan: &Plan<'c, 'a, G>, transitions: &mut Vec<Transition>)
        where G: Clone + Fn(i32, i32) -> Option<&'a Action>  {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::common::as_score;

        let ball = simulator.ball();
        let me = plan.simulator.me();
        let rules = simulator.rules();

        if me.nitro_amount() <= rules.START_NITRO_AMOUNT
            || plan.simulator.current_tick() > 10 {
            return;
        }

        simulator.robots().iter()
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

    pub fn try_add_take_nitro_pack<'c, 'a, G>(plan: &Plan<'c, 'a, G>, transitions: &mut Vec<Transition>)
        where G: Clone + Fn(i32, i32) -> Option<&'a Action>  {

        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::common::as_score;

        let rules = plan.simulator.rules();
        let me = plan.simulator.me();

        if me.nitro_amount() == rules.MAX_NITRO_AMOUNT {
            return;
        }

        plan.simulator.nitro_packs().iter()
            .filter(|v| {
                v.z < plan.max_z && v.respawn_ticks.is_none()
            })
            .map(|v| (v.position().distance(me.position()), v))
            .filter(|(distance, _)| *distance < rules.arena.depth / 6.0)
            .min_by_key(|(distance, _)| as_score(*distance))
            .map(|(_, nitro_pack)| {
                let distance = nitro_pack.position().distance(me.position());
                let max_speed = if distance > rules.min_running_distance() {
                    rules.ROBOT_MAX_GROUND_SPEED
                } else {
                    rules.ROBOT_MAX_GROUND_SPEED / rules.min_running_distance()
                };

                transitions.push(Transition::take_nitro_pack(nitro_pack.position(), max_speed));
            });
    }

    pub fn get_transitions_for_forked_ball_state<'c, 'a, G>(&mut self, state: &ForkedBall<'c, 'a, G>) -> Vec<Transition>
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
            .collect::<Vec<_>>();

        if result.len() < 7 && observe_simulator.rules().team_size <= 2 {
            let to_ball = observe_simulator.ball().projected_to_arena_position_with_shift(rules.ROBOT_RADIUS)
                - observe_simulator.me().position();
            result.push(Transition::walk_to_ball(to_ball, true));
        }

        result
    }

    pub fn get_transitions_for_forked_robot_state<'c, 'a, G>(&mut self, state: &ForkedRobot<'c, 'a, G>) -> Vec<Transition>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        let observe_simulator = &state.observe_simulator;
        let robot = observe_simulator.get_robot(state.robot_id);

        if observe_simulator.rules().is_flying(robot.base()) {
            let to_robot = robot.projected_to_arena_position_with_shift(observe_simulator.rules().ROBOT_RADIUS)
                - state.plan.simulator.me().position();
            vec![Transition::walk_to_robot(state.robot_id, to_robot, true)]
        } else if observe_simulator.current_tick() <= 10 {
            let tick_interval = observe_simulator.rules().tick_time_interval();
            let until_time = state.plan.time_to_play.max(20.0 * tick_interval);
            vec![Transition::push_robot(state.robot_id, true, until_time)]
        } else {
            Vec::new()
        }
    }

    pub fn fork<'c, 'a, G>(&mut self, state: &State<'c, 'a, G>) -> State<'c, 'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        match state {
            State::ObservedBall(state) => {
                let plan = &state.plan;

                log!(
                    plan.current_tick, "[{}] <{}> <{}> fork ball {}:{}:{}",
                    plan.simulator.me().id(), plan.order_id, state.id,
                    plan.simulator.current_time(), plan.simulator.current_tick(),
                    plan.simulator.current_micro_tick()
                );

                let mut initial_plan = state.initial_plan.clone();

                #[cfg(feature = "enable_stats")]
                initial_plan.stats.update(&plan.stats);

                State::forked_ball(self.state_id_generator.next(), initial_plan, state.plan.simulator.clone())
            },
            State::ObservedRobot(state) => {
                let plan = &state.plan;

                log!(
                    plan.current_tick, "[{}] <{}> <{}> fork robot {}:{}:{} robot_id={}",
                    plan.simulator.me().id(), plan.order_id, state.id,
                    plan.simulator.current_time(), plan.simulator.current_tick(),
                    plan.simulator.current_micro_tick(), state.robot_id
                );

                let mut initial_plan = state.initial_plan.clone();

                #[cfg(feature = "enable_stats")]
                initial_plan.stats.update(&plan.stats);

                State::forked_robot(self.state_id_generator.next(), state.robot_id, initial_plan, state.plan.simulator.clone())
            },
            _ => unimplemented!(),
        }
    }

    pub fn use_scenario<'c, 'a, G>(&mut self, state: &State<'c, 'a, G>, transition: &Transition) -> State<'c, 'a, G>
        where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

        use crate::my_strategy::entity::Entity;

        let mut plan = state.plan().clone();

        log!(
            plan.current_tick, "[{}] <{}> <{}> use_scenario {}:{}:{} {:?}",
            plan.simulator.me().id(), plan.order_id, state.id(),
            plan.simulator.current_time(), plan.simulator.current_tick(),
            plan.simulator.current_micro_tick(), transition
        );

        let path_micro_ticks_before = plan.path_micro_ticks;

        plan.adaptive_near_micro_ticks_per_tick = match transition {
            Transition::WalkToPosition(v) => {
                let time_interval = plan.simulator.rules().tick_time_interval();
                let distance_to_ball = plan.simulator.me().position()
                    .distance(plan.simulator.ball().position());
                let ball_distance_limit = plan.simulator.rules().ball_distance_limit()
                    + v.max_speed * time_interval;
                if distance_to_ball > ball_distance_limit {
                    plan.config.far_micro_ticks_per_tick
                } else {
                    plan.config.near_micro_ticks_per_tick
                }
            },
            Transition::WalkToBall(_) => plan.config.far_micro_ticks_per_tick,
            _ => plan.adaptive_near_micro_ticks_per_tick,
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
            my_time_to_ball: &mut plan.my_time_to_ball,
            opponent_time_to_ball: &mut plan.opponent_time_to_ball,
            time_to_goal: &mut plan.time_to_goal,
            get_robot_action_at: plan.get_robot_action_at.clone(),
            actions: &mut plan.actions,
            near_micro_ticks_per_tick: plan.adaptive_near_micro_ticks_per_tick,
            far_micro_ticks_per_tick: plan.config.far_micro_ticks_per_tick,
            used_path_micro_ticks: &mut plan.path_micro_ticks,
            max_path_micro_ticks: match transition {
                Transition::Observe(_) => plan.max_plan_micro_ticks.max(self.used_micro_ticks) - self.used_micro_ticks,
                _ => plan.config.max_path_micro_ticks,
            },
            config: plan.config,
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
                    Transition::WalkToPosition(_) => State::walked(self.state_id_generator.next(), plan),
                    Transition::Jump(_) => State::jumped(self.state_id_generator.next(), plan),
                    Transition::FarJump(_) => State::far_jumped(self.state_id_generator.next(), plan),
                    Transition::WatchMeJump(_) => State::hit(self.state_id_generator.next(), plan),
                    Transition::WatchBallMove(_) => State::end(self.state_id_generator.next(), plan),
                    Transition::PushRobot(_) => State::initial(self.state_id_generator.next(), plan),
                    Transition::TakeNitroPack(_) => State::initial(self.state_id_generator.next(), plan),
                    Transition::WalkToBall(_) => State::walked(self.state_id_generator.next(), plan),
                    Transition::WalkToRobot(_) => State::walked(self.state_id_generator.next(), plan),
                    Transition::Observe(_) => unimplemented!(),
                    Transition::ForkBall(_) => unimplemented!(),
                    Transition::ForkRobot(_) => unimplemented!(),
                },
                Err(error) => {
                    match (transition, error) {
                        (Transition::Observe(t), ScenarioError::UseBall) => {
                            State::observed_ball(self.state_id_generator.next(), t.number,
                                                 plan, state.get_initial_plan().clone())
                        },
                        (Transition::Observe(t), ScenarioError::PushRobot(robot_id)) => {
                            State::observed_robot(self.state_id_generator.next(), t.number,
                                                  robot_id, plan, state.get_initial_plan().clone())
                        },
                        _ => {
                            log!(
                                state.plan().current_tick, "[{}] <{}> <{}> error {}:{}:{} {:?}",
                                state.plan().simulator.me().id(), state.plan().order_id, state.id(),
                                state.plan().simulator.current_time(), state.plan().simulator.current_tick(),
                                state.plan().simulator.current_micro_tick(), error
                            );
                            State::end(self.state_id_generator.next(), plan)
                        }
                    }
                },
            }
        }
    }
}

impl<'r, 'c, 'a, G> Visitor<State<'c, 'a, G>, Transition> for VisitorImpl<'r>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn is_final(&self, state: &State<'c, 'a, G>) -> bool {
        state.is_final()
    }

    fn get_transitions(&mut self, state: &State<'c, 'a, G>) -> Vec<Transition> {
        let result = match state {
            State::Initial(v) => Self::get_transitions_for_initial_state(v),
            State::ObservedBall(v) => vec![
                Transition::fork_ball(),
                Transition::observe(v.number + 1, v.plan.time_to_play, v.plan.max_z),
            ],
            State::ObservedRobot(v) => vec![
                Transition::fork_robot(v.robot_id),
                Transition::observe(v.number, v.plan.time_to_play, v.plan.max_z),
            ],
            State::ForkedBall(v) => self.get_transitions_for_forked_ball_state(v),
            State::ForkedRobot(v) => self.get_transitions_for_forked_robot_state(v),
            State::Walked(_) => vec![Transition::jump(false)],
            State::Jumped(v) => vec![
                Transition::watch_me_jump(v.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, false)
            ],
            State::FarJumped(v) => vec![
                Transition::watch_me_jump(v.plan.simulator.rules().ROBOT_MAX_JUMP_SPEED, false)
            ],
            State::Hit(_) => vec![Transition::watch_ball_move()],
            State::End(_) => Vec::new(),
        };

        for transition in result.iter() {
            log!(
                state.plan().current_tick, "[{}] <{}> <{}> push {}:{}:{} {:?}",
                state.plan().simulator.me().id(), state.plan().order_id, state.id(),
                state.plan().simulator.current_time(), state.plan().simulator.current_tick(),
                state.plan().simulator.current_micro_tick(), transition
            );
        }

        result
    }

    fn apply(&mut self, iteration: usize, state: &State<'c, 'a, G>, transition: &Transition) -> State<'c, 'a, G> {
        let mut result = match transition {
            Transition::ForkBall(_) => self.fork(state),
            Transition::ForkRobot(_) => self.fork(state),
            _ => self.use_scenario(state, transition),
        };

        log!(
            state.plan().current_tick, "[{}] <{}> <{}> transition {}:{}:{} {:?} -> <{}> {:?}",
            state.plan().simulator.me().id(), state.plan().order_id, state.id(),
            state.plan().simulator.current_time(), state.plan().simulator.current_tick(),
            state.plan().simulator.current_micro_tick(), transition, result.id(), result
        );

        #[cfg(feature = "enable_stats")]
        {
            result.plan_mut().stats.iteration = iteration;
            result.plan_mut().stats.path.push(transition.name());
        }

        result
    }

    fn get_transition_cost(&mut self, source_state: &State<'c, 'a, G>, destination_state: &State<'c, 'a, G>, transition: &Transition) -> i32 {
        match transition {
            Transition::ForkBall(_) => 0,
            Transition::Observe(_) => 1,
            _ => source_state.score() - destination_state.score(),
        }
    }

    fn get_score(&self, state: &State<'c, 'a, G>) -> i32 {
        state.plan().get_score()
    }
}

#[derive(Clone)]
pub enum State<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    Initial(Final<'c, 'a, G>),
    ObservedBall(ObservedBall<'c, 'a, G>),
    ObservedRobot(ObservedRobot<'c, 'a, G>),
    ForkedBall(ForkedBall<'c, 'a, G>),
    ForkedRobot(ForkedRobot<'c, 'a, G>),
    Walked(Final<'c, 'a, G>),
    Jumped(Final<'c, 'a, G>),
    FarJumped(Final<'c, 'a, G>),
    Hit(Final<'c, 'a, G>),
    End(Final<'c, 'a, G>),
}

impl<'c, 'a, G> State<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub fn initial(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::Initial(Final { id, score: 0, plan })
    }

    pub fn observed_ball(id: i32, number: usize, plan: Plan<'c, 'a, G>, initial_plan: Plan<'c, 'a, G>) -> Self {
        State::ObservedBall(ObservedBall { id, number, score: 0, plan, initial_plan })
    }

    pub fn observed_robot(id: i32, number: usize, robot_id: i32, plan: Plan<'c, 'a, G>, initial_plan: Plan<'c, 'a, G>) -> Self {
        State::ObservedRobot(ObservedRobot { id, number, robot_id, score: 0, plan, initial_plan })
    }

    pub fn forked_ball(id: i32, plan: Plan<'c, 'a, G>, observe_simulator: Simulator) -> Self {
        State::ForkedBall(ForkedBall { id, score: plan.get_score(), plan, observe_simulator })
    }

    pub fn forked_robot(id: i32, robot_id: i32, plan: Plan<'c, 'a, G>, observe_simulator: Simulator) -> Self {
        State::ForkedRobot(ForkedRobot { id, score: plan.get_score(), robot_id, plan, observe_simulator })
    }

    pub fn walked(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::Walked(Final { id, score: plan.get_score(), plan })
    }

    pub fn jumped(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::Jumped(Final { id, score: plan.get_score(), plan })
    }

    pub fn far_jumped(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::FarJumped(Final { id, score: plan.get_score(), plan })
    }

    pub fn hit(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::Hit(Final { id, score: plan.get_score(), plan })
    }

    pub fn end(id: i32, plan: Plan<'c, 'a, G>) -> Self {
        State::End(Final { id, score: plan.get_score(), plan })
    }

    pub fn id(&self) -> i32 {
        match self {
            State::Initial(v) => v.id,
            State::ObservedBall(v) => v.id,
            State::ObservedRobot(v) => v.id,
            State::ForkedBall(v) => v.id,
            State::ForkedRobot(v) => v.id,
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
            State::ObservedBall(v) => v.score,
            State::ObservedRobot(v) => v.score,
            State::ForkedBall(v) => v.score,
            State::ForkedRobot(v) => v.score,
            State::Walked(v) => v.score,
            State::Jumped(v) => v.score,
            State::FarJumped(v) => v.score,
            State::Hit(v) => v.score,
            State::End(v) => v.score,
        }
    }

    pub fn plan(&self) -> &Plan<'c, 'a, G> {
        match self {
            State::Initial(v) => &v.plan,
            State::ObservedBall(v) => &v.plan,
            State::ObservedRobot(v) => &v.plan,
            State::ForkedBall(v) => &v.plan,
            State::ForkedRobot(v) => &v.plan,
            State::Walked(v) => &v.plan,
            State::Jumped(v) => &v.plan,
            State::FarJumped(v) => &v.plan,
            State::Hit(v) => &v.plan,
            State::End(v) => &v.plan,
        }
    }

    pub fn plan_mut(&mut self) -> &mut Plan<'c, 'a, G> {
        match self {
            State::Initial(v) => &mut v.plan,
            State::ObservedBall(v) => &mut v.plan,
            State::ObservedRobot(v) => &mut v.plan,
            State::ForkedBall(v) => &mut v.plan,
            State::ForkedRobot(v) => &mut v.plan,
            State::Walked(v) => &mut v.plan,
            State::Jumped(v) => &mut v.plan,
            State::FarJumped(v) => &mut v.plan,
            State::Hit(v) => &mut v.plan,
            State::End(v) => &mut v.plan,
        }
    }

    pub fn take_plan(self) -> Plan<'c, 'a, G> {
        match self {
            State::Initial(v) => v.plan,
            State::ObservedBall(v) => v.plan,
            State::ObservedRobot(v) => v.plan,
            State::ForkedBall(v) => v.plan,
            State::ForkedRobot(v) => v.plan,
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
            State::ObservedBall(_) => "ObservedBall",
            State::ObservedRobot(_) => "ObservedRobot",
            State::ForkedBall(_) => "ForkedBall",
            State::ForkedRobot(_) => "ForkedRobot",
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

    pub fn get_initial_plan(&self) -> &Plan<'c, 'a, G> {
        match self {
            State::ObservedBall(v) => &v.initial_plan,
            State::ObservedRobot(v) => &v.initial_plan,
            _ => self.plan(),
        }
    }
}

impl<'c, 'a, G> std::fmt::Debug for State<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'c, 'a, G> Identifiable for State<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    fn id(&self) -> i32 {
        Self::id(self)
    }
}

#[derive(Clone)]
pub struct Final<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub plan: Plan<'c, 'a, G>,
}

#[derive(Clone)]
pub struct ObservedBall<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub number: usize,
    pub plan: Plan<'c, 'a, G>,
    pub initial_plan: Plan<'c, 'a, G>,
}

#[derive(Clone)]
pub struct ObservedRobot<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub robot_id: i32,
    pub number: usize,
    pub plan: Plan<'c, 'a, G>,
    pub initial_plan: Plan<'c, 'a, G>,
}

#[derive(Clone)]
pub struct ForkedBall<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub plan: Plan<'c, 'a, G>,
    pub observe_simulator: Simulator,
}

#[derive(Clone)]
pub struct ForkedRobot<'c, 'a, G>
    where G: Clone + Fn(i32, i32) -> Option<&'a Action> {

    pub id: i32,
    pub score: i32,
    pub robot_id: i32,
    pub plan: Plan<'c, 'a, G>,
    pub observe_simulator: Simulator,
}

#[derive(Clone, Debug)]
pub enum Transition {
    Observe(Observe),
    ForkBall(ForkBall),
    ForkRobot(ForkRobot),
    WalkToPosition(WalkToPosition),
    Jump(Jump),
    FarJump(FarJump),
    WatchMeJump(WatchMeJump),
    WatchBallMove(WatchBallMove),
    PushRobot(PushRobot),
    TakeNitroPack(WalkToPosition),
    WalkToBall(WalkToBall),
    WalkToRobot(WalkToRobot),
}

impl Transition {
    pub fn observe(number: usize, wait_until: f64, max_ball_z: f64) -> Self {
        Transition::Observe(Observe { number, wait_until, max_z: max_ball_z })
    }

    pub fn fork_ball() -> Self {
        Transition::ForkBall(ForkBall {})
    }

    pub fn fork_robot(robot_id: i32) -> Self {
        Transition::ForkRobot(ForkRobot { robot_id })
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

    pub fn take_nitro_pack(target: Vec3, max_speed: f64) -> Self {
        Transition::TakeNitroPack(WalkToPosition { target, max_speed })
    }

    pub fn walk_to_ball(direction: Vec3, allow_nitro: bool) -> Self {
        Transition::WalkToBall(WalkToBall { direction, allow_nitro })
    }

    pub fn walk_to_robot(robot_id: i32, direction: Vec3, allow_nitro: bool) -> Self {
        Transition::WalkToRobot(WalkToRobot { robot_id, direction, allow_nitro })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Transition::Observe(_) => "observe",
            Transition::WalkToPosition(_) => "walk_to_position",
            Transition::Jump(_) => "jump",
            Transition::FarJump(_) => "far_jump",
            Transition::WatchMeJump(_) => "watch_me_jump",
            Transition::WatchBallMove(_) => "watch_ball_move",
            Transition::PushRobot(_) => "push_robot",
            Transition::TakeNitroPack(_) => "take_nitro_pack",
            Transition::ForkBall(_) => "fork_ball",
            Transition::WalkToBall(_) => "walk_to_ball",
            Transition::WalkToRobot(_) => "walk_to_robot",
            Transition::ForkRobot(_) => "fork_robot",
        }
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
            Transition::TakeNitroPack(v) => v.perform(ctx),
            Transition::WalkToBall(v) => v.perform(ctx),
            Transition::WalkToRobot(v) => v.perform(ctx),
            Transition::ForkBall(_) => unimplemented!(),
            Transition::ForkRobot(_) => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ForkBall;

#[derive(Debug, Clone)]
pub struct ForkRobot {
    pub robot_id: i32,
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
