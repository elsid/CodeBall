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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CollisionType {
    None,
    Touch,
    Kick,
}

#[derive(Clone, Debug)]
pub struct RobotExt {
    pub base: Robot,
    pub touch_normal: Option<Vec3>,
    pub radius_change_speed: f64,
    pub action: Action,
    pub mass: f64,
    pub arena_e: f64,
    pub is_me: bool,
    pub ball_collision_type: CollisionType,
}

impl RobotExt {
    pub fn from_robot(robot: &Robot, rules: &Rules) -> Self {
        RobotExt {
            base: robot.clone(),
            touch_normal: None,
            radius_change_speed: 0.0,
            action: Action::default(),
            mass: rules.ROBOT_MASS,
            arena_e: rules.ROBOT_ARENA_E,
            is_me: false,
            ball_collision_type: CollisionType::None,
        }
    }

    pub fn id(&self) -> i32 {
        self.base.id
    }

    pub fn base(&self) -> &Robot {
        &self.base
    }

    pub fn is_me(&self) -> bool {
        self.is_me
    }

    pub fn is_teammate(&self) -> bool {
        self.base.is_teammate
    }

    pub fn ball_collision_type(&self) -> CollisionType {
        self.ball_collision_type
    }

    pub fn jump(&mut self, jump_speed: f64, rules: &Rules) {
        self.base.jump(jump_speed, rules);
        self.radius_change_speed = jump_speed;
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

    pub fn from_ball(ball: &Ball, rules: &Rules) -> Self {
        BallExt {
            base: ball.clone(),
            mass: rules.BALL_MASS,
            arena_e: rules.BALL_ARENA_E,
        }
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
    ignore_me: bool,
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
                    ball_collision_type: CollisionType::None,
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
            score: Simulator::check_for_goal(&world.game.ball, &world.rules),
            me_index,
            ignore_me: false,
        }
    }

    pub fn robots(&self) -> &Vec<RobotExt> {
        &self.robots
    }

    pub fn robots_mut(&mut self) -> &mut Vec<RobotExt> {
        &mut self.robots
    }

    pub fn ball(&self) -> &BallExt {
        &self.ball
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }

    pub fn current_tick(&self) -> i32 {
        self.current_tick
    }

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

    pub fn set_ignore_me(&mut self, value: bool) {
        self.ignore_me = value;
    }

    pub fn tick(&mut self, time_interval: f64, micro_ticks_per_tick: usize, rng: &mut XorShiftRng) {
        let micro_tick_time_interval = time_interval / micro_ticks_per_tick as f64;
        for robot in self.robots.iter_mut() {
            robot.ball_collision_type = CollisionType::None;
        }
        for _ in 0..micro_ticks_per_tick {
            self.micro_tick(micro_tick_time_interval, rng);
        }
        self.current_tick += 1;
        self.current_time += time_interval;
    }

    fn micro_tick(&mut self, time_interval: f64, rng: &mut XorShiftRng) {
        rng.shuffle(&mut self.robots[..]);

        for robot in self.robots.iter_mut() {
            if robot.is_me && self.ignore_me {
                continue;
            }
            if let Some(touch_normal) = robot.touch_normal {
                if let Some(v) = Simulator::adjust_velocity(
                    time_interval,
                    robot.velocity(),
                    robot.action.target_velocity(),
                    touch_normal,
                    self.rules.ROBOT_MAX_GROUND_SPEED,
                    self.rules.ROBOT_ACCELERATION,
                ) {
                    robot.set_velocity(v);
                }
            }
            robot.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);
            robot.jump(robot.action.jump_speed, &self.rules);
        }

        self.ball.shift(time_interval, self.rules.GRAVITY, self.rules.MAX_ENTITY_SPEED);

        for i in 0 .. self.robots.len() - 1 {
            if self.robots[i].is_me && self.ignore_me {
                continue;
            }
            for j in i + 1 .. self.robots.len() {
                if self.robots[j].is_me && self.ignore_me {
                    continue;
                }
                let mut robot_i = self.robots[i].clone();
                let mut robot_j = self.robots[j].clone();
                let e = || { rng.gen_range(self.rules.MIN_HIT_E, self.rules.MAX_HIT_E) };
                Simulator::collide(e, &mut robot_i, &mut robot_j);
                self.robots[i] = robot_i;
                self.robots[j] = robot_j;
            }
        }

        let mut ball = self.ball.clone();

        for i in 0 .. self.robots.len() {
            if self.robots[i].is_me && self.ignore_me {
                continue;
            }
            let mut robot = self.robots[i].clone();
            let e = || { rng.gen_range(self.rules.MIN_HIT_E, self.rules.MAX_HIT_E) };
            let collision_type = Simulator::collide(e, &mut robot, &mut ball);
            let touch_normal = self.rules.arena.collide(&mut robot);
            robot.touch_normal = touch_normal;
            if robot.ball_collision_type == CollisionType::None {
                robot.ball_collision_type = collision_type;
            }
            self.robots[i] = robot;
        }

        self.rules.arena.collide(&mut ball);

        self.ball = ball;

        if self.score == 0 {
            self.score = Simulator::check_for_goal(self.ball.base(), &self.rules);
        }

        self.current_micro_tick += 1;
    }

    pub fn adjust_velocity(time_interval: f64, velocity: Vec3, mut target_velocity: Vec3,
                           touch_normal: Vec3, max_speed: f64, acceleration: f64) -> Option<Vec3> {
        target_velocity = target_velocity.clamp(max_speed);
        let new_velocity = target_velocity
            - touch_normal * touch_normal.dot(target_velocity);
        let velocity_change = new_velocity - velocity;
        let velocity_change_norm = velocity_change.norm();
        if velocity_change_norm > 0.0 {
            let acceleration = acceleration * touch_normal.y().max(0.0);
            Some(velocity + (velocity_change / velocity_change_norm * acceleration * time_interval)
                .clamp(velocity_change_norm))
        } else {
            None
        }
    }

    pub fn collide<F>(mut e: F, a: &mut Solid, b: &mut Solid) -> CollisionType
        where F: FnMut() -> f64 {

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
                - b.radius_change_speed() - a.radius_change_speed();
            if delta_velocity < 0.0 {
                let k = 1.0 + e();
                let impulse = normal * k * delta_velocity;
                let a_velocity = a.velocity() + impulse * k_a;
                let b_velocity = b.velocity() - impulse * k_b;
                a.set_velocity(a_velocity);
                b.set_velocity(b_velocity);
                return CollisionType::Kick;
            }
            return CollisionType::Touch;
        }
        CollisionType::None
    }

    pub fn check_for_goal(ball: &Ball, rules: &Rules) -> i32 {
        if ball.z > rules.arena.depth / 2.0 + ball.radius {
            1
        } else if ball.z < -(rules.arena.depth / 2.0 + ball.radius) {
            -1
        } else {
            0
        }
    }
}

pub fn integrate_movement(final_position: Vec3, final_speed: f64, max_speed: f64,
                          acceleration: f64, time_interval: f64, max_time: f64,
                          position: &mut Vec3, velocity: &mut Vec3) -> f64 {

    let mut iteration = 0;
    let mut prev_target_speed = max_speed;
    loop {
        let time_left = max_time - iteration as f64 * time_interval;
        if position.distance(final_position) < velocity.norm() * time_interval
            || time_left <= 0.0 {
            break;
        }
        let to_target = final_position - *position;
        let time_left_to_position = to_target.norm() / velocity.norm();
        let approximate_time_left = time_left.min(time_left_to_position) - time_interval;
        let target_speed = if velocity.norm() - final_speed < acceleration * approximate_time_left {
            max_speed.min(prev_target_speed)
        } else {
            final_speed
        };
        let target_velocity = to_target.normalized() * target_speed;
        if let Some(v) = Simulator::adjust_velocity(
            time_interval,
            *velocity,
            target_velocity,
            Vec3::j(),
            max_speed,
            acceleration
        ) {
            *velocity = v;
        }
        *position = *position + *velocity * time_interval;
        iteration += 1;
        prev_target_speed = target_speed;
    }
    iteration as f64 * time_interval
}
