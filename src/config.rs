#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub max_ticks: i32,
    pub near_micro_ticks_per_tick: usize,
    pub far_micro_ticks_per_tick: usize,
    pub max_observations: usize,
    pub ticks_per_steps: Vec<usize>,
    pub max_iterations: usize,
    pub max_path_micro_ticks: usize,
    pub max_plan_micro_ticks: usize,
    pub max_act_micro_ticks: usize,
    pub robot_priority_change_gap: i32,
    pub robot_role_change_gap: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_ticks: 100,
            near_micro_ticks_per_tick: 25,
            far_micro_ticks_per_tick: 3,
            max_observations: 6,
            ticks_per_steps: vec![1, 3, 4, 8],
            max_iterations: 100,
            max_path_micro_ticks: 1100,
            max_plan_micro_ticks: 40000,
            max_act_micro_ticks: 15000,
            robot_priority_change_gap: 10,
            robot_role_change_gap: 0,
        }
    }
}
