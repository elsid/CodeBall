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
    pub ball_goal_distance_score_weight: f64,
    pub ball_goal_direction_score_weight: f64,
    pub my_time_to_ball_score_weight: f64,
    pub time_to_goal_score_weight: f64,
    pub opponent_time_to_ball_penalty_weight: f64,
    pub nitro_amount_score_weight: f64,
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
            ball_goal_distance_score_weight: 1.0,
            ball_goal_direction_score_weight: 0.1,
            my_time_to_ball_score_weight: 0.5,
            time_to_goal_score_weight: 0.25,
            opponent_time_to_ball_penalty_weight: 0.1,
            nitro_amount_score_weight: 0.1,
        }
    }
}