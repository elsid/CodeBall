#[test]
fn test_try_play() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 1227);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-14.622886891738023, 0.0, 26.194869324954386));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.7166666666666672,
        time_to_watch: 0.7666666666666674,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 37,
        total_iterations: 76,
        game_score: 0,
        order_score: 1227,
        path_micro_ticks: 300,
        plan_micro_ticks: 6275,
        game_micro_ticks: 6275,
        game_micro_ticks_limit: 30000,
        current_step: 4,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 34,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_position", "jump", "watch_me_jump", "watch_ball_move"],
    });
}

#[test]
fn test_try_play_should_not_jump_on_ball_top() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::{Order, Context};
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
    let mut ctx = Context {
        config: &world.config,
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

    assert_eq!(result.score(), 1471);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(16.985123857184913, 0.0, 24.72863861105354));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.31666666666666665,
        time_to_watch: 0.36666666666666664,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 60,
        total_iterations: 61,
        game_score: 0,
        order_score: 1471,
        path_micro_ticks: 300,
        plan_micro_ticks: 4450,
        game_micro_ticks: 4450,
        game_micro_ticks_limit: 30000,
        current_step: 8,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 33,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_position", "jump", "watch_me_jump", "watch_ball_move"],
    });
}

#[test]
fn test_try_play_far_jump() {
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::{Order, Context};
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
    let mut ctx = Context {
        config: &world.config,
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

    assert_eq!(result.score(), 1362);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 30.0));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 2,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.16666666666666666,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 3,
        total_iterations: 103,
        game_score: 0,
        order_score: 1362,
        path_micro_ticks: 413,
        plan_micro_ticks: 8681,
        game_micro_ticks: 8681,
        game_micro_ticks_limit: 30000,
        current_step: 0,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 4,
        ticks_with_far_micro_ticks: 96,
        path: vec!["far_jump", "watch_me_jump", "watch_ball_move"],
    });
}

#[test]
fn test_try_play_continue_jump() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobots);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
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

    assert_eq!(result.score(), 3018);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(0.0, 0.0, 0.0));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.11666666666666665,
        time_to_end: 1.183333333333334,
        time_to_score: Some(1.183333333333334),
        iteration: 2,
        total_iterations: 5,
        game_score: 1,
        order_score: 3018,
        path_micro_ticks: 367,
        plan_micro_ticks: 821,
        game_micro_ticks: 821,
        game_micro_ticks_limit: 30000,
        current_step: 0,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 7,
        ticks_with_far_micro_ticks: 64,
        path: vec!["watch_me_jump", "watch_ball_move"],
    });
}

#[test]
fn test_try_play_continue_jump_with_nitro() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.me.set_position(Vec3::new(2.1244535492642953, 1.2931418435925501, -5.178084712824993));
    world.me.set_velocity(Vec3::new(-14.027545213608223, 13.03953237777574, 22.02482045613948));
    world.me.radius = 1.05;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.y = 2.123101000000013;
    world.game.ball.velocity_y = 12.815500000000347;

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 3054);
    assert_eq!(result.action().use_nitro, true);
    assert_eq!(result.action().jump_speed, 15.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-37.54676686691887, 14.6683757644347, 91.51545798538402));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.11666666666666665,
        time_to_end: 1.2333333333333338,
        time_to_score: Some(1.2333333333333338),
        iteration: 6,
        total_iterations: 9,
        game_score: 1,
        order_score: 3054,
        path_micro_ticks: 376,
        plan_micro_ticks: 1747,
        game_micro_ticks: 1747,
        game_micro_ticks_limit: 30000,
        current_step: 0,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 7,
        ticks_with_far_micro_ticks: 67,
        path: vec!["watch_me_jump", "watch_ball_move"],
    });
}


#[test]
fn test_try_play_when_far_from_ball_at_my_side() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
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

    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 951);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-16.641005886756876, 0.0, 24.961508830135312));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.0,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 4,
        total_iterations: 21,
        game_score: 0,
        order_score: 951,
        path_micro_ticks: 300,
        plan_micro_ticks: 4300,
        game_micro_ticks: 4300,
        game_micro_ticks_limit: 30000,
        current_step: 1,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 36,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_position"],
    });
}

#[test]
fn test_try_play_when_far_from_ball_at_opponent_side() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
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

    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 394);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(16.641005886756876, 0.0, -24.961508830135312));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.0,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 4,
        total_iterations: 21,
        game_score: 0,
        order_score: 394,
        path_micro_ticks: 300,
        plan_micro_ticks: 4300,
        game_micro_ticks: 4300,
        game_micro_ticks_limit: 30000,
        current_step: 1,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 36,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_position"],
    });
}

#[test]
fn test_try_play_goalkeeper_should_catch_but_cant() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;
    use my_strategy::my_strategy::roles::Goalkeeper;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    world.game.ball.set_position(Vec3::new(0.198560151715065, 4.92791046901793, -1.66068357870943));
    world.game.ball.set_velocity(Vec3::new(5.10521022216499, 16.6258312833173, -42.698087751137));
    world.me.set_position(world.rules.get_goalkeeper_position(world.game.ball.position()));
    world.me.nitro_amount = 50.0;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());

    let result = Order::try_play(&world.me, &world, &[], Goalkeeper::max_z(&world), &mut ctx);

    assert_eq!(result.score(), -995);
    assert_eq!(result.action().use_nitro, false);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(26.73720956999724, 0.0, 13.605940776368541));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.0,
        time_to_watch: 0.0,
        time_to_end: 0.9500000000000013,
        time_to_score: Some(0.9500000000000013),
        iteration: 12,
        total_iterations: 49,
        game_score: -1,
        order_score: -995,
        path_micro_ticks: 342,
        plan_micro_ticks: 11120,
        game_micro_ticks: 11120,
        game_micro_ticks_limit: 30000,
        current_step: 8,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 86,
        ticks_with_far_micro_ticks: 57,
        path: vec!["fork_ball", "walk_to_ball"],
    });
}

#[test]
fn test_try_play_goalkeeper_should_catch() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;
    use my_strategy::my_strategy::roles::Goalkeeper;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let mut world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
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

    assert_eq!(result.score(), 963);
    assert_eq!(result.action().use_nitro, false);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-11.479472765463528, 0.0, 27.7168126779935));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.06666666666666667,
        time_to_watch: 0.39999999999999997,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 32,
        total_iterations: 79,
        game_score: 0,
        order_score: 963,
        path_micro_ticks: 315,
        plan_micro_ticks: 7609,
        game_micro_ticks: 7609,
        game_micro_ticks_limit: 30000,
        current_step: 3,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 34,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_ball", "jump", "watch_me_jump", "watch_ball_move"],
    });
}

#[test]
fn test_try_play_for_tree_robots_with_nitro() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::{Order, Context};
    use my_strategy::my_strategy::common::IdGenerator;

    #[cfg(feature = "enable_stats")]
    use my_strategy::my_strategy::stats::Stats;

    let world = example_world(GameType::ThreeRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut order_id_generator = IdGenerator::new();
    let mut micro_ticks = 0;
    let mut ctx = Context {
        config: &world.config,
        rng: &mut rng,
        order_id_generator: &mut order_id_generator,
        micro_ticks: &mut micro_ticks,
    };

    let result = Order::try_play(&world.me, &world, &[], std::f64::MAX, &mut ctx);

    assert_eq!(result.score(), 1289);
    assert_eq!(result.action().jump_speed, 0.0);
    assert_eq!(result.action().target_velocity(), Vec3::new(-16.314604599267437, 0.0, 25.17605363772412));

    #[cfg(feature = "enable_stats")]
    assert_eq!(result.stats(), &Stats {
        player_id: 1,
        robot_id: 1,
        current_tick: 0,
        order: "play",
        time_to_jump: 0.733333333333334,
        time_to_watch: 0.7833333333333341,
        time_to_end: 1.6666666666666656,
        time_to_score: None,
        iteration: 60,
        total_iterations: 76,
        game_score: 0,
        order_score: 1289,
        path_micro_ticks: 300,
        plan_micro_ticks: 6275,
        game_micro_ticks: 6275,
        game_micro_ticks_limit: 30000,
        current_step: 8,
        reached_game_limit: false,
        reached_plan_limit: false,
        reached_path_limit: false,
        other_number: 0,
        ticks_with_near_micro_ticks: 50,
        ticks_with_far_micro_ticks: 100,
        path: vec!["fork_ball", "walk_to_position", "jump", "watch_me_jump", "watch_ball_move"],
    });
}
