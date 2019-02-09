use my_strategy::my_strategy::simulator::Simulator;
use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
use my_strategy::my_strategy::random::XorShiftRng;

#[test]
fn test_two_robots_first_action_to_go_to_goalkeeper_position() {
    use my_strategy::model::Action;
    use my_strategy::strategy::Strategy;
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::config::Config;

    let world = example_world(GameType::TwoRobots);
    let simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: -11.32564751448835,
        target_velocity_y: 0.0,
        target_velocity_z: -27.780023548902257,
        jump_speed: 0.0,
        use_nitro: false,
    });
}

#[test]
fn test_two_robots_first_action_to_go_for_ball() {
    use my_strategy::model::Action;
    use my_strategy::strategy::Strategy;
    use my_strategy::examples::{GameType, example_world};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::config::Config;

    let world = example_world(GameType::TwoRobots);
    let simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: 16.247834600650872,
        target_velocity_y: 0.0,
        target_velocity_z: 25.21919647391432,
        jump_speed: 0.0,
        use_nitro: false,
    });
}

#[test]
fn test_two_robots_first_ball_kick_until_goal() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::config::Config;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 113);
}

#[test]
fn test_two_robots_with_nitro_first_ball_kick_until_goal() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::config::Config;

    let world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 102);
}

#[test]
fn test_two_robots_with_nitro_goalkeeper_should_catch_1() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::config::Config;

    let mut world = example_world(GameType::TwoRobotsWithNitro);

    world.game.ball.set_position(Vec3::new(0.198560151715065, 4.92791046901793, -1.66068357870943));
    world.game.ball.set_velocity(Vec3::new(5.10521022216499, 16.6258312833173, -42.698087751137));
    world.me.set_position(world.rules.get_goalkeeper_position(world.game.ball.position()));
    world.me.nitro_amount = 50.0;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());

    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 100
    });

    assert_eq!(simulator.score(), 0);
    assert_eq!(simulator.current_tick(), 100);
}

#[test]
fn test_two_robots_with_nitro_goalkeeper_should_catch() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::config::Config;

    let mut world = example_world(GameType::TwoRobotsWithNitro);

    world.game.ball.set_position(Vec3::new(0.198560151715065, 4.92791046901793, -1.66068357870943));
    world.game.ball.set_velocity(Vec3::new(5.10521022216499, 14.6258312833173, -42.698087751137));
    world.me.set_position(world.rules.get_goalkeeper_position(world.game.ball.position()));
    world.me.nitro_amount = 50.0;
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());

    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 100
    });

    assert_eq!(simulator.score(), 0);
    assert_eq!(simulator.current_tick(), 100);
}

#[test]
fn test_three_robots_with_nitro_first_ball_kick_until_goal() {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::config::Config;

    let world = example_world(GameType::ThreeRobotsWithNitro);
    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 3);
    let mut my_strategy = MyStrategyImpl::new(
        Config::default(),
        simulator.me().base(),
        simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 108);
}

fn simulate_while<P>(my_strategy: &mut MyStrategyImpl, simulator: &mut Simulator,
                     rng: &mut XorShiftRng, predicate: P)
    where P: Fn(&mut Simulator) -> bool {
    use my_strategy::model::Action;
    use my_strategy::strategy::Strategy;

    while predicate(simulator) {
        let mut action = Action::default();
        my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
        *simulator.me_mut().action_mut() = action;
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            rng,
        );
    }
}
