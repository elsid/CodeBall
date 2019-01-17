use crate::model::Player;

impl Player {
    pub fn opposite(&self) -> Player {
        Player {
            id: self.id,
            me: !self.me,
            strategy_crashed: self.strategy_crashed,
            score: self.score,
        }
    }
}
