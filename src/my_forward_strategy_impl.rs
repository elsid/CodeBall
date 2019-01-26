use std::time::{Instant, Duration};
use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::world::World;
use crate::my_strategy::orders::Order;
use crate::my_strategy::common::IdGenerator;
use crate::my_strategy::roles::Role;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

const ROBOT_PRIORITY_CHANGE_GAP: i32 = 0;
const ROBOT_ROLE_CHANGE_GAP: i32 = 0;

pub struct MyStrategyImpl {
    world: World,
    rng: XorShiftRng,
    start_time: Instant,
    tick_start_time: Instant,
    time_spent: Duration,
    cpu_time_spent: Duration,
    max_cpu_time_spent: Duration,
    last_tick: i32,
    orders: Vec<Order>,
    robots_priority: Vec<i32>,
    roles: Vec<Role>,
    order_id_generator: IdGenerator,
    micro_ticks: usize,
    last_reset_tick: i32,
    #[cfg(feature = "enable_time")]
    micro_ticks_before: usize,
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
        self.on_start();
        if self.last_tick != game.current_tick {
            self.last_tick = game.current_tick;
            self.update_world(me, game);
            if self.world.is_reset_ticks() {
                self.last_reset_tick = game.current_tick;
                self.roles.clear();
                self.robots_priority.clear();
                self.orders.clear();
            } else if game.current_tick - self.last_reset_tick > 20 {
                self.assign_roles();
                self.set_priority();
                self.give_orders();
            }
            #[cfg(feature = "enable_stats")]
            for v in self.orders.iter() {
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
            orders: Vec::new(),
            robots_priority: Vec::new(),
            roles: Vec::new(),
            order_id_generator: IdGenerator::new(),
            micro_ticks: 0,
            last_reset_tick: 0,
            #[cfg(feature = "enable_time")]
            micro_ticks_before: 0,
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

    fn assign_roles(&mut self) {
        let current_score = self.roles.iter()
            .map(|v| v.get_score(&self.world))
            .sum::<i32>();
        let new_roles = self.get_roles();
        let new_score = new_roles.iter()
            .map(|v| {
                if v.can_quit(&self.world) {
                    v.get_score(&self.world)
                } else {
                    0
                }
            })
            .sum::<i32>();

        let is_same = self.roles.len() == new_roles.len()
            && self.roles.iter().zip(new_roles.iter())
                .all(|(l, r)| l == r);

        if !is_same {
            if self.roles.is_empty() || new_score > current_score + ROBOT_ROLE_CHANGE_GAP {
                log!(
                    self.world.game.current_tick, "assign roles {:?} with total score {} ({})",
                    new_roles, new_score, new_score - current_score
                );
                self.roles = new_roles;
            } else {
                log!(
                    self.world.game.current_tick, "reject roles {:?} with total score {} ({})",
                    new_roles, new_score, new_score - current_score
                );
            }
        } else {
            log!(self.world.game.current_tick, "use roles {:?}", self.roles);
        }
    }

    fn get_roles(&self) -> Vec<Role> {
        self.world.game.robots.iter()
            .filter(|v| v.is_teammate)
            .map(|v| Role::forward(v.id))
            .collect()
    }

    fn set_priority(&mut self) {
        use crate::my_strategy::common::as_score;

        if self.robots_priority.is_empty() {
            let mut robots_with_distance_to_ball = self.world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .map(|v| {
                    let distance = self.world.game.ball.position().distance(v.position());
                    (as_score(distance), v.id)
                })
                .collect::<Vec<_>>();

            robots_with_distance_to_ball.sort();

            self.robots_priority = robots_with_distance_to_ball.iter()
                .map(|(_, id)| *id)
                .collect();
        } else {
            self.robots_priority = self.orders.iter()
                .map(|v| v.robot_id())
                .collect();
        }
    }

    fn give_orders(&mut self) {
        use crate::my_strategy::orders::OrderContext;
        use crate::my_strategy::common::as_score;

        let world = &self.world;
        let mut ctx = OrderContext {
            rng: &mut self.rng,
            order_id_generator: &mut self.order_id_generator,
            micro_ticks: &mut self.micro_ticks,
        };
        let has_orders = !self.orders.is_empty();
        let opposite_world = world.opposite();
        let roles = &self.roles;

        let opponents_orders = world.game.robots.iter()
            .filter(|v| {
                !v.is_teammate
            })
            .min_by_key(|v| {
                as_score(v.position().distance(world.game.ball.position()))
            })
            .map(|robot| {
                Order::try_play(&robot.opposite(), &opposite_world, &Vec::new(), std::f64::MAX, &mut ctx).opposite()
            })
            .into_iter()
            .collect::<Vec<_>>();

        let mut orders = self.robots_priority.iter()
            .enumerate()
            .map(|(n, robot_id)| {
                let robot = world.get_robot(*robot_id);
                let max_z = 10.0;
                let order = Order::try_play(robot, world, &opponents_orders[..], max_z, &mut ctx);
                (
                    n,
                    if order.is_idle() {
                        if robot.nitro_amount < world.rules.START_NITRO_AMOUNT
                            && world.game.ball.position().distance(world.rules.get_goalkeeper_position())
                                > world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS {

                            match Order::try_take_nitro_pack(robot, world, max_z, ctx.order_id_generator) {
                                Order::Idle(_) => Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator),
                                v => v,
                            }
                        } else {
                            Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator)
                        }
                    } else {
                        order
                    }
                )
            })
            .collect::<Vec<_>>();

        orders.sort_by_key(|(n, order)| {
            if has_orders {
                -(order.score() - *n as i32 * ROBOT_PRIORITY_CHANGE_GAP)
            } else {
                -order.score()
            }
        });

        let opponents_orders_num = opponents_orders.len();
        let mut all_orders = opponents_orders;

        for (_, order) in orders.into_iter() {
            all_orders.push(order);
        }

        if all_orders.len() > opponents_orders_num + 1 {
            for i in opponents_orders_num + 1..all_orders.len() {
                let robot_id = all_orders[i].robot_id();
                let robot = world.get_robot(robot_id);
                let role = roles.iter()
                    .find(|v| v.robot_id() == robot_id)
                    .unwrap();
                let max_z = 10.0;
                all_orders[i] = {
                    let order = Order::try_play(robot, world, &all_orders[0..i], max_z, &mut ctx);
                    if order.is_idle() {
                        match role {
                            Role::Forward(_) => Order::try_take_nitro_pack(robot, world, max_z, ctx.order_id_generator),
                            Role::Goalkeeper(_) => {
                                if robot.nitro_amount < world.rules.START_NITRO_AMOUNT
                                    && world.game.ball.position().distance(world.rules.get_goalkeeper_position())
                                        > world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS {

                                    match Order::try_take_nitro_pack(robot, world, max_z, ctx.order_id_generator) {
                                        Order::Idle(_) => Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator),
                                        v => v,
                                    }
                                } else {
                                    Order::walk_to_goalkeeper_position(robot, world, ctx.order_id_generator)
                                }
                            },
                        }
                    } else {
                        order
                    }
                }
            }
        }

        self.orders = all_orders.into_iter().skip(opponents_orders_num).collect();
    }

    fn apply_action(&mut self, action: &mut Action) {
        self.orders.iter()
            .find(|v| v.robot_id() == self.world.me.id)
            .map(|v| {
                *action = v.action().clone();
                log!(self.world.game.current_tick, "[{}] <{}> apply order {:?}", self.world.me.id, v.id(), action);
            });
    }

    fn on_start(&mut self) {
        self.tick_start_time = Instant::now();
        #[cfg(feature = "enable_time")]
        {
            self.micro_ticks_before = self.micro_ticks;
        }
    }

    fn on_finish(&mut self) {
        #[cfg(feature = "enable_time")]
        use crate::my_strategy::common::milliseconds;

        let finish = Instant::now();
        let cpu_time_spent = finish - self.tick_start_time;
        self.max_cpu_time_spent = self.max_cpu_time_spent.max(cpu_time_spent);
        self.cpu_time_spent += cpu_time_spent;
        self.time_spent = finish - self.start_time;

        #[cfg(feature = "enable_time")]
        {
            let micro_ticks = self.micro_ticks - self.micro_ticks_before;
            println!("{} {}", milliseconds(&cpu_time_spent), micro_ticks);
        }
    }

    #[cfg(feature = "enable_render")]
    fn render(&mut self) {
        use crate::my_strategy::render::Object;
        use crate::my_strategy::vec3::Vec3;
        use crate::my_strategy::roles::Goalkeeper;

        self.render.clear();

        self.render.add(Object::text(format!("priority: {:?}", self.robots_priority)));
        self.render.add(Object::text(format!(
            "orders: {:?}",
            self.orders.iter().map(|v| (v.robot_id(), v.id(), v.score())).collect::<Vec<_>>()
        )));

        let mut robots: Vec<&Robot> = self.world.game.robots.iter().map(|v| v).collect();
        robots.sort_by_key(|v| v.id);

        let render = &mut self.render;

        render.add(Object::text(format!("current_tick: {}", self.world.game.current_tick)));

        self.world.game.ball.render(render);

        for robot in robots {
            robot.render(render);

            let role = self.roles.iter()
                .find(|v| v.robot_id() == robot.id)
                .map(|v| v);

            if let Some(role) = role {
                role.render(robot, render);
            }

            let order = self.orders.iter()
                .find(|v| v.robot_id() == robot.id)
                .map(|v| v);

            if let Some(order) = order {
                order.render(robot, render);
            }
        }

        render.add(Object::line(
            Vec3::new(
                -self.world.rules.arena.width / 2.0,
                self.world.rules.ROBOT_RADIUS,
                Goalkeeper::max_z(&self.world)
            ),
            Vec3::new(
                self.world.rules.arena.width / 2.0,
                self.world.rules.ROBOT_RADIUS,
                Goalkeeper::max_z(&self.world)
            ),
            3.0,
            Goalkeeper::get_color()
        ));
    }
}