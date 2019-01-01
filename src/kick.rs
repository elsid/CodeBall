use crate::model::{Robot, Ball, Rules};
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::physics::MoveEquation;

#[derive(Debug, Clone, PartialEq)]
pub struct Kick {
    pub angle: f64,
    pub final_speed: f64,
    pub jump_speed: f64,
    pub kick_ball_target: KickBallTarget,
    pub ball_movement: BallMovement,
}

pub fn get_optimal_kick(target: Vec3, distance_to_ball: f64, time: f64, ball: &Ball, robot: &Robot,
                        rules: &Rules, max_time: f64, max_iter: usize) -> Kick {
    use ndarray::prelude::{Array, ArrayView1};
    use optimize::{NelderMeadBuilder, Minimizer};
    use crate::my_strategy::common::{Square, Clamp};
    use crate::my_strategy::optimization::optimize1d;

    let optimizer = NelderMeadBuilder::default()
        .maxiter(max_iter)
        .build()
        .unwrap();
    let initial_angle = (ball.position() - target).with_y(0.0).normalized().absolute_rotation_by_y();
    let initial_max_speed = rules.ROBOT_MAX_GROUND_SPEED;
    let initial_jump_speed = rules.ROBOT_MAX_JUMP_SPEED;
    let args = Array::from_vec(vec![
        initial_angle,
        initial_max_speed,
        initial_jump_speed,
    ]);
    let get_kick_ball_position = |angle: f64| {
        get_kick_ball_position(angle, distance_to_ball, ball.position(), rules)
    };
    let get_kick_ball_target = |max_speed: f64, kick_ball_position: Vec3| {
        get_kick_ball_target(
            robot.position(),
            robot.velocity(),
            max_speed,
            kick_ball_position,
            rules,
            time,
        )
    };
    let get_ball_movement = |jump_speed: f64, kick_ball_target: &KickBallTarget| {
        get_ball_movement(target, jump_speed, &kick_ball_target, robot, ball, rules, max_time + 2.0)
    };
    let get_penalty = |angle: f64, final_speed: f64, jump_speed: f64| {
        let clamped_angle = angle.clamp(
            initial_angle - std::f64::consts::PI,
            initial_angle + std::f64::consts::PI,
        );
        let clamped_final_speed = final_speed.clamp(0.0, rules.ROBOT_MAX_GROUND_SPEED);
        let clamped_jump_speed = jump_speed.clamp(0.0, rules.ROBOT_MAX_JUMP_SPEED);
        let kick_ball_position = get_kick_ball_position(clamped_angle);
        let kick_ball_target = get_kick_ball_target(clamped_final_speed, kick_ball_position);
        let time_distance = (kick_ball_target.time - time).abs();
        let distance_to_kick_ball_position = kick_ball_target.position.distance(kick_ball_position);
        let angle_overflow = (angle - clamped_angle).square();
        let final_speed_overflow = (final_speed - clamped_final_speed).square();
        let jump_overflow = (jump_speed - clamped_jump_speed).square();
        let ball_target = get_ball_movement(clamped_jump_speed, &kick_ball_target);
        let end = ball_target.move_equation.get_position(ball_target.time_to_target);
        let cos = (end - ball.position()).cos(target - ball.position());
        let distance = end.distance(target);
        let ceiling_penetration = (ball_target.move_equation.get_max_y() + rules.BALL_RADIUS + 0.1 - rules.arena.height).min(0.0).square();
        let goal_ceiling_penetration = if end.z() > rules.arena.depth / 2.0 + rules.arena.goal_side_radius + rules.BALL_RADIUS {
            let at_depth = optimize1d(0.0, ball_target.time_to_target, 10,
            |time| {
                (ball_target.move_equation.get_position(time).z() - rules.arena.depth / 2.0 - rules.arena.goal_side_radius - rules.BALL_RADIUS).abs()
            });
            (ball_target.move_equation.get_position(at_depth).y()
                + rules.BALL_RADIUS + 0.1 - rules.arena.goal_height).min(0.0).square()
        } else {
            0.0
        };
        angle_overflow + jump_overflow + final_speed_overflow
            + time_distance + distance_to_kick_ball_position
            + ceiling_penetration + goal_ceiling_penetration + 1000.0 * distance / cos.max(1e-3)
    };
    let result = optimizer.minimize(
        |x: ArrayView1<f64>| get_penalty(x[0], x[1], x[2]),
        args.view()
    );
    let angle = result[0].clamp(
        initial_angle - std::f64::consts::PI,
        initial_angle + std::f64::consts::PI,
    );
    let final_speed = result[1].clamp(0.0, rules.ROBOT_MAX_GROUND_SPEED);
    let jump_speed = result[2].clamp(0.0, rules.ROBOT_MAX_JUMP_SPEED);
    let kick_ball_position = get_kick_ball_position(angle);
    let kick_ball_target = get_kick_ball_target(final_speed, kick_ball_position);
    let ball_movement = get_ball_movement(jump_speed, &kick_ball_target);
    Kick {
        angle,
        final_speed,
        jump_speed,
        kick_ball_target,
        ball_movement,
    }
}

pub fn get_kick_ball_position(angle: f64, distance: f64, position: Vec3, rules: &Rules) -> Vec3 {
    position.with_y(rules.ROBOT_MIN_RADIUS) + Vec3::i().rotated_by_y(angle) * distance
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallMovement {
    pub move_equation: MoveEquation,
    pub time_to_target: f64,
}

pub fn get_ball_movement(target: Vec3, jump_speed: f64, kick_ball_target: &KickBallTarget,
                         robot: &Robot, ball: &Ball, rules: &Rules, max_time: f64) -> BallMovement {
    use crate::my_strategy::entity::Entity;
    use crate::my_strategy::optimization::optimize1d;
    use crate::my_strategy::simulator::{Simulator, RobotExt, BallExt};

    let mut robot_ext = RobotExt::from_robot(robot, rules);
    robot_ext.set_position(kick_ball_target.position);
    robot_ext.set_velocity(kick_ball_target.velocity);
    robot_ext.jump(jump_speed, &rules);
    let mut ball_ext = BallExt::from_ball(ball, rules);
    let e = rules.mean_e();
    Simulator::collide(|| e, &mut robot_ext, &mut ball_ext);
    let move_equation = MoveEquation {
        initial_position: ball_ext.position(),
        initial_velocity: ball_ext.velocity(),
        acceleration: Vec3::only_y(-rules.GRAVITY),
    };
    let get_distance_to_target = |time: f64| {
        let position = move_equation.get_position(time);
        position.with_max_y(ball.radius).distance(target)
            + ball.radius - position.y().min(ball.radius)
    };
    let time = optimize1d(0.0, max_time, 10, get_distance_to_target);
    BallMovement {move_equation, time_to_target: time }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KickBallTarget {
    pub time: f64,
    pub position: Vec3,
    pub velocity: Vec3,
}

pub fn get_kick_ball_target(mut position: Vec3, mut velocity: Vec3, final_robot_speed: f64,
                            kick_ball_position: Vec3, rules: &Rules, max_time: f64)
    -> KickBallTarget {
    use crate::my_strategy::simulator::integrate_movement;

    let far_time = integrate_movement(
        kick_ball_position,
        final_robot_speed,
        rules.ROBOT_MAX_GROUND_SPEED,
        rules.ROBOT_ACCELERATION,
        rules.tick_time_interval(),
        max_time,
        &mut position,
        &mut velocity,
    );

    let near_time = integrate_movement(
        kick_ball_position,
        final_robot_speed,
        final_robot_speed,
        rules.ROBOT_ACCELERATION,
        rules.micro_tick_time_interval(),
        max_time - far_time,
        &mut position,
        &mut velocity,
    );

    KickBallTarget {
        time: far_time + near_time,
        position,
        velocity,
    }
}
