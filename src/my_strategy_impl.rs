use model::{Game, Action, Robot, Rules, Ball, Arena};
use strategy::Strategy;
use my_strategy::vec3::Vec3;
use my_strategy::world::{World};

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

trait Contains {
    fn contains(&self, position: &Vec3) -> bool;
}

impl Contains for Arena {
    fn contains(&self, position: &Vec3) -> bool {
        -self.width / 2.0 + BALL_RADIUS < position.x() && position.x() < self.width / 2.0 - BALL_RADIUS
        && position.y() < self.height - BALL_RADIUS
        && -self.depth / 2.0 + BALL_RADIUS < position.z() && position.z() < self.depth / 2.0 - BALL_RADIUS
    }
}

fn get_goal_target(arena: &Arena) -> Vec3 {
    Vec3::new(0.0, arena.goal_height / 2.0, arena.depth / 2.0 + arena.goal_depth / 2.0)
}

fn get_defend_target(arena: &Arena) -> Vec3 {
    Vec3::new(0.0, arena.goal_height / 2.0, -arena.depth / 2.0 + arena.goal_depth / 2.0)
}

pub struct MyStrategyImpl {
    game: Game,
    world: World,
}

impl Default for MyStrategyImpl {
    fn default() -> Self {
        unimplemented!()
    }
}

impl Strategy for MyStrategyImpl {
    fn act(&mut self, me: &Robot, rules: &Rules, game: &Game, action: &mut Action) {
        self.update_actual_game(me, game);
        self.apply_action(action);
    }
}

impl MyStrategyImpl {
    pub fn new(me: &Robot, rules: &Rules, game: &Game) -> Self {
        MyStrategyImpl {
            game: game.clone(),
            world: World::new(me.clone(), rules.clone(), game.clone()),
        }
    }

    fn update_actual_game(&mut self, me: &Robot, game: &Game) {
        self.world.update(me, game);
    }

    fn apply_action(&mut self, action: &mut Action) {
        let robot_to_act = self.world.game.robots.iter()
            .filter(|&v| v.is_teammate)
            .map(|v| (v.id, self.get_position_to_jump(v)))
            .filter(|(_, v)| self.world.rules.arena.contains(v))
            .find(|(id, _)| *id == self.world.me.id);
        if let Some(v) = robot_to_act {
            if self.world.me.position().distance(v.1) < 0.1 {
                action.jump_speed = ROBOT_MAX_JUMP_SPEED;
            } else {
                let target_velocity = (v.1 - self.world.me.position()).normalized();
                if target_velocity.y() > target_velocity.x() && target_velocity.y() > target_velocity.z() {
                    action.jump_speed = target_velocity.y() * ROBOT_MAX_JUMP_SPEED;
                    action.set_velocity(&(target_velocity * ROBOT_MAX_GROUND_SPEED));
                } else {
                    action.set_velocity(&(target_velocity.with_y(0.0).normalized() * ROBOT_MAX_GROUND_SPEED));
                }
            }
            return;
        }
        action.set_velocity(&((get_defend_target(&self.world.rules.arena) - self.world.me.position()).with_y(0.0).normalized() * ROBOT_MAX_GROUND_SPEED));
    }

    fn get_position_to_jump(&self, robot: &Robot) -> Vec3 {
        let goal_target = get_goal_target(&self.world.rules.arena);
        let to_goal = goal_target - self.world.game.ball.position();
        let to_goal_direction = to_goal.normalized();
        let desired_ball_velocity = to_goal_direction * ROBOT_MAX_JUMP_SPEED;
        let desired_robot_hit_direction = (desired_ball_velocity - self.world.game.ball.velocity()).normalized();
        (self.world.game.ball.position() - desired_robot_hit_direction * (self.world.game.ball.radius + ROBOT_MIN_RADIUS + 1e-3))
            .with_min_y(ROBOT_MIN_RADIUS)
    }
}
