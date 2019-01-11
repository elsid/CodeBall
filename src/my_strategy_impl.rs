use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::world::World;
use crate::my_strategy::orders::Order;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

pub struct MyStrategyImpl {
    world: World,
    rng: XorShiftRng,
//    start_time: Instant,
//    tick_start_time: Instant,
//    cpu_time_spent: Duration,
    last_tick: i32,
    order: Option<Order>,
    #[cfg(feature = "enable_render")]
    render: Render,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, _rules: &Rules, game: &Game, action: &mut Action) {
//        self.tick_start_time = if game.current_tick == 0 {
//            self.start_time
//        } else {
//            Instant::now()
//        };
        if self.last_tick != game.current_tick {
            self.last_tick = game.current_tick;
            self.update_world(me, game);
            if self.world.is_reset_ticks() {
                self.order = None;
            } else {
                self.give_orders();
            }
            #[cfg(feature = "enable_stats")]
            for v in self.order.iter() {
                println!("{}", serde_json::to_string(&v.stats).unwrap());
            }
            #[cfg(feature = "enable_render")]
            self.render();
        } else {
            self.update_world_me(me);
        }
        if !self.world.is_reset_ticks() {
            self.apply_action(action);
        }
//        let finish = Instant::now();
//        let cpu_time_spent = finish - self.tick_start_time;
//        self.cpu_time_spent += cpu_time_spent;
    }
}

impl MyStrategyImpl {
//    pub fn new(me: &Robot, rules: &Rules, game: &Game, start_time: Instant) -> Self {
    pub fn new(me: &Robot, rules: &Rules, game: &Game) -> Self {
        let world = World::new(me.clone(), rules.clone(), game.clone());
        log!(game.current_tick, "start");
        MyStrategyImpl {
            world: world.clone(),
            rng: XorShiftRng::from_seed([
                rules.seed as u32,
                (rules.seed >> 32) as u32,
                1841971383,
                1904458926,
            ]),
//            start_time,
//            tick_start_time: start_time,
//            cpu_time_spent: Duration::default(),
            last_tick: -1,
            order: None,
            #[cfg(feature = "enable_render")]
            render: Render::new(),
        }
    }

    #[cfg(feature = "enable_render")]
    pub fn get_render(&self) -> &Render {
        &self.render
    }

    fn update_world(&mut self, me: &Robot, game: &Game) {
        self.world.update(me, game);
    }

    fn update_world_me(&mut self, me: &Robot) {
        self.world.me = me.clone();
    }

    fn give_orders(&mut self) {
        let world = &self.world;
        let rng = &mut self.rng;
        self.order = if let Some(action) = &self.order {
            let robot = world.get_robot(action.robot_id);
            let current_robot_action = Order::new(robot, world, rng);
            if let Some(a) = current_robot_action {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != action.robot_id)
                    .filter_map(|v| Order::new(v, world, rng))
                    .max_by_key(|v| v.score)
                    .filter(|v| v.score > a.score + 100)
                    .or(Some(a))
            } else {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != action.robot_id)
                    .filter_map(|v| Order::new(v, world, rng))
                    .max_by_key(|v| v.score)
            }
        } else {
            world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .filter_map(|v| Order::new(v, world, rng))
                .max_by_key(|v| v.score)
        };
    }

    fn apply_action(&mut self, action: &mut Action) {
        let action_applied = self.order.iter()
            .find(|v| v.robot_id == self.world.me.id)
            .map(|v| {
                *action = v.action.clone();
                log!(self.world.game.current_tick, "[{}] <{}> apply order {:?}", self.world.me.id, v.id, action);
            })
            .is_some();
        if action_applied {
            return;
        }
        let target = self.world.rules.get_goalkeeper_position();
        let to_target = target - self.world.me.position();
        let velocity = if to_target.norm() > self.world.rules.min_running_distance() {
            to_target.normalized() * self.world.rules.ROBOT_MAX_GROUND_SPEED
        } else {
            to_target
        };
        action.set_target_velocity(velocity);
        log!(self.world.game.current_tick, "[{}] apply default action {:?}", self.world.me.id, action);
    }

    #[cfg(feature = "enable_render")]
    fn render(&mut self) {
        self.render.clear();

        let mut robots: Vec<&Robot> = self.world.game.robots.iter().map(|v| v).collect();
        robots.sort_by_key(|v| v.id);

        let render = &mut self.render;

        for robot in robots {
            robot.render(render);

            let order = self.order.iter()
                .find(|v| v.robot_id == robot.id)
                .map(|v| v);

            if let Some(order) = order {
                order.render(robot, render);
            }
        }

        self.world.game.ball.render(render);
    }

//    fn real_time_spent(&self) -> Duration {
//        Instant::now() - self.start_time
//    }
//
//    fn cpu_time_spent(&self) -> Duration {
//        self.cpu_time_spent + (Instant::now() - self.tick_start_time)
//    }
}
