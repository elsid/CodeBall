use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::world::World;
use crate::my_strategy::render::{Render, Tag};
use crate::my_strategy::optimal_action::OptimalAction;

pub struct MyStrategyImpl {
    world: World,
    rng: XorShiftRng,
    max_ticks_count: i32,
//    start_time: Instant,
//    tick_start_time: Instant,
//    cpu_time_spent: Duration,
    last_tick: i32,
    optimal_action: Option<OptimalAction>,
    render: Render,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Drop for MyStrategyImpl {
    fn drop(&mut self) {
        if cfg!(feature = "dump_result") {
            println!("{}", serde_json::to_string(&self.world.game.players).unwrap());
        }
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, _rules: &Rules, game: &Game, action: &mut Action) {
        use std::process::exit;
        if game.current_tick >= self.max_ticks_count {
            exit(1);
        }
//        self.tick_start_time = if game.current_tick == 0 {
//            self.start_time
//        } else {
//            Instant::now()
//        };
        if self.last_tick != game.current_tick {
            self.last_tick = game.current_tick;
            self.render.clear();
            self.update_world(me, game);
            self.generate_actions();
            self.render.ignore_all();
            for v in self.optimal_action.iter() {
                self.render.include_tag(Tag::RobotId(v.robot_id));
            }
        } else {
            self.update_world_me(me);
        }
        self.apply_action(action);
//        let finish = Instant::now();
//        let cpu_time_spent = finish - self.tick_start_time;
//        self.cpu_time_spent += cpu_time_spent;
    }
}

impl MyStrategyImpl {
//    pub fn new(me: &Robot, rules: &Rules, game: &Game, start_time: Instant) -> Self {
    pub fn new(me: &Robot, rules: &Rules, game: &Game) -> Self {
        use std::env;
        use std::i32;
        let world = World::new(me.clone(), rules.clone(), game.clone());
        log!(game.current_tick, "start");
        MyStrategyImpl {
            world: world.clone(),
            rng: XorShiftRng::from_seed([
                rules.seed as u32,
                (rules.seed >> 32) as u32,
                0,
                0,
            ]),
            max_ticks_count: if let Ok(v) = env::var("MAX_TICKS") {
                if let Ok(v_v) = v.parse::<i32>() {
                    v_v
                } else {
                    i32::MAX
                }
            } else {
                i32::MAX
            },
//            start_time,
//            tick_start_time: start_time,
//            cpu_time_spent: Duration::default(),
            last_tick: -1,
            optimal_action: None,
            render: Render::new(),
        }
    }

    pub fn render(&self) -> &Render {
        &self.render
    }

    fn update_world(&mut self, me: &Robot, game: &Game) {
        self.world.update(me, game);
    }

    fn update_world_me(&mut self, me: &Robot) {
        self.world.me = me.clone();
    }

    fn generate_actions(&mut self) {
        let world = &self.world;
        let rng = &mut self.rng;
        let render= &mut self.render;
        self.optimal_action = if let Some(action) = &self.optimal_action {
            let current_robot_action = world.game.robots.iter()
                .find(|v| v.id == action.robot_id)
                .unwrap()
                .get_optimal_action(world, rng, render);
            world.game.robots.iter()
                .filter(|v| v.is_teammate && v.id != action.robot_id)
                .map(|v| v.get_optimal_action(world, rng, render))
                .max_by_key(|v| v.score)
                .filter(|v| v.score > current_robot_action.score + 100)
                .or(Some(current_robot_action))
        } else {
            world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .map(|v| v.get_optimal_action(world, rng, render))
                .max_by_key(|v| v.score)
        };
    }

    fn apply_action(&mut self, action: &mut Action) {
        let action_applied = self.optimal_action.iter()
            .find(|v| v.robot_id == self.world.me.id)
            .map(|v| {
                *action = v.action.clone();
                log!(self.world.game.current_tick, "[{}] <{}> apply optimal action {:?}", self.world.me.id, v.id, action);
            })
            .is_some();
        if action_applied {
            return;
        }
        let target = self.world.rules.arena.get_defend_target();
        let velocity = (target - self.world.me.position()).normalized()
            * self.world.rules.ROBOT_MAX_GROUND_SPEED;
        action.set_target_velocity(velocity);
        log!(self.world.game.current_tick, "[{}] apply default action {:?}", self.world.me.id, action);
    }

//    fn real_time_spent(&self) -> Duration {
//        Instant::now() - self.start_time
//    }
//
//    fn cpu_time_spent(&self) -> Duration {
//        self.cpu_time_spent + (Instant::now() - self.tick_start_time)
//    }
}
