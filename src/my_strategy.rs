#[allow(dead_code)]
#[path = "random.rs"]
mod random;

#[path = "common.rs"]
#[macro_use]
mod common;

#[path = "vec2.rs"]
mod vec2;

#[path = "vec3.rs"]
mod vec3;

#[path = "plane.rs"]
mod plane;

#[path = "sphere.rs"]
mod sphere;

#[path = "render.rs"]
mod render;

#[path = "arena.rs"]
mod arena;

#[path = "rules.rs"]
mod rules;

#[path = "action.rs"]
mod action;

#[path = "entity.rs"]
mod entity;

#[path = "ball.rs"]
mod robot;

#[path = "robot.rs"]
mod ball;

#[path = "world.rs"]
mod world;

#[path = "simulator.rs"]
mod simulator;

#[path = "optimal_action.rs"]
mod optimal_action;

#[path = "my_strategy_impl.rs"]
mod my_strategy_impl;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use self::my_strategy_impl::MyStrategyImpl;

pub struct MyStrategy {
//    start_time: Instant,
    strategy_impl: Option<MyStrategyImpl>,
}

impl Strategy for MyStrategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        if self.strategy_impl.is_none() {
            log!(game.current_tick, "start");
//            self.strategy_impl = Some(MyStrategyImpl::new(me, rules, game, self.start_time));
            self.strategy_impl = Some(MyStrategyImpl::new(me, rules, game));
        }
        self.strategy_impl.as_mut().unwrap().act(me, rules, game, action);
    }

    fn custom_rendering(&mut self) -> String {
        if let Some(v) = &self.strategy_impl {
            serde_json::to_string(v.render()).unwrap()
        } else {
            String::new()
        }
    }
}

impl Default for MyStrategy {
    fn default() -> Self {
        MyStrategy {
//            start_time: Instant::now(),
            strategy_impl: None,
        }
    }
}
