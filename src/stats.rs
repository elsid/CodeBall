#[derive(Debug, PartialEq, Default, Serialize, Clone)]
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
