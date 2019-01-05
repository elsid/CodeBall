#[test]
fn test_order_for_ball_target() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::targets::Target;
    use my_strategy::my_strategy::orders::Order;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let target = Target::Ball;
    let result = Order::new(None, &target, &world.me, &world, &mut rng);

    assert_eq!(result.score(), 628);
    assert_eq!(result.name(), "play");
    assert_eq!(result.action().target_velocity(), Vec3::new(-16.72247725643346, 0.0, 24.907002116032867));
    assert_eq!(result.action().jump_speed, 0.0);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), Stats {
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
fn test_order_for_ball_target_should_not_jump_on_ball_top() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::targets::Target;
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
    let target = Target::Ball;

    let result = Order::new(None, &target, &world.me, &world, &mut rng);

    assert_eq!(result.score(), 951);
    assert_eq!(result.name(), "play");
    assert_eq!(result.action().target_velocity(), Vec3::new(18.636335025013768, 0.0, 23.509296391756287));
    assert_eq!(result.action().jump_speed, 0.0);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), Stats {
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
