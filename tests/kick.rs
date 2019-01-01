#[test]
fn test_get_kick_ball_position() {
    use my_strategy::examples::{example_ball, example_rules};
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;
    use my_strategy::my_strategy::kick::get_kick_ball_position;

    let mut ball = example_ball();
    let rules = example_rules();
    ball.y = 2.0;
    assert_eq!(
        get_kick_ball_position(
            0.0,
            get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap(),
            ball.position(),
            &rules,
        ).distance(ball.position()),
        3.0000000000000004
    );
    ball.y = 3.0;
    assert_eq!(
        get_kick_ball_position(
            1.0,
            get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap(),
            ball.position(),
            &rules,
        ).distance(ball.position()),
        3.0
    );
    ball.y = 4.0;
    assert_eq!(
        get_kick_ball_position(
            2.0,
            get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap(),
            ball.position(),
            &rules,
        ).distance(ball.position()),
        3.0
    );
}

#[test]
fn test_get_kick_ball_target_with_speed_increase() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::kick::get_kick_ball_target;

    let robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_kick_ball_target(
        robot.position(),
        robot.velocity(),
        30.0,
        kick_ball_position,
        &rules,
        1.0,
    );
    assert!(result.position.distance(kick_ball_position) < 1.0,
            format!("{}", result.position.distance(kick_ball_position)));
    assert!((result.velocity.norm() - 30.0).abs() < 1e-2,
            format!("{}", result.velocity.norm()));
    assert_eq!(result.time, 0.8083333333333333);
}

#[test]
fn test_get_kick_ball_target_with_speed_decrease() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::kick::get_kick_ball_target;

    let mut robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    robot.set_velocity(
        (ball.position() - robot.position()).with_y(0.0).normalized()
            * rules.ROBOT_MAX_GROUND_SPEED
    );
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_kick_ball_target(
        robot.position(),
        robot.velocity(),
        20.0,
        kick_ball_position,
        &rules,
        2.0,
    );
    assert!(result.position.distance(kick_ball_position) < 1.0,
            format!("{}", result.position.distance(kick_ball_position)));
    assert!((result.velocity.norm() - 20.0).abs() < 1e-2,
            format!("{}", result.velocity.norm()));
    assert_eq!(result.time, 0.6958333333333333);
}

#[test]
fn test_get_kick_ball_target_with_time_limit() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::kick::get_kick_ball_target;

    let mut robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    robot.set_velocity(
        (ball.position() - robot.position()).with_y(0.0).normalized()
            * rules.ROBOT_MAX_GROUND_SPEED
    );
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_kick_ball_target(
        robot.position(),
        robot.velocity(),
        30.0,
        kick_ball_position,
        &rules,
        0.5,
    );
    assert!((result.time - 0.5).abs() < 0.1, format!("{}", result.time));
    assert!((result.velocity.norm() - 30.0).abs() < 1e-1,
            format!("{}", result.velocity.norm()));
}

#[test]
fn test_get_ball_target() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;
    use my_strategy::my_strategy::kick::{get_kick_ball_position, get_kick_ball_target, get_ball_movement};

    let mut ball = example_ball();
    ball.y = 3.254651000000122;
    ball.velocity_y = 9.815499999999878;
    let robot = example_me();
    let rules = example_rules();
    let angle = -1.6;
    let distance = get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap();
    let kick_ball_position = get_kick_ball_position(angle, distance, ball.position(), &rules);
    let max_time = 3.0;
    let kick_ball_target = get_kick_ball_target(
        robot.position(),
        robot.velocity(),
        rules.ROBOT_MAX_GROUND_SPEED,
        kick_ball_position,
        &rules,
        max_time,
    );
    assert!(kick_ball_target.time < 2.0, format!("{}", kick_ball_target.time));
    let jump_speed = rules.ROBOT_MAX_JUMP_SPEED;
    let result = get_ball_movement(
        rules.arena.get_goal_target(rules.BALL_RADIUS),
        jump_speed,
        &kick_ball_target,
        &robot,
        &ball,
        &rules,
        max_time,
    );
    assert_eq!(result.time_to_target, 1.8662978943833592);
    assert_eq!(
        result.move_equation.get_position(result.time_to_target),
        Vec3::new(0.8051418515723323, 1.9492903216222501, 28.670753286418115)
    );
}

#[test]
fn test_get_optimal_kick() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;
    use my_strategy::my_strategy::kick::get_optimal_kick;

    let rules = example_rules();
    let time = 46.0 * rules.tick_time_interval();
    let mut ball = example_ball();
    ball.y = 3.5651676666668153;
    ball.velocity_y = 8.815499999999721;
    let robot = example_me();
    let distance = get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap();
    let max_time = 3.0;
    let max_iter= 30;
    let target = Vec3::new(-4.0, 2.0, 21.0);
    let result = get_optimal_kick(
        target,
        distance,
        time,
        &ball,
        &robot,
        &rules,
        max_time,
        max_iter,
    );
    assert_eq!(
        result.ball_movement.move_equation
            .get_position(result.ball_movement.time_to_target).distance(target),
        0.6298453248146962
    );
    assert_eq!(
        result.kick_ball_target.position.distance(ball.position()),
        3.00244109611291
    );
}
