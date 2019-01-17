#[test]
fn test_new() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &mut ctx).unwrap();

    assert_eq!(result.score(), 1189);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-15.811748664438161, 0.0, 25.494874076422462));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_at_position",
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
        action_score: 1189,
        total_micro_ticks: 5350,
        current_step: 8,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
    });
}

#[test]
fn test_new_should_not_jump_on_ball_top() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.id = 2;
    world.me.set_position(Vec3::new(-5.838617159216834, 1.0, -10.508900380791133));
    world.me.set_velocity(Vec3::new(16.91322406429886, 0.0, 24.77759529887104));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 5.233161866399729;
    world.game.ball.velocity_y = -12.500000000000554;

    let result = Order::try_play(&world.me, &world, &mut ctx).unwrap();

    assert_eq!(result.score(), 1325);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(15.065425465228744, 0.0, 25.942878705950065));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "jump_at_position",
        micro_ticks_to_jump: 57,
        micro_ticks_to_watch: 63,
        micro_ticks_to_end: 300,
        time_to_jump: 0.31666666666666665,
        time_to_watch: 0.35,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 5,
        total_iterations: 5,
        score: 0,
        jump_simulation: false,
        far_jump_simulation: false,
        action_score: 1325,
        total_micro_ticks: 3525,
        current_step: 8,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
    });
}

#[test]
fn test_new_far_jump() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.id = 2;
    world.me.set_position(Vec3::new(0.0, 1.0, -5.0));
    world.me.set_velocity(Vec3::new(0.0, 0.0, 30.0));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 6.0;
    world.game.ball.velocity_y = 0.0;

    let result = Order::try_play(&world.me, &world, &mut ctx).unwrap();

    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 30.0));
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.score(), 1292);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "jump_to_ball",
        micro_ticks_to_jump: 0,
        micro_ticks_to_watch: 118,
        micro_ticks_to_end: 388,
        time_to_jump: 0.0,
        time_to_watch: 0.16666666666666666,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 0,
        total_iterations: 5,
        score: 0,
        jump_simulation: false,
        far_jump_simulation: true,
        action_score: 1292,
        total_micro_ticks: 6113,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
    });
}

#[test]
fn test_new_continue_jump() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.set_position(Vec3::new(2.1936554230690004, 1.2931423061355878, -5.139036703684824));
    world.me.set_velocity(Vec3::new(-14.028384818473757, 14.488397341551114, 25.018417478389015));
    world.me.radius = 1.05;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.y = 2.123101000000013;
    world.game.ball.velocity_y = 12.815500000000347;

    let result = Order::try_play(&world.me, &world, &mut ctx).unwrap();

    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.score(), 2671);

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "continue_jump",
        micro_ticks_to_jump: 0,
        micro_ticks_to_watch: 175,
        micro_ticks_to_end: 367,
        time_to_jump: 0.0,
        time_to_watch: 0.11666666666666665,
        time_to_end: 1.183333333333334,
        time_to_score: Some(1.183333333333334),
        iteration: 0,
        total_iterations: 0,
        score: 1,
        jump_simulation: false,
        far_jump_simulation: false,
        action_score: 2671,
        total_micro_ticks: 821,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
    });
}
