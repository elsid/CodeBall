use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::vec3::Vec3;

pub struct BallState {
    pub position: Vec3,
}

pub struct RobotState {
    pub id: i32,
    pub radius: f64,
    pub position: Vec3,
    pub velocity: Vec3,
}

pub struct State {
    pub time: f64,
    pub ball: BallState,
    pub me: RobotState,
    pub robots: Vec<RobotState>,
}

impl State {
    pub fn new(simulator: &Simulator) -> Self {
        use crate::my_strategy::entity::Entity;
        use crate::my_strategy::simulator::Solid;

        let ball = simulator.ball().base();
        let me = simulator.me().base();
        let mut robots: Vec<RobotState> = simulator.robots().iter()
            .filter(|v| !v.is_me())
            .map(|v| RobotState {
                id: v.id(),
                radius: v.radius(),
                position: v.position(),
                velocity: v.velocity(),
            })
            .collect();
        robots.sort_by_key(|v| v.id);
        State {
            time: simulator.current_time(),
            ball: BallState {
                position: ball.position(),
            },
            me: RobotState {
                id: me.id,
                radius: me.radius,
                position: me.position(),
                velocity: me.velocity(),
            },
            robots,
        }
    }
}

#[derive(Default, Serialize)]
pub struct Stats {
    pub micro_ticks_to_jump: i32,
    pub micro_ticks_to_watch: i32,
    pub micro_ticks_to_end: i32,
    pub time_to_jump: f64,
    pub time_to_watch: f64,
    pub time_to_end: f64,
    pub time_to_score: Option<f64>,
    pub iteration: usize,
    pub total_iterations: usize,
    pub score: i32,
    pub jump_simulation: bool,
    pub far_jump_simulation: bool,
    pub action_score: i32,
    pub total_micro_ticks: i32,
    pub current_step: i32,
}
