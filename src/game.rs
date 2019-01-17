use crate::model::Game;

impl Game {
    pub fn opposite(&self) -> Game {
        Game {
            current_tick: self.current_tick,
            players: self.players.iter().map(|v| v.opposite()).collect(),
            robots: self.robots.iter().map(|v| v.opposite()).collect(),
            nitro_packs: self.nitro_packs.iter().map(|v| v.opposite()).collect(),
            ball: self.ball.opposite(),
        }
    }
}
