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

    assert_eq!(simulator.me().position().y(), 1.2931412514106515);
    assert_eq!(simulator.current_tick(), 38);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().collision_type() == RobotCollisionType::None
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.ball().base(), &Ball {
        x: -0.01203067903613229,
        y: 3.2868333721773118,
        z: 0.14248956311577,
        velocity_x: -3.2224059495834947,
        velocity_y: 18.43551781146553,
        velocity_z: 38.16569410245228,
        radius: 2.0,
    });
    assert_eq!(simulator.me().action().jump_speed, 15.0);
    assert_eq!(simulator.current_tick(), 44);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < 150
    });

    assert_eq!(simulator.score(), 1);
    assert_eq!(simulator.current_tick(), 110);
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

    assert_eq!(simulator.me().position().y(), 1.2931412514106515);
    assert_eq!(simulator.current_tick(), 38);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().collision_type() == RobotCollisionType::None
            && simulator.current_tick() < 150
    });

    assert_eq!(simulator.ball().base(), &Ball {
        x: -0.03833700749365775,
        y: 3.335190276207626,
        z: 0.3537466721261574,
        velocity_x: -4.41470246968162,
        velocity_y: 19.09000953594782,
        velocity_z: 40.735738368086174,
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
