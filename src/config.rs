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
    pub goalkeeper_max_z_factor: f64,
}

impl Config {
    pub fn new(team_size: i32) -> Self {
        Config {
            max_ticks: 100,
            near_micro_ticks_per_tick: 25,
            far_micro_ticks_per_tick: 3,
            max_observations: 6,
            ticks_per_steps: vec![1, 3, 4, 8],
            max_iterations: if team_size <= 2 {
                150
            } else {
                100
            },
            max_path_micro_ticks: 1100,
            max_plan_micro_ticks: 40000,
            max_act_micro_ticks: 15000,
            robot_priority_change_gap: 10,
            robot_role_change_gap: 0,
            ball_goal_distance_score_weight: 1.2360679748635974,
            ball_goal_direction_score_weight: 0.0016993994285499695,
            my_time_to_ball_score_weight: 0.5000113249625928,
            time_to_goal_score_weight: 0.24999619213943386,
            opponent_time_to_ball_penalty_weight: 0.09999078428632137,
            nitro_amount_score_weight: 0.10000021192195667,
            goalkeeper_max_z_factor: 1.6666666666666667,
        }
    }
}
