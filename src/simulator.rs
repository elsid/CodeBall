use crate::model::{Action, Ball, Robot, Rules};
use crate::my_strategy::common::Square;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::random::{Rng, XorShiftRng};
use crate::my_strategy::world::World;
use crate::my_strategy::entity::Entity;

trait Shiftable : Entity {
    fn shift(&mut self, time_interval: f64, gravity: f64, max_entity_speed: f64) {
        let clamped_velocity = self.velocity().clamp(max_entity_speed);
        let next_position = self.position() + clamped_velocity * time_interval
            - Vec3::default().with_y(gravity * time_interval.square() / 2.0);
        let next_velocity = self.velocity()
            - Vec3::default().with_y(gravity * time_interval);
        self.set_position(next_position);
        self.set_velocity(next_velocity);
    }
}

pub trait Solid : Entity {
    fn radius(&self) -> f64;
    fn mass(&self) -> f64;
    fn radius_change_speed(&self) -> f64;
    fn arena_e(&self) -> f64;
}

#[derive(Clone, Debug)]
pub struct RobotExt {
    pub base: Robot,
    pub touch_normal: Option<Vec3>,
    pub radius_change_speed: f64,
    pub action: Action,
    pub mass: f64,
    pub arena_e: f64,
    is_me: bool,
}

impl RobotExt {
    pub fn id(&self) -> i32 {
        self.base.id
    }

    pub fn base(&self) -> &Robot {
        &self.base
    }

    pub fn is_me(&self) -> bool {
        self.is_me
    }

    fn set_radius(&mut self, value: f64) {
        self.base.radius = value;
    }
}

impl Entity for RobotExt {
    fn position(&self) -> Vec3 {
        self.base.position()
    }

    fn velocity(&self) -> Vec3 {
        self.base.velocity()
    }

    fn set_position(&mut self, value: Vec3) {
        self.base.set_position(value)
    }

    fn set_velocity(&mut self, value: Vec3) {
        self.base.set_velocity(value)
    }
}

#[derive(Clone)]
pub struct BallExt {
    base: Ball,
    mass: f64,
    arena_e: f64,
}

impl BallExt {
    pub fn new(base: Ball, mass: f64, arena_e: f64) -> Self {
        BallExt {base, mass, arena_e}
    }

    pub fn base(&self) -> &Ball {
        &self.base
    }
}

impl Entity for BallExt {
    fn position(&self) -> Vec3 {
        self.base.position()
    }

    fn velocity(&self) -> Vec3 {
        self.base.velocity()
    }

    fn set_position(&mut self, value: Vec3) {
        self.base.set_position(value)
    }

    fn set_velocity(&mut self, value: Vec3) {
        self.base.set_velocity(value)
    }
}

impl Shiftable for BallExt {}

impl Shiftable for RobotExt {}

impl Solid for BallExt {
    fn radius(&self) -> f64 {
        self.base.radius
    }

    fn mass(&self) -> f64 {
        self.mass
    }

    fn radius_change_speed(&self) -> f64 {
        0.0
    }

    fn arena_e(&self) -> f64 {
        self.arena_e
    }
}

impl Solid for RobotExt {
    fn radius(&self) -> f64 {
        self.base.radius
    }

    fn mass(&self) -> f64 {
        self.mass
    }

    fn radius_change_speed(&self) -> f64 {
        self.radius_change_speed
    }

    fn arena_e(&self) -> f64 {
        self.arena_e
    }
}

#[derive(Clone)]
pub struct Simulator {
    robots: Vec<RobotExt>,
    ball: BallExt,
    rules: Rules,
    current_tick: i32,
    current_micro_tick: i32,
    current_time: f64,
    score: i32,
    me_index: usize,
}

impl Simulator {
    pub fn new(world: &World, me_id: i32) -> Self {
        let robots: Vec<RobotExt> = world.game.robots.iter()
            .map(|v| {
                let touch_normal = if v.touch {
                    Some(Vec3::new(
                        v.touch_normal_x.unwrap(),
                        v.touch_normal_y.unwrap(),
                        v.touch_normal_z.unwrap()
                    ))
                } else {
                    None
                };
                RobotExt {
                    base: v.clone(),
                    touch_normal,
                    radius_change_speed: 0.0,
                    action: Action::default(),
                    mass: world.rules.ROBOT_MASS,
                    arena_e: world.rules.ROBOT_ARENA_E,
                    is_me: v.id == me_id,
                }
            })
            .collect();
        let me_index = robots.iter()
            .position(|v| v.id() == me_id)
            .unwrap();

        Simulator {
            robots,
            ball: BallExt {
                base: world.game.ball.clone(),
                mass: world.rules.BALL_MASS,
                arena_e: world.rules.BALL_ARENA_E,
            },
            rules: world.rules.clone(),
            current_tick: 0,
            current_micro_tick: 0,
            current_time: 0.0,
            score: 0,
            me_index,
        }
    }

    pub fn robots(&self) -> &Vec<RobotExt> {
        &self.robots
    }

//    pub fn robots_mut(&mut self) -> &mut Vec<RobotExt> {
//        &mut self.robots
//    }

    pub fn ball(&self) -> &BallExt {
        &self.ball
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

//    pub fn current_tick(&self) -> i32 {
//        self.current_tick
//    }

    pub fn current_micro_tick(&self) -> i32 {
        self.current_micro_tick
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn me(&self) -> &RobotExt {
        self.robots.iter()
            .find(|v| v.is_me)
            .unwrap()
    }

    pub fn me_mut(&mut self) -> &mut RobotExt {
        self.robots.iter_mut()
            .find(|v| v.is_me)
            .unwrap()
    }

    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize, rng: &mut XorShiftRng) {
        let micro_tick_time_interval = time_interval / micro_ticks_per_tick as f64;
        for _ in 0..micro_ticks_per_tick {
            self.micro_tick(micro_tick_time_interval, rng);
        }
        self.current_tick += 1;
        self.current_time += time_interval;
    }

    fn micro_tick(&mut self, time_interval: f64, rng: &mut XorShiftRng) {
        rng.shuffle(&mut self.robots[..]);

        for robot in self.robots.iter_mut() {
            if let Some(touch_normal) = robot.touch_normal {
                let target_velocity = robot.action.target_velocity()
                    .clamp(self.rules.ROBOT_MAX_GROUND_SPEED);
                let velocity = target_velocity
                    - touch_normal * touch_normal.dot(target_velocity);
                let velocity_change = velocity - robot.velocity();
                let velocity_change_norm = velocity_change.norm();
                if velocity_change_norm > 0.0 {
                    let acceleration = self.rules.ROBOT_ACCELERATION
                        * touch_normal.y().max(0.0);
                    let robot_velocity = robot.velocity()
                        + (velocity_change.normalized() * acceleration * time_interval)
                        .clamp(velocity_change_norm);
                    robot.set_velocity(robot_velocity);
                }
            }
            robot.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);
            let robot_radius = self.rules.ROBOT_MIN_RADIUS
                + (self.rules.ROBOT_MAX_RADIUS - self.rules.ROBOT_MIN_RADIUS)
                * robot.action.jump_speed / self.rules.ROBOT_MAX_JUMP_SPEED;
            robot.set_radius(robot_radius);
            robot.radius_change_speed = robot.action.jump_speed;
        }

        self.ball.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);

        for i in 0 .. self.robots.len() - 1 {
            for j in i + 1 .. self.robots.len() {
                let mut robot_i = self.robots[i].clone();
                let mut robot_j = self.robots[j].clone();
                self.collide(&mut robot_i, &mut robot_j, rng);
                self.robots[i] = robot_i;
                self.robots[j] = robot_j;
            }
        }

        let mut ball = self.ball.clone();

        for i in 0 .. self.robots.len() {
            let mut robot = self.robots[i].clone();
            self.collide(&mut robot, &mut ball, rng);
            let touch_normal = self.rules.arena.collide(&mut robot);
            robot.touch_normal = touch_normal;
            self.robots[i] = robot;
        }

        self.rules.arena.collide(&mut ball);

        self.ball = ball;

        if self.ball.position().z() > self.rules.arena.depth / 2.0 + self.ball.radius() {
            self.score = 1;
        } else if self.ball.position().z() < -(self.rules.arena.depth / 2.0 + self.ball.radius()) {
            self.score = -1;
        }

        self.current_micro_tick += 1;
    }

    pub fn collide(&self, a: &mut Solid, b: &mut Solid, rng: &mut XorShiftRng) {
        let delta_position = b.position() - a.position();
        let distance = delta_position.norm();
        let penetration = a.radius() + b.radius() - distance;
        if penetration > 0.0 {
            let k_a = (1.0 / a.mass()) / ((1.0 / a.mass()) + (1.0 / b.mass()));
            let k_b = (1.0 / b.mass()) / ((1.0 / a.mass()) + (1.0 / b.mass()));
            let normal = delta_position.normalized();
            let a_position = a.position() - normal * penetration * k_a;
            let b_position = b.position() + normal * penetration * k_b;
            a.set_position(a_position);
            b.set_position(b_position);
            let delta_velocity = normal.dot(b.velocity() - a.velocity())
                + b.radius_change_speed() - a.radius_change_speed();
            if delta_velocity < 0.0 {
                let k = 1.0 + rng.gen_range(self.rules.MIN_HIT_E, self.rules.MAX_HIT_E);
                let impulse = normal * k * delta_velocity;
                let a_velocity = a.velocity() + impulse * k_a;
                let b_velocity = b.velocity() - impulse * k_b;
                a.set_velocity(a_velocity);
                b.set_velocity(b_velocity);
            }
        }
    }
}
