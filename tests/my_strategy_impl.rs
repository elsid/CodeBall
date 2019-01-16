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

    let world = example_world(GameType::TwoRobots);
    let simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: 12.288688718619582,
        target_velocity_y: 0.0,
        target_velocity_z: -27.367647498038114,
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

    let world = example_world(GameType::TwoRobots);
    let simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: -15.811748664438161,
        target_velocity_y: 0.0,
        target_velocity_z: 25.494874076422462,
        jump_speed: 0.0,
        use_nitro: false,
    });
}

#[test]
fn test_two_robots_first_ball_kick_until_goal() {
    use my_strategy::model::Ball;
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::{Simulator, RobotCollisionType, Solid};
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::entity::Entity;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let mut simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().position().y() - simulator.me().radius() < 1e-3
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.me().position().y(), 1.293141297487357);
    assert_eq!(simulator.current_tick(), 38);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().collision_type() == RobotCollisionType::None
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.ball().base(), &Ball {
        x: -0.016130474632929347,
        y: 3.28926242990611,
        z: 0.1528015345870449,
        velocity_x: -3.9640379758286053,
        velocity_y: 18.321202756258515,
        velocity_z: 37.55072926567292,
        radius: 2.0,
    });
    assert_eq!(simulator.me().action().jump_speed, 15.0);
    assert_eq!(simulator.current_tick(), 44);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 111);
}

#[test]
fn test_two_robots_first_ball_kick_until_goal_with_nitro() {
    use my_strategy::model::Ball;
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::{Simulator, RobotCollisionType, Solid};
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::entity::Entity;

    let world = example_world(GameType::TwoRobotsWithNitro);
    let mut rng = example_rng();
    let mut simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().position().y() - simulator.me().radius() < 1e-3
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.me().position().y(), 1.293141297487357);
    assert_eq!(simulator.current_tick(), 38);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().collision_type() == RobotCollisionType::None
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.ball().base(), &Ball {
        x: -0.04715537713809805,
        y: 3.338259065077118,
        z: 0.36615825476218744,
        velocity_x: -5.233193799383277,
        velocity_y: 19.094128107628052,
        velocity_z: 40.63538931737973,
        radius: 2.0,
    });
    assert_eq!(simulator.me().action().jump_speed, 15.0);
    assert_eq!(simulator.current_tick(), 44);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 106);
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
