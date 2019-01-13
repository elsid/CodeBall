use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::world::World;
use crate::my_strategy::orders::Order;
use crate::my_strategy::common::IdGenerator;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

pub struct MyStrategyImpl {
    world: World,
    rng: XorShiftRng,
//    start_time: Instant,
//    tick_start_time: Instant,
//    cpu_time_spent: Duration,
    last_tick: i32,
    active_order: Option<Order>,
    passive_orders: Vec<Order>,
    order_id_generator: IdGenerator,
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
                self.active_order = None;
            } else {
                self.give_orders();
            }
            #[cfg(feature = "enable_stats")]
            for v in self.active_order.iter() {
                println!("{}", serde_json::to_string(&v.stats()).unwrap());
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
            active_order: None,
            passive_orders: Vec::new(),
            order_id_generator: IdGenerator::new(),
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
        let order_id_generator = &mut self.order_id_generator;
        self.active_order = if let Some(order) = &self.active_order {
            let robot = world.get_robot(order.robot_id());
            let current_robot_order = Order::try_play(robot, world, rng, order_id_generator);
            if let Some(current) = current_robot_order {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != order.robot_id())
                    .filter_map(|v| Order::try_play(v, world, rng, order_id_generator))
                    .max_by_key(|v| v.score())
                    .filter(|v| v.score() > current.score() + 100)
                    .or(Some(current))
            } else {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != order.robot_id())
                    .filter_map(|v| Order::try_play(v, world, rng, order_id_generator))
                    .max_by_key(|v| v.score())
            }
        } else {
            world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .filter_map(|v| Order::try_play(v, world, rng, order_id_generator))
                .max_by_key(|v| v.score())
        };
        let active_order = self.active_order.as_ref();
        self.passive_orders = world.game.robots.iter()
            .filter(|robot| {
                active_order
                    .map(|v| v.robot_id() != robot.id)
                    .unwrap_or(true)
            })
            .map(|robot| {
                Order::walk_to_goalkeeper_position(robot, world, order_id_generator)
            })
            .collect();
    }

    fn apply_action(&mut self, action: &mut Action) {
        self.active_order.iter()
            .find(|v| v.robot_id() == self.world.me.id)
            .map(|v| {
                *action = v.action().clone();
                log!(self.world.game.current_tick, "[{}] <{}> apply active order {:?}", self.world.me.id, v.id(), action);
            });
        self.passive_orders.iter()
            .find(|v| v.robot_id() == self.world.me.id)
            .map(|v| {
                *action = v.action().clone();
                log!(self.world.game.current_tick, "[{}] <{}> apply passive order {:?}", self.world.me.id, v.id(), action);
            });
    }

    #[cfg(feature = "enable_render")]
    fn render(&mut self) {
        self.render.clear();

        let mut robots: Vec<&Robot> = self.world.game.robots.iter().map(|v| v).collect();
        robots.sort_by_key(|v| v.id);

        let render = &mut self.render;

        for robot in robots {
            robot.render(render);

            let order = self.active_order.iter()
                .find(|v| v.robot_id() == robot.id)
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
