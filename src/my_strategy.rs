#[path = "common.rs"]
#[macro_use]
mod common;

#[path = "vec3.rs"]
mod vec3;

#[path = "world.rs"]
mod world;

#[path = "my_strategy_impl.rs"]
mod my_strategy_impl;

use model::{Game, Action, Robot, Rules};
use strategy::Strategy;
use self::my_strategy_impl::MyStrategyImpl;

pub struct MyStrategy {
    strategy_impl: Option<MyStrategyImpl>,
}

impl Strategy for MyStrategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        if self.strategy_impl.is_none() {
            self.strategy_impl = Some(MyStrategyImpl::new(me, rules, game));
        }
        self.strategy_impl.as_mut().unwrap().act(me, rules, game, action);
    }
}

impl Default for MyStrategy {
    fn default() -> Self {
        MyStrategy {
            strategy_impl: None,
        }
    }
}
