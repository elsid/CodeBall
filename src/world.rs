use crate::model::{Robot, Game, Rules};

#[derive(Debug)]
pub struct World {
    pub me: Robot,
    pub rules: Rules,
    pub game: Game,
}

impl World {
    pub fn new(me: Robot, rules: Rules, game: Game) -> Self {
        World {
            me: me,
            rules: rules,
            game: game,
        }
    }

    pub fn update(&mut self, me: &Robot, game: &Game) {
        self.me = me.clone();
        self.game = game.clone();
    }
}
