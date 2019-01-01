#[test]
fn test_pass_ball_to_position() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::scenarios::{PassBallToPosition, Context};
    use my_strategy::my_strategy::history::Stats;
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::entity::Entity;
    use my_strategy::examples::{example_world, example_rng};

    let mut rng = example_rng();
    let world = example_world();
    let kick_ball_time = 46.0 * world.rules.tick_time_interval();
    let rules = &world.rules;
    let robot = &world.me;
    let ball = {
        let mut ball_simulator = Simulator::new(&world, robot.id);
        while ball_simulator.current_time() < kick_ball_time {
            ball_simulator.tick(rules.tick_time_interval(), rules.MICROTICKS_PER_TICK, &mut rng);
        };
        ball_simulator.ball().base().clone()
    };
    let target = Vec3::new(-4.0, 2.0, 21.0);
    let mut simulator = Simulator::new(&world, robot.id);
    let mut history = Vec::new();
    let mut stats = Stats::default();
    let mut time_to_ball = None;

    let mut context = Context {
        current_tick: 0,
        robot_id: 0,
        action_id: 0,
        simulator: &mut simulator,
        rng: &mut rng,
        history: &mut history,
        stats: &mut stats,
        time_to_ball: &mut time_to_ball,
    };

    let result = PassBallToPosition {
        ball: &ball,
        ball_target: target,
        kick_ball_time,
        max_time: kick_ball_time + 200.0 * world.rules.tick_time_interval(),
        max_iter: 30,
        tick_time_interval: rules.tick_time_interval(),
        micro_ticks_per_tick_before_jump: rules.MICROTICKS_PER_TICK,
        micro_ticks_per_tick_after_jump: rules.MICROTICKS_PER_TICK,
        max_micro_ticks: std::i32::MAX,
    }.perform(&mut context);

    assert!(result.is_some(), format!("{:?}", result));
    assert_eq!(simulator.ball().position().distance(target), 0.5586802561809826);
    assert_eq!(simulator.current_time() / world.rules.tick_time_interval(), 157.99999999999974);
}
