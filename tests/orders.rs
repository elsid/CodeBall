#[test]
fn test_new() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::Order;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(-15.381759823977964, 0.0, 25.756580998212037));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1128);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats, Stats {
        micro_ticks_to_jump: 132,
        micro_ticks_to_watch: 138,
        micro_ticks_to_end: 300,
        time_to_jump: 0.733333333333334,
        time_to_watch: 0.7666666666666674,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 4,
        total_iterations: 5,
        score: 0,
        jump_simulation: false,
        far_jump_simulation: false,
        action_score: 1128,
        total_micro_ticks: 8950,
        current_step: 8,
    });
}

#[test]
fn test_new_should_not_jump_on_ball_top() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::Order;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);

    world.me.id = 2;
    world.me.set_position(Vec3::new(-5.838617159216834, 1.0, -10.508900380791133));
    world.me.set_velocity(Vec3::new(16.91322406429886, 0.0, 24.77759529887104));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 5.233161866399729;
    world.game.ball.velocity_y = -12.500000000000554;

    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(17.24955662441977, 0.0, 24.544913857272668));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1355);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats, Stats {
        micro_ticks_to_jump: 54,
        micro_ticks_to_watch: 66,
        micro_ticks_to_end: 300,
        time_to_jump: 0.3,
        time_to_watch: 0.36666666666666664,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 4,
        total_iterations: 5,
        score: 0,
        jump_simulation: false,
        far_jump_simulation: false,
        action_score: 1355,
        total_micro_ticks: 8325,
        current_step: 8,
    });
}

#[test]
fn test_new_far_jump() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::Order;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);

    world.me.id = 2;
    world.me.set_position(Vec3::new(0.0, 1.0, -5.0));
    world.me.set_velocity(Vec3::new(0.0, 0.0, 30.0));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 6.0;
    world.game.ball.velocity_y = 0.0;

    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(0.0, 0.0, 30.0));
    assert_eq!(result.action.jump_speed, 15.0);
    assert_eq!(result.score, 1171);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats, Stats {
        micro_ticks_to_jump: 0,
        micro_ticks_to_watch: 1018,
        micro_ticks_to_end: 1018,
        time_to_jump: 0.0,
        time_to_watch: 0.7666666666666674,
        time_to_end: 0.7666666666666674,
        time_to_score: None,
        iteration: 0,
        total_iterations: 5,
        score: 0,
        jump_simulation: false,
        far_jump_simulation: true,
        action_score: 1171,
        total_micro_ticks: 9743,
        current_step: 0,
    });
}
