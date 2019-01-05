use std::time::{Instant, Duration};
use crate::model::{Game, Action, Robot, Rules};
use crate::strategy::Strategy;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::world::World;
use crate::my_strategy::roles::Role;
use crate::my_strategy::targets::Target;
use crate::my_strategy::orders::Order;

#[cfg(feature = "enable_render")]
use crate::my_strategy::render::Render;

pub struct MyStrategyImpl {
    world: World,
    rng: XorShiftRng,
    start_time: Instant,
    tick_start_time: Instant,
    last_tick: i32,
    roles: Vec<RobotRole>,
    targets: Vec<RobotTarget>,
    orders: Vec<RobotOrder>,
    #[cfg(feature = "enable_render")]
    render: Render,
}

#[derive(Debug)]
pub struct RobotRole {
    pub robot_id: i32,
    pub score: i32,
    pub role: Role,
}

#[derive(Debug)]
pub struct RobotTarget {
    pub robot_id: i32,
    pub score: i32,
    pub target: Target,
}

pub struct RobotOrder {
    pub robot_id: i32,
    pub order: Order,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, _rules: &Rules, game: &Game, action: &mut Action) {
        self.tick_start_time = if game.current_tick == 0 {
            self.start_time
        } else {
            Instant::now()
        };
        if self.last_tick != game.current_tick {
            self.last_tick = game.current_tick;

            self.update_world(me, game);
            if self.world.is_reset_ticks() {
                self.roles.clear();
            } else {
                self.assign_roles();
                self.select_targets();
                self.give_orders();

                #[cfg(feature = "enable_render")]
                self.render();
            }

            #[cfg(feature = "enable_stats")]
            for v in self.orders.iter() {
                match &v.order {
                    Order::Idle(_) => (),
                    Order::Walk(walk) => println!("{}", serde_json::to_string(&walk.stats).unwrap()),
                    Order::Play(play) => println!("{}", serde_json::to_string(&play.stats).unwrap()),
                }
            }
            #[cfg(feature = "enable_render")]
            self.render();
        } else {
            self.update_world_me(me);
        }
        if !self.world.is_reset_ticks() {
            self.apply_action(action);
        }
        let finish = Instant::now();
        log!(game.current_tick, "[{}] time={:?}", me.id, finish - self.tick_start_time);
    }
}

impl MyStrategyImpl {
    pub fn new(me: &Robot, rules: &Rules, game: &Game, start_time: Instant) -> Self {
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
            start_time,
            tick_start_time: start_time,
            last_tick: -1,
            roles: Vec::new(),
            targets: Vec::new(),
            orders: Vec::new(),
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
            .map(|v| {
                v.role.get_score(self.world.get_robot(v.robot_id), &self.world)
            })
            .sum::<i32>();
        let new_roles = self.get_roles();
        let new_score = new_roles.iter()
            .map(|v| {
                v.role.get_score(self.world.get_robot(v.robot_id), &self.world)
            })
            .sum::<i32>();

        let is_same = self.roles.len() == new_roles.len()
            && self.roles.iter().zip(new_roles.iter())
                .all(|(l, r)| {
                    (l.robot_id, l.role) == (r.robot_id, r.role)
                });

        if !is_same {
            if self.roles.is_empty() || new_score > current_score + 150 {
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

    fn get_roles(&self) -> Vec<RobotRole> {
        let mut my_robots_ids: Vec<i32> = self.world.game.robots.iter()
            .filter(|v| v.is_teammate)
            .map(|v| v.id)
            .collect();

        my_robots_ids.sort();

        if my_robots_ids.is_empty() {
            Vec::new()
        } else if my_robots_ids.len() == 1 {
            vec![RobotRole { robot_id: my_robots_ids[0], score: 0, role: Role::forward() }]
        } else if my_robots_ids.len() == 2 {
            let first = self.world.game.robots.iter()
                .find(|v| v.id == my_robots_ids[0]).unwrap();
            let second = self.world.game.robots.iter()
                .find(|v| v.id == my_robots_ids[1]).unwrap();
            let first_forward = Role::forward().get_score(first, &self.world);
            let first_goalkeeper = Role::goalkeeper().get_score(first, &self.world);
            let second_forward = Role::forward().get_score(second, &self.world);
            let second_goalkeeper = Role::goalkeeper().get_score(second, &self.world);

            if first_forward + second_goalkeeper > first_goalkeeper + second_forward {
                vec![
                    RobotRole { robot_id: my_robots_ids[0], score: first_forward, role: Role::forward() },
                    RobotRole { robot_id: my_robots_ids[1], score: second_goalkeeper, role: Role::goalkeeper() },
                ]
            } else {
                vec![
                    RobotRole { robot_id: my_robots_ids[0], score: first_goalkeeper, role: Role::goalkeeper() },
                    RobotRole { robot_id: my_robots_ids[1], score: second_forward, role: Role::forward() },
                ]
            }
        } else {
            unimplemented!()
        }
    }

    fn select_targets(&mut self) {
        use std::collections::BTreeSet;

        let mut robots = self.world.game.robots.iter()
            .filter(|v| v.is_teammate)
            .map(|v| v.id)
            .collect::<BTreeSet<_>>();

        let mut targets = Target::generate(&self.world).into_iter()
            .map(|v| (v.get_priority(&self.world), v))
            .collect::<Vec<(i32, Target)>>();

        targets.sort_by_key(|(priority, _)| -priority);

        let mut new_targets = Vec::new();

        while !targets.is_empty() && !robots.is_empty() {
            let (_, target) = targets.pop().unwrap();
            robots.iter()
                .map(|id| {
                    let score = target.get_score(
                        self.get_role(*id).unwrap(),
                        self.world.get_robot(*id),
                        &self.world,
                    );
                    log!(
                        self.world.game.current_tick, "[{}] suggest {} score for target {:?}",
                        id, score, target
                    );
                    (score, *id)
                })
                .filter(|(score, _)| *score > 0)
                .max()
                .map(|(score, robot_id)| {
                    robots.remove(&robot_id);
                    new_targets.push(RobotTarget { robot_id, score, target });
                });
        }

        self.targets = new_targets;

        log!(self.world.game.current_tick, "selected targets {:?}", self.targets);
    }

    fn give_orders(&mut self) {
        self.orders = {
            let world = &self.world;
            let rng = &mut self.rng;
            let targets = &self.targets;
            let orders = &self.orders;

            world.game.robots.iter()
                .filter(|v| v.is_teammate)
                .map(|robot| {
                    let prev = orders.iter()
                        .find(|v| v.robot_id == robot.id)
                        .map(|v| &v.order);
                    RobotOrder {
                        robot_id: robot.id,
                        order: targets.iter()
                            .find(|v| v.robot_id == robot.id)
                            .map(|v| Order::new(prev, &v.target, robot, world, rng))
                            .unwrap(),
                    }
                })
                .collect()
        };
    }

    fn apply_action(&mut self, action: &mut Action) {
        self.orders.iter()
            .find(|v| v.robot_id == self.world.me.id)
            .map(|v| {
                *action = v.order.action();
                log!(
                    self.world.game.current_tick, "[{}] <{}> apply order {} {:?}",
                    self.world.me.id, v.order.id(), v.order.name(), action
                );
            });
    }

    fn get_role(&self, id: i32) -> Option<&Role> {
        self.roles.iter()
            .find(|v| v.robot_id == id)
            .map(|v| &v.role)
    }

    fn get_target(&self, id: i32) -> Option<&Target> {
        self.targets.iter()
            .find(|v| v.robot_id == id)
            .map(|v| &v.target)
    }

    fn get_order(&self, id: i32) -> Option<&Order> {
        self.orders.iter()
            .find(|v| v.robot_id == id)
            .map(|v| &v.order)
    }

    #[cfg(feature = "enable_render")]
    fn render(&mut self) {
        self.render.clear();

        let mut robots: Vec<&Robot> = self.world.game.robots.iter().map(|v| v).collect();
        robots.sort_by_key(|v| v.id);

        let render = &mut self.render;

        for robot in robots {
            robot.render(render);

            let role = self.roles.iter()
                .find(|v| v.robot_id == robot.id)
                .map(|v| &v.role);

            if let Some(role) = role {
                role.render(&robot, render);
            }

            let target = self.targets.iter()
                .find(|v| v.robot_id == robot.id)
                .map(|v| &v.target);

            if let Some(target) = target {
                target.render(&robot, &self.world, render);
            }

            let order = self.orders.iter()
                .find(|v| v.robot_id == robot.id)
                .map(|v| &v.order);

            if let Some(order) = order {
                order.render(&robot, render);
            }
        }

        self.world.game.ball.render(render);
    }

    fn real_time_spent(&self) -> Duration {
        Instant::now() - self.start_time
    }
}
