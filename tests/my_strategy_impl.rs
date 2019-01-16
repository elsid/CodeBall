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
    let simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: -11.91038935845869,
        target_velocity_y: 0.0,
        target_velocity_z: -27.53438986667244,
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
    let simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );
    let mut action = Action::default();
    my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
    assert_eq!(action, Action {
        target_velocity_x: 16.28549293405953,
        target_velocity_y: 0.0,
        target_velocity_z: 25.19489471886511,
        jump_speed: 0.0,
        use_nitro: false,
    });
}

#[test]
fn test_two_robots_first_ball_kick() {
    use my_strategy::model::Ball;
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::{Simulator, RobotCollisionType, Solid};
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
    use my_strategy::my_strategy::entity::Entity;

    let world = example_world(GameType::TwoRobots);
    let mut rng = example_rng();
    let mut simulator = Simulator::new(&world, 2);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().position().y() - simulator.me().radius() < 1e-3
            && simulator.current_tick() < 100
    });

    assert_eq!(simulator.me().position().y(), 1.2931412499999937);
    assert_eq!(simulator.current_tick(), 44);

    simulate_while(&mut my_strategy, &mut simulator, &mut rng, |simulator| {
        simulator.me().ball_collision_type() == RobotCollisionType::None
            && simulator.current_tick() < 100
    });

    assert_eq!(simulator.ball().base(), &Ball {
        x: 0.054987696231321916,
        y: 3.587902646169355,
        z: 0.20805867815500878,
        velocity_x: 8.018268316012518,
        velocity_y: 34.66278284658389,
        velocity_z: 30.338974375352638,
        radius: 2.0,
    });
    assert_eq!(simulator.current_tick(), 45);
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
