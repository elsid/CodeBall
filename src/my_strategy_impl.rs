use std::time::{Instant, Duration};
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
    start_time: Instant,
    tick_start_time: Instant,
    time_spent: Duration,
    cpu_time_spent: Duration,
    max_cpu_time_spent: Duration,
    last_tick: i32,
    active_order: Option<Order>,
    passive_orders: Vec<Order>,
    order_id_generator: IdGenerator,
    micro_ticks: usize,
    #[cfg(feature = "enable_render")]
    render: Render,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Drop for MyStrategyImpl {
    fn drop(&mut self) {
        eprintln!("{} {:?} {:?} {:?}", self.micro_ticks, self.time_spent, self.cpu_time_spent, self.max_cpu_time_spent);
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, _rules: &Rules, game: &Game, action: &mut Action) {
        self.on_start(game);
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
        self.on_finish();
    }
}

impl MyStrategyImpl {
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
            start_time: Instant::now(),
            tick_start_time: Instant::now(),
            time_spent: Duration::default(),
            cpu_time_spent: Duration::default(),
            max_cpu_time_spent: Duration::default(),
            last_tick: -1,
            active_order: None,
            passive_orders: Vec::new(),
            order_id_generator: IdGenerator::new(),
            micro_ticks: 0,
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
        use crate::my_strategy::orders::OrderContext;

        let world = &self.world;
        let mut ctx = OrderContext {
            rng: &mut self.rng,
            order_id_generator: &mut self.order_id_generator,
            micro_ticks: &mut self.micro_ticks,
        };
        self.active_order = if let Some(order) = &self.active_order {
            let robot = world.get_robot(order.robot_id());
            let current_robot_order = Order::try_play(robot, world, &mut ctx);
            if let Some(current) = current_robot_order {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != order.robot_id())
                    .filter_map(|v| Order::try_play(v, world, &mut ctx))
                    .max_by_key(|v| v.score())
                    .filter(|v| v.score() > current.score() + 116)
                    .or(Some(current))
            } else {
                world.game.robots.iter()
                    .filter(|v| v.is_teammate && v.id != order.robot_id())
                    .filter_map(|v| Order::try_play(v, world, &mut ctx))
                    .max_by_key(|v| v.score())
            }
        } else {
            world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .filter_map(|v| Order::try_play(v, world, &mut ctx))
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
                if robot.nitro_amount < world.rules.START_NITRO_AMOUNT
                    && world.game.ball.position().distance(world.rules.get_goalkeeper_position())
                        > world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS {

                    Order::try_take_nitro_pack(robot, world, ctx.order_id_generator)
                        .unwrap_or_else(|| {
                            Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator)
                        })
                } else {
                    Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator)
                }
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

    fn on_start(&mut self, game: &Game) {
        self.tick_start_time = if game.current_tick == 0 {
            self.start_time
        } else {
            Instant::now()
        };
    }

    fn on_finish(&mut self) {
        let finish = Instant::now();
        let cpu_time_spent = finish - self.tick_start_time;
        self.max_cpu_time_spent = self.max_cpu_time_spent.max(cpu_time_spent);
        self.cpu_time_spent += cpu_time_spent;
        self.time_spent = finish - self.start_time;
    }

    #[cfg(feature = "enable_render")]
    fn render(&mut self) {
        use crate::my_strategy::render::Object;

        self.render.clear();

        let mut robots: Vec<&Robot> = self.world.game.robots.iter().map(|v| v).collect();
        robots.sort_by_key(|v| v.id);

        let render = &mut self.render;

        render.add(Object::text(format!("current_tick: {}", self.world.game.current_tick)));

        self.world.game.ball.render(render);

        for robot in robots {
            robot.render(render);

            let order = self.active_order.iter()
                .find(|v| v.robot_id() == robot.id)
                .map(|v| v);

            if let Some(order) = order {
                order.render(robot, render);
            }
        }
    }
}
