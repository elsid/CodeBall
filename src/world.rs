use crate::model::{Robot, Game, Rules};
use crate::my_strategy::config::Config;

#[derive(Debug, Clone)]
pub struct World {
    config: Config,
    pub me: Robot,
    pub rules: Rules,
    pub game: Game,
    reset_ticks_left: usize,
}

impl World {
    pub fn new(config: Config, me: Robot, rules: Rules, game: Game) -> Self {
        World { config, me, rules, game, reset_ticks_left: 0 }
    }

    pub fn update(&mut self, me: &Robot, game: &Game) {
        let prev_score = self.game.players.iter().map(|v| v.score).sum::<i32>();
        let curr_score = game.players.iter().map(|v| v.score).sum::<i32>();
        if prev_score < curr_score {
            self.reset_ticks_left = self.rules.RESET_TICKS;
        }
        self.me = me.clone();
        self.game = game.clone();
        self.reset_ticks_left = if self.reset_ticks_left > 0 { self.reset_ticks_left - 1 } else { 0 };
    }

    pub fn is_reset_ticks(&self) -> bool {
        self.reset_ticks_left > 0
    }

    pub fn get_robot(&self, id: i32) -> &Robot {
        self.game.robots.iter()
            .find(|v| v.id == id)
            .unwrap()
    }

    pub fn is_micro_ticks_limit_reached(&self, micro_ticks: usize) -> bool {
        micro_ticks > self.get_micro_ticks_limit()
    }

    pub fn get_micro_ticks_limit(&self) -> usize {
        (self.game.current_tick + 2) as usize * self.config.max_act_micro_ticks
    }

    pub fn opposite(&self) -> Self {
        World {
            config: self.config.clone(),
            me: self.me.opposite(),
            rules: self.rules.clone(),
            game: self.game.opposite(),
            reset_ticks_left: self.reset_ticks_left,
        }
    }

    pub fn is_teammate(&self, robot_id: i32) -> bool {
        self.get_robot(robot_id).is_teammate
    }
}
