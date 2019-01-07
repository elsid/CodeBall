#[test]
fn test_get_kick_ball_position() {
    use my_strategy::examples::{example_ball, example_rules};
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;
    use my_strategy::my_strategy::kick::get_jump_position;

    let mut ball = example_ball();
    let rules = example_rules();
    ball.y = 2.0;
    assert_eq!(
        get_jump_position(
            0.0,
            get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap(),
            ball.position(),
            &rules,
        ).distance(ball.position()),
        3.0000000000000004
    );
    ball.y = 3.0;
    assert_eq!(
        get_jump_position(
            1.0,
            get_min_distance_between_spheres(ball.y, rules.BALL_RADIUS, rules.ROBOT_MIN_RADIUS).unwrap(),
            ball.position(),
            &rules,
        ).distance(ball.position()),
        3.0
    );
    ball.y = 4.0;
    assert_eq!(
        get_jump_position(
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
    use my_strategy::my_strategy::kick::get_jump_target;

    let robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_jump_target(
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
    use my_strategy::my_strategy::kick::get_jump_target;

    let mut robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    robot.set_velocity(
        (ball.position() - robot.position()).with_y(0.0).normalized()
            * rules.ROBOT_MAX_GROUND_SPEED
    );
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_jump_target(
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
    use my_strategy::my_strategy::kick::get_jump_target;

    let mut robot = example_me();
    let ball = example_ball();
    let rules = example_rules();
    robot.set_velocity(
        (ball.position() - robot.position()).with_y(0.0).normalized()
            * rules.ROBOT_MAX_GROUND_SPEED
    );
    let kick_ball_position = ball.position().with_y(robot.position().y());
    let result = get_jump_target(
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
fn test_get_ball_movement() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;
    use my_strategy::my_strategy::kick::{get_jump_position, get_jump_target, get_ball_movement};

    let mut ball = example_ball();
    ball.y = 3.5651676666668153;
    ball.velocity_y = 8.815499999999721;
    let robot = example_me();
    let rules = example_rules();
    let angle = -1.6;
    let distance = 1.6;
    let jump_position = get_jump_position(angle, distance, ball.position(), &rules);
    let max_time = 3.0;
    let jump_target = get_jump_target(
        robot.position(),
        robot.velocity(),
        30.0,
        jump_position,
        &rules,
        max_time,
    );
    assert_eq!(jump_target.time / rules.tick_time_interval(), 45.78);
    let jump_speed = 14.4;
    let target = Vec3::new(0.0, 7.0, 42.0);
    let result = get_ball_movement(
        target,
        jump_speed,
        &jump_target,
        &robot,
        &ball,
        &rules,
        max_time,
    );
    assert_eq!(result.time_to_target, 2.227630778708753);
    assert_eq!(
        result.move_equation.get_position(result.time_to_target),
        Vec3::new(1.1483946162964405, 7.933926622071425, 37.57295774065914)
    );
}

#[test]
fn test_get_optimal_kick() {
    use my_strategy::examples::{example_ball, example_me, example_rules};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::kick::{KickParams, get_optimal_kick};
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    let rules = example_rules();
    let time = 46.0 * rules.tick_time_interval();
    let mut ball = example_ball();
    ball.y = 3.5651676666668153;
    ball.velocity_y = 8.815499999999721;
    let robot = example_me();
    let max_time = 3.0;
    let max_iter = 30;
    let target = Vec3::new(0.0, 7.0, 42.0);
    let result = get_optimal_kick(
        target,
        time,
        &ball,
        &robot,
        &rules,
        max_time,
        max_iter,
    );
    assert_eq!(
        result.params,
        KickParams {
            angle: -1.569135955309053,
            distance: 1.6419577587310836,
            final_speed: 29.97720305173167,
            jump_speed: 15.0,
        }
    );
    assert_eq!(
        result.ball_movement.move_equation
            .get_position(result.ball_movement.time_to_target).distance(target),
        2.9117982814505172
    );
    assert_eq!(
        result.jump_target.position.distance(ball.position()),
        3.0075705162207376
    );
}
