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

#[path = "mat3.rs"]
pub mod mat3;

#[path = "line2.rs"]
pub mod line2;

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

#[path = "nitro_pack.rs"]
pub mod nitro_pack;

#[path = "player.rs"]
pub mod player;

#[path = "game.rs"]
pub mod game;

#[path = "config.rs"]
pub mod config;

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

#[path = "search.rs"]
pub mod search;

#[path = "plan.rs"]
pub mod plan;

#[path = "roles.rs"]
pub mod roles;

#[path = "orders.rs"]
pub mod orders;

#[cfg(not(feature = "use_test_strategy"))]
#[path = "my_strategy_impl.rs"]
pub mod my_strategy_impl;

#[cfg(feature = "use_test_strategy")]
#[path = "my_test_strategy_impl.rs"]
pub mod my_test_strategy_impl;

#[cfg(feature = "use_goalkeeper_strategy")]
#[path = "my_goalkeeper_strategy_impl.rs"]
pub mod my_goalkeeper_strategy_impl;

#[cfg(feature = "use_forward_strategy")]
#[path = "my_forward_strategy_impl.rs"]
pub mod my_forward_strategy_impl;

use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;

#[cfg(feature = "use_test_strategy")]
use self::my_test_strategy_impl::MyStrategyImpl;

#[cfg(feature = "use_goalkeeper_strategy")]
use self::my_goalkeeper_strategy_impl::MyStrategyImpl;

#[cfg(feature = "use_forward_strategy")]
use self::my_forward_strategy_impl::MyStrategyImpl;

#[cfg(all(not(feature = "use_test_strategy"), not(feature = "use_goalkeeper_strategy"), not(feature = "use_forward_strategy")))]
use self::my_strategy_impl::MyStrategyImpl;

pub struct MyStrategy {
    strategy_impl: Option<MyStrategyImpl>,
}

impl Strategy for MyStrategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        use self::config::Config;

        if self.strategy_impl.is_none() {
            let config: Config = if cfg!(feature = "read_config") {
                serde_json::from_str(
                    std::fs::read_to_string(
                        std::env::var("CONFIG").expect("CONFIG env is not found")
                    ).expect("Can't read config file").as_str()
                ).expect("Can't parse config file")
            } else {
                Config::default()
            };
            self.strategy_impl = Some(MyStrategyImpl::new(config, me, rules, game));
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
            strategy_impl: None,
        }
    }
}
