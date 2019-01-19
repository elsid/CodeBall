#[derive(Debug, PartialEq, Default, Serialize, Clone)]
pub struct Stats {
    pub player_id: i32,
    pub robot_id: i32,
    pub current_tick: i32,
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
    pub current_step: usize,
    pub reached_game_limit: bool,
    pub reached_play_limit: bool,
    pub reached_scenario_limit: bool,
}

impl Stats {
    pub fn new(player_id: i32, robot_id: i32, current_tick: i32) -> Self {
        let mut result = Stats::default();
        result.player_id = player_id;
        result.robot_id = robot_id;
        result.current_tick = current_tick;
        result
    }
}
