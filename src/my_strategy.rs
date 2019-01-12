#[allow(dead_code)]
#[path = "random.rs"]
pub mod random;

#[path = "common.rs"]
#[macro_use]
pub mod common;

#[path = "vec2.rs"]
pub mod vec2;

#[path = "vec3.rs"]
pub mod vec3;

#[path = "plane.rs"]
pub mod plane;

#[path = "sphere.rs"]
pub mod sphere;

#[path = "optimization.rs"]
mod optimization;

#[cfg(feature = "enable_render")]
#[path = "render.rs"]
pub mod render;

#[path = "arena.rs"]
pub mod arena;

#[path = "rules.rs"]
pub mod rules;

#[path = "action.rs"]
pub mod action;

#[path = "entity.rs"]
pub mod entity;

#[path = "ball.rs"]
pub mod ball;

#[path = "robot.rs"]
pub mod robot;

#[path = "world.rs"]
pub mod world;

#[cfg(feature = "enable_stats")]
#[path = "stats.rs"]
pub mod stats;

#[path = "physics.rs"]
pub mod physics;

#[path = "simulator.rs"]
pub mod simulator;

#[path = "scenarios.rs"]
pub mod scenarios;

#[path = "orders.rs"]
pub mod orders;

#[cfg(not(feature = "use_test_strategy"))]
#[path = "my_strategy_impl.rs"]
pub mod my_strategy_impl;

#[cfg(feature = "use_test_strategy")]
#[path = "my_test_strategy_impl.rs"]
pub mod my_test_strategy_impl;

use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;

#[cfg(feature = "use_test_strategy")]
use self::my_test_strategy_impl::MyStrategyImpl;

#[cfg(not(feature = "use_test_strategy"))]
use self::my_strategy_impl::MyStrategyImpl;

pub struct MyStrategy {
//    start_time: Instant,
    strategy_impl: Option<MyStrategyImpl>,
}

impl Strategy for MyStrategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        if self.strategy_impl.is_none() {
//            self.strategy_impl = Some(MyStrategyImpl::new(me, rules, game, self.start_time));
            self.strategy_impl = Some(MyStrategyImpl::new(me, rules, game));
        }
        self.strategy_impl.as_mut().unwrap().act(me, rules, game, action);
    }

    fn custom_rendering(&mut self) -> String {
        #[cfg(feature = "enable_render")]
        {
            if let Some(v) = &self.strategy_impl {
                serde_json::to_string(v.get_render()).unwrap()
            } else {
                String::new()
            }
        }
        #[cfg(not(feature = "enable_render"))]
        {
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
