#[test]
fn test_try_play() {
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

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 1145);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-15.811748664438161, 0.0, 25.494874076422462));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_at_position",
        time_to_jump: 0.733333333333334,
        time_to_watch: 0.7666666666666674,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 4,
        total_iterations: 5,
        game_score: 0,
        order_score: 1145,
        scenario_micro_ticks: 300,
        play_micro_ticks: 5375,
        game_micro_ticks: 5375,
        game_micro_ticks_limit: 28000,
        current_step: 8,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 0,
        ticks_with_far_micro_ticks: 100,
    });
}

#[test]
fn test_try_play_should_not_jump_on_ball_top() {
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

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 1278);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(15.065425465228744, 0.0, 25.942878705950065));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "jump_at_position",
        time_to_jump: 0.31666666666666665,
        time_to_watch: 0.35,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 5,
        total_iterations: 5,
        game_score: 0,
        order_score: 1278,
        scenario_micro_ticks: 300,
        play_micro_ticks: 3550,
        game_micro_ticks: 3550,
        game_micro_ticks_limit: 28000,
        current_step: 8,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 0,
        ticks_with_far_micro_ticks: 100,
    });
}

#[test]
fn test_try_play_far_jump() {
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

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 1248);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 30.0));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "jump_to_ball",
        time_to_jump: 0.0,
        time_to_watch: 0.16666666666666666,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 0,
        total_iterations: 5,
        game_score: 0,
        order_score: 1248,
        scenario_micro_ticks: 413,
        play_micro_ticks: 6113,
        game_micro_ticks: 6113,
        game_micro_ticks_limit: 28000,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 4,
        ticks_with_far_micro_ticks: 96,
    });
}

#[test]
fn test_try_play_continue_jump() {
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

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 2621);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 0.0));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "continue_jump",
        time_to_jump: 0.0,
        time_to_watch: 0.11666666666666665,
        time_to_end: 1.183333333333334,
        time_to_score: Some(1.183333333333334),
        iteration: 0,
        total_iterations: 0,
        game_score: 1,
        order_score: 2621,
        scenario_micro_ticks: 367,
        play_micro_ticks: 821,
        game_micro_ticks: 821,
        game_micro_ticks_limit: 28000,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 7,
        ticks_with_far_micro_ticks: 64,
    });
}

#[test]
fn test_try_play_continue_jump_with_nitro() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.set_position(Vec3::new(2.1244535492642953, 1.2931418435925501, -5.178084712824993));
    world.me.set_velocity(Vec3::new(-15.58616134845358, 14.488369308639712, 24.472022729043868));
    world.me.radius = 1.05;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.y = 2.123101000000013;
    world.game.ball.velocity_y = 12.815500000000347;

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 2610);
    assert_eq!(result.action().use_nitro, true);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-37.54676686691887, 14.6683757644347, 91.51545798538402));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "continue_jump",
        time_to_jump: 0.0,
        time_to_watch: 0.11666666666666665,
        time_to_end: 1.1500000000000008,
        time_to_score: Some(1.1500000000000008),
        iteration: 0,
        total_iterations: 0,
        game_score: 1,
        order_score: 2610,
        scenario_micro_ticks: 361,
        play_micro_ticks: 1621,
        game_micro_ticks: 1621,
        game_micro_ticks_limit: 28000,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 7,
        ticks_with_far_micro_ticks: 62,
    });
}


#[test]
fn test_try_play_when_far_from_ball_at_my_side() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);

    world.me.set_position(Vec3::new(20.0, 1.0, -30.0));
    world.me.radius = 1.05;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(Vec3::new(-20.0, 10.0, 30.0));

    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 800);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-16.148150133875735, 0.0, 25.283141562191375));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_at_position",
        time_to_jump: 0.0,
        time_to_watch: 0.0,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 1,
        total_iterations: 5,
        game_score: 0,
        order_score: 800,
        scenario_micro_ticks: 300,
        play_micro_ticks: 4000,
        game_micro_ticks: 4000,
        game_micro_ticks_limit: 28000,
        current_step: 3,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 0,
        ticks_with_far_micro_ticks: 100,
    });
}

#[test]
fn test_try_play_when_far_from_ball_at_opponent_side() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);

    world.me.set_position(Vec3::new(-20.0, 1.0, 30.0));
    world.me.radius = 1.05;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(Vec3::new(20.0, 10.0, -30.0));

    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 334);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(16.148150133875735, 0.0, -25.283141562191375));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_at_position",
        time_to_jump: 0.0,
        time_to_watch: 0.0,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 1,
        total_iterations: 5,
        game_score: 0,
        order_score: 334,
        scenario_micro_ticks: 300,
        play_micro_ticks: 4000,
        game_micro_ticks: 4000,
        game_micro_ticks_limit: 28000,
        current_step: 3,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 0,
        ticks_with_far_micro_ticks: 100,
    });
}

#[test]
fn test_try_play_goalkeeper_should_catch_but_cant() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;
    use my_strategy::my_strategy::roles::Goalkeeper;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.set_position(world.rules.get_goalkeeper_position());
    world.me.nitro_amount = 50.0;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(Vec3::new(0.198560151715065, 4.92791046901793, -1.66068357870943));
    world.game.ball.set_velocity(Vec3::new(5.10521022216499, 16.6258312833173, -42.698087751137));

    let result = Order::try_play(&world.me, &world, &[], Goalkeeper::max_z(&world), &mut ctx);

    assert_eq!(result.score(), 500);
    assert_eq!(result.action().use_nitro, false);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(9.87305861387808, 0.0, -28.32883184331694));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_at_position",
        time_to_jump: 0.5833333333333335,
        time_to_watch: 0.7666666666666674,
        time_to_end: 1.566666666666666,
        time_to_score: Some(1.566666666666666),
        iteration: 2,
        total_iterations: 5,
        game_score: -1,
        order_score: 500,
        scenario_micro_ticks: 282,
        play_micro_ticks: 5051,
        game_micro_ticks: 5051,
        game_micro_ticks_limit: 28000,
        current_step: 4,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 0,
        ticks_with_far_micro_ticks: 94,
    });
}

#[test]
fn test_try_play_goalkeeper_should_catch() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, OrderContext};
    use my_strategy::my_strategy::common::IdGenerator;
    use my_strategy::my_strategy::roles::Goalkeeper;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng();
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = OrderContext {
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.set_position(Vec3::new(2.6398424813638695, 1.0, -41.95171478620124));
    world.me.set_velocity(Vec3::new(6.335451032857034, 0.0, -2.036252062849781));
    world.me.nitro_amount = 50.0;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(Vec3::new(2.6660784257613335, 8.492895589287127, -22.298092658424864));
    world.game.ball.set_velocity(Vec3::new(5.10521022216499, 0.1258312833164129, -42.698087751137));

    let result = Order::try_play(&world.me, &world, &[], Goalkeeper::max_z(&world), &mut ctx);

    assert_eq!(result.score(), 768);
    assert_eq!(result.action().use_nitro, false);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(28.561044526277186, 0.0, -9.179691474554682));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "jump_to_ball",
        time_to_jump: 0.0,
        time_to_watch: 0.44999999999999996,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 0,
        total_iterations: 5,
        game_score: 0,
        order_score: 768,
        scenario_micro_ticks: 391,
        play_micro_ticks: 3537,
        game_micro_ticks: 3537,
        game_micro_ticks_limit: 28000,
        current_step: 0,
        reached_game_limit: false,
        reached_play_limit: false,
        reached_scenario_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 3,
        ticks_with_far_micro_ticks: 97,
    });
}
