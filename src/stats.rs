#[derive(Debug, PartialEq, Default, Serialize, Clone)]
pub struct Stats {
    pub player_id: i32,
    pub robot_id: i32,
    pub current_tick: i32,
    pub order: &'static str,
    pub time_to_jump: f64,
    pub time_to_watch: f64,
    pub time_to_end: f64,
    pub time_to_score: Option<f64>,
    pub iteration: usize,
    pub total_iterations: usize,
    pub game_score: i32,
    pub order_score: i32,
    pub path_micro_ticks: usize,
    pub plan_micro_ticks: usize,
    pub game_micro_ticks: usize,
    pub game_micro_ticks_limit: usize,
    pub current_step: usize,
    pub reached_game_limit: bool,
    pub reached_plan_limit: bool,
    pub reached_path_limit: bool,
    pub other_number: usize,
    pub ticks_with_near_micro_ticks: usize,
    pub ticks_with_far_micro_ticks: usize,
    pub path_type: Option<&'static str>,
}

impl Stats {
    pub fn new(player_id: i32, robot_id: i32, current_tick: i32, order: &'static str) -> Self {
        let mut result = Stats::default();
        result.player_id = player_id;
        result.robot_id = robot_id;
        result.current_tick = current_tick;
        result.order = order;
        result
    }

    pub fn update(&mut self, other: &Stats) {
        self.path_micro_ticks += other.path_micro_ticks;
        self.ticks_with_far_micro_ticks += other.ticks_with_far_micro_ticks;
        self.ticks_with_near_micro_ticks += other.ticks_with_near_micro_ticks;
        self.current_step = other.current_step;
    }
}
