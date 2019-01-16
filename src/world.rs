use crate::model::{Robot, Game, Rules};

const MICRO_TICKS_LIMIT_PER_TICK: usize = 15000;

#[derive(Debug, Clone)]
pub struct World {
    pub me: Robot,
    pub rules: Rules,
    pub game: Game,
    reset_ticks_left: usize,
}

impl World {
    pub fn new(me: Robot, rules: Rules, game: Game) -> Self {
        World {me, rules, game, reset_ticks_left: 0}
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
        (self.game.current_tick + 2) as usize * MICRO_TICKS_LIMIT_PER_TICK
    }
}
