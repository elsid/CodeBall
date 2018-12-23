use std::time::{Instant, Duration};
use crate::model::{Game, Action, Robot, Rules, Ball, Arena};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::world::World;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::simulator::{Simulator, Solid};
use crate::my_strategy::render::{Render, Tag};
use crate::my_strategy::optimal_action::OptimalAction;

pub struct MyStrategyImpl {
    game: Game,
    world: World,
    rng: XorShiftRng,
    max_ticks_count: i32,
    start_time: Instant,
    tick_start_time: Instant,
    cpu_time_spent: Duration,
    plan_orders_max_cpu_time: Duration,
    last_tick: i32,
    actions: Vec<(i32, OptimalAction)>,
    render: Render,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        use std::process::exit;
        if game.current_tick >= self.max_ticks_count {
            exit(1);
        }
        self.tick_start_time = if game.current_tick == 0 {
            self.start_time
        } else {
            Instant::now()
        };
        if self.last_tick != game.current_tick {
            self.last_tick = game.current_tick;
            self.actions.clear();
            self.render.clear();
            self.render.include_tag(Tag::RobotId(1));
            self.update_world(me, game);
            self.generate_actions();
        } else {
            self.update_world_me(me);
        }
        self.apply_action(action);
        let finish = Instant::now();
        let cpu_time_spent = finish - self.tick_start_time;
        self.cpu_time_spent += cpu_time_spent;
    }
}

impl MyStrategyImpl {
    pub fn new(me: &Robot, rules: &Rules, game: &Game, start_time: Instant) -> Self {
        use std::env;
        use std::i32;
        let world = World::new(me.clone(), rules.clone(), game.clone());
        MyStrategyImpl {
            game: game.clone(),
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
            start_time,
            tick_start_time: start_time,
            cpu_time_spent: Duration::default(),
            plan_orders_max_cpu_time: Duration::default(),
            last_tick: -1,
            actions: Vec::new(),
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
        let actions = &mut self.actions;
        let render= &mut self.render;
        let rng = &mut self.rng;
        for (id, action) in world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .map(|v| (v.id, v.get_optimal_action(world, rng, render)))
        {
            actions.push((id, action));
        }
    }

    fn apply_action(&mut self, action: &mut Action) {
        let mut action_applied = false;
        self.actions.iter()
            .find(|(id, _)| *id == self.world.me.id)
            .map(|(_, v)| {
                *action = v.action.clone();
                action_applied = true;
                log!(self.world.game.current_tick, "[{}] <{}> apply optimal action {:?}", self.world.me.id, v.id, action);
            });
        if action_applied {
            return;
        }
        let target = self.world.rules.arena.get_defend_target();
        let velocity = (target - self.world.me.position()).normalized()
            * self.world.rules.ROBOT_MAX_GROUND_SPEED;
        action.set_target_velocity(velocity);
        log!(self.world.game.current_tick, "[{}] apply default action {:?}", self.world.me.id, action);
    }

    fn real_time_spent(&self) -> Duration {
        Instant::now() - self.start_time
    }

    fn cpu_time_spent(&self) -> Duration {
        self.cpu_time_spent + (Instant::now() - self.tick_start_time)
    }
}
