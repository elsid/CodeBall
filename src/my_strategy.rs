use model::*;
use strategy::Strategy;

#[path = "common.rs"]
mod common;

#[path = "vec3.rs"]
mod vec3;

use self::vec3::Vec3;

const BALL_RADIUS: f64 = 2.0;
const ROBOT_MAX_GROUND_SPEED: f64 = 30.0;
const ROBOT_MAX_JUMP_SPEED: f64 = 15.0;
const ROBOT_MAX_RADIUS: f64 = 1.05;
const ROBOT_MIN_RADIUS: f64 = 1.0;

pub struct MyStrategy;

trait HasPosition {
    fn position(&self) -> Vec3;
}

trait HasVelocity {
    fn velocity(&self) -> Vec3;
}

trait HasMutVelocity {
    fn set_velocity(&mut self, value: &Vec3);
}

impl HasPosition for Ball {
    fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl HasPosition for Robot {
    fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl HasVelocity for Ball {
    fn velocity(&self) -> Vec3 {
        Vec3::new(self.velocity_x, self.velocity_y, self.velocity_z)
    }
}

impl HasVelocity for Robot {
    fn velocity(&self) -> Vec3 {
        Vec3::new(self.velocity_x, self.velocity_y, self.velocity_z)
    }
}

impl HasVelocity for Action {
    fn velocity(&self) -> Vec3 {
        Vec3::new(self.target_velocity_x, self.target_velocity_y, self.target_velocity_z)
    }
}

impl HasMutVelocity for Action {
    fn set_velocity(&mut self, value: &Vec3) {
        self.target_velocity_x = value.x();
        self.target_velocity_y = value.y();
        self.target_velocity_z = value.z();
    }
}

impl Default for MyStrategy {
    fn default() -> Self {
        Self {}
    }
}

fn get_goal_target(arena: &Arena) -> Vec3 {
    Vec3::new(0.0, arena.goal_height / 2.0, arena.depth / 2.0 + arena.goal_depth / 2.0)
}

fn get_defend_target(arena: &Arena) -> Vec3 {
    Vec3::new(0.0, arena.goal_height / 2.0, -arena.depth / 2.0 + arena.goal_depth / 2.0)
}

fn get_position_to_jump(me: &Robot, rules: &Rules, game: &Game) -> Vec3 {
    let goal_target = get_goal_target(&rules.arena);
    let to_goal = goal_target - game.ball.position();
    let to_goal_direction = to_goal.normalized();
    let desired_ball_velocity = to_goal_direction * ROBOT_MAX_JUMP_SPEED;
    let desired_robot_hit_direction = (desired_ball_velocity - game.ball.velocity()).normalized();
    (game.ball.position() - desired_robot_hit_direction * (game.ball.radius + ROBOT_MIN_RADIUS + 1e-3))
        .with_min_y(ROBOT_MIN_RADIUS)
}

fn is_inside(arena: &Arena, position: &Vec3) -> bool {
    -arena.width / 2.0 + BALL_RADIUS < position.x() && position.x() < arena.width / 2.0 - BALL_RADIUS
    && position.y() < arena.height - BALL_RADIUS
    && -arena.depth / 2.0 + BALL_RADIUS < position.z() && position.z() < arena.depth / 2.0 - BALL_RADIUS
}

impl Strategy for MyStrategy {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        let robot_to_act = game.robots.iter()
            .filter(|&v| v.is_teammate)
            .map(|v| (v.id, get_position_to_jump(v, rules, game)))
            .filter(|(_, v)| is_inside(&rules.arena, v))
            .find(|(id, _)| *id == me.id);
        if let Some(v) = robot_to_act {
            if me.position().distance(v.1) < 0.1 {
                action.jump_speed = ROBOT_MAX_JUMP_SPEED;
            } else {
                let target_velocity = (v.1 - me.position()).normalized();
                if target_velocity.y() > target_velocity.x() && target_velocity.y() > target_velocity.z() {
                    action.jump_speed = target_velocity.y() * ROBOT_MAX_JUMP_SPEED;
                    action.set_velocity(&(target_velocity * ROBOT_MAX_GROUND_SPEED));
                } else {
                    action.set_velocity(&(target_velocity.with_y(0.0).normalized() * ROBOT_MAX_GROUND_SPEED));
                }
            }
            return;
        }
        action.set_velocity(&((get_defend_target(&rules.arena) - me.position()).with_y(0.0).normalized() * ROBOT_MAX_GROUND_SPEED));
    }
}
