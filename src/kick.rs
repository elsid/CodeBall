use crate::model::{Robot, Ball, Rules};
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::physics::MoveEquation;

#[derive(Debug, Clone, PartialEq)]
pub struct KickParams {
    pub angle: f64,
    pub distance: f64,
    pub final_speed: f64,
    pub jump_speed: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Kick {
    pub params: KickParams,
    pub jump_target: JumpTarget,
    pub ball_movement: BallMovement,
}

pub fn get_optimal_kick(target: Vec3, time: f64, ball: &Ball, robot: &Robot, rules: &Rules,
                        max_time: f64, max_iter: usize) -> Kick {
    use ndarray::prelude::{Array, ArrayView1};
    use optimize::{NelderMeadBuilder, Minimizer};
    use crate::my_strategy::common::{Square, Clamp};
    use crate::my_strategy::optimization::optimize1d;
    use crate::my_strategy::physics::get_min_distance_between_spheres;

    let optimizer = NelderMeadBuilder::default()
        .maxiter(max_iter)
        .build()
        .unwrap();
    let min_distance = get_min_distance_between_spheres(
        ball.y,
        rules.BALL_RADIUS,
        rules.ROBOT_MIN_RADIUS,
    ).unwrap_or(0.0);
    let max_distance = rules.ROBOT_MIN_RADIUS + rules.BALL_RADIUS;
    let initial_angle = (ball.position() - target).with_y(0.0).normalized().absolute_rotation_by_y();
    let initial_distance = min_distance;
    let initial_max_speed = rules.ROBOT_MAX_GROUND_SPEED;
    let initial_jump_speed = rules.ROBOT_MAX_JUMP_SPEED;
    let args = Array::from_vec(vec![
        initial_angle,
        initial_distance,
        initial_max_speed,
        initial_jump_speed,
    ]);
    let get_jump_position = |angle: f64, distance: f64| {
        get_jump_position(distance, angle, ball.position(), rules)
    };
    let get_jump_target = |max_speed: f64, jump_position: Vec3| {
        get_jump_target(
            robot.position(),
            robot.velocity(),
            max_speed,
            jump_position,
            rules,
            time + rules.tick_time_interval(),
        )
    };
    let get_ball_movement = |jump_speed: f64, kick_ball_target: &JumpTarget| {
        get_ball_movement(target, jump_speed, &kick_ball_target, robot, ball, rules, max_time + 2.0)
    };
    let clamp_angle = |v: f64| {
        v.clamp(
            initial_angle - std::f64::consts::PI,
            initial_angle + std::f64::consts::PI,
        )
    };
    let clamp_distance = |v: f64| v.clamp(min_distance, max_distance);
    let clamp_final_speed = |v: f64| v.clamp(0.0, rules.ROBOT_MAX_GROUND_SPEED);
    let clamp_jump_speed = |v: f64| v.clamp(0.0, rules.ROBOT_MAX_JUMP_SPEED);
    let get_penalty = |angle: f64, distance: f64, final_speed: f64, jump_speed: f64| {
        let clamped_angle = clamp_angle(angle);
        let clamped_distance = clamp_distance(distance);
        let clamped_final_speed = clamp_final_speed(final_speed);
        let clamped_jump_speed = clamp_jump_speed(jump_speed);
        let jump_position = get_jump_position(clamped_distance, clamped_angle);
        let jump_target = get_jump_target(clamped_final_speed, jump_position);
        let time_distance = (jump_target.time - time).abs();
        let distance_to_jump_position = jump_target.position.distance(jump_position);
        let angle_overflow = (angle - clamped_angle).square();
        let distance_overflow = (distance - clamped_distance).square();
        let final_speed_overflow = (final_speed - clamped_final_speed).square();
        let jump_overflow = (jump_speed - clamped_jump_speed).square();
        let ball_target = get_ball_movement(clamped_jump_speed, &jump_target);
        let end = ball_target.move_equation.get_position(ball_target.time_to_target);
        let cos = (end - ball.position()).cos(target - ball.position());
        let distance_to_target = end.distance(target);
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
        0.0
            + angle_overflow
            + distance_overflow
            + jump_overflow
            + final_speed_overflow
            + time_distance
            + distance_to_jump_position
            + ceiling_penetration
            + goal_ceiling_penetration
            + 9.0 * distance_to_target
    };
    let result = optimizer.minimize(
        |x: ArrayView1<f64>| get_penalty(x[0], x[1], x[2], x[3]),
        args.view()
    );
    let angle = clamp_angle(result[0]);
    let distance = clamp_distance(result[1]);
    let final_speed = clamp_final_speed(result[2]);
    let jump_speed = clamp_jump_speed(result[3]);
    let jump_position = get_jump_position(angle, distance);
    let kick_ball_target = get_jump_target(final_speed, jump_position);
    let ball_movement = get_ball_movement(jump_speed, &kick_ball_target);
    Kick {
        params: KickParams {
            angle,
            distance,
            final_speed,
            jump_speed,
        },
        jump_target: kick_ball_target,
        ball_movement,
    }
}

pub fn get_jump_position(angle: f64, distance: f64, position: Vec3, rules: &Rules) -> Vec3 {
    position.with_y(rules.ROBOT_MIN_RADIUS) + Vec3::i().rotated_by_y(angle) * distance
}

#[derive(Debug, Clone, PartialEq)]
pub struct BallMovement {
    pub move_equation: MoveEquation,
    pub time_to_target: f64,
}

pub fn get_ball_movement(target: Vec3, jump_speed: f64, jump_target: &JumpTarget,
                         robot: &Robot, ball: &Ball, rules: &Rules, max_time: f64) -> BallMovement {
    use crate::my_strategy::entity::Entity;
    use crate::my_strategy::simulator::{Simulator, RobotExt, BallExt};

    let mut robot_ext = RobotExt::from_robot(robot, rules);
    robot_ext.set_position(jump_target.position);
    robot_ext.set_velocity(jump_target.velocity);
    robot_ext.jump(jump_speed, &rules);
    rules.arena.collide(&mut robot_ext);
    let robot_move_equation = MoveEquation::from_entity(&robot_ext, rules);
    let time_to_ball = robot_move_equation.get_time_to_target(
        ball.position(),
        rules.ROBOT_MAX_RADIUS,
        max_time - jump_target.time,
        0.0,
        10,
    );
    let time_to_collide_with_ball = robot_move_equation.get_time_to_target(
        ball.position(),
        rules.ROBOT_MAX_RADIUS,
        time_to_ball,
        (rules.ROBOT_MAX_RADIUS + rules.ROBOT_MIN_RADIUS) / 2.0 + rules.BALL_RADIUS,
        10,
    );
    robot_ext.set_position(robot_move_equation.get_position(time_to_collide_with_ball));
    robot_ext.set_velocity(robot_move_equation.get_velocity(time_to_collide_with_ball));
    let mut ball_ext = BallExt::from_ball(ball, rules);
    Simulator::collide(|| rules.mean_e(), &mut robot_ext, &mut ball_ext);
    let ball_move_equation = MoveEquation::from_entity(&ball_ext, rules);
    BallMovement {
        time_to_target: ball_move_equation.get_time_to_target(
            target,
            rules.BALL_RADIUS,
            max_time - jump_target.time - time_to_collide_with_ball,
            0.0,
            10,
        ),
        move_equation: ball_move_equation,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JumpTarget {
    pub time: f64,
    pub position: Vec3,
    pub velocity: Vec3,
}

pub fn get_jump_target(mut position: Vec3, mut velocity: Vec3, final_robot_speed: f64,
                       jump_position: Vec3, rules: &Rules, max_time: f64)
                       -> JumpTarget {
    use crate::my_strategy::simulator::integrate_movement;

    let far_time = integrate_movement(
        jump_position,
        final_robot_speed,
        rules.ROBOT_MAX_GROUND_SPEED,
        rules.ROBOT_ACCELERATION,
        rules.tick_time_interval(),
        max_time,
        &mut position,
        &mut velocity,
    );

    let near_time = integrate_movement(
        jump_position,
        final_robot_speed,
        final_robot_speed,
        rules.ROBOT_ACCELERATION,
        rules.micro_tick_time_interval(),
        max_time - far_time,
        &mut position,
        &mut velocity,
    );

    JumpTarget {
        time: far_time + near_time,
        position,
        velocity,
    }
}
