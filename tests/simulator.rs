use my_strategy::my_strategy::vec3::Vec3;
use my_strategy::my_strategy::simulator::Simulator;
use my_strategy::my_strategy::entity::Entity;
use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
use my_strategy::examples::example_world;

#[test]
fn test_simulator_tick_robot_jump() {
    let world = example_world();
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        0,
        0,
    ]);
    simulator.me_mut().action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.2931412499999937, -17.463246216636257)
    );
}

#[test]
fn test_simulator_tick_robot_jump_with_half_micro_ticks() {
    let world = example_world();
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        0,
        0,
    ]);
    simulator.me_mut().action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.2931412499999937, -17.463246216636257)
    );
}

#[test]
fn test_simulator_robot_jump() {
    let world = example_world();
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        0,
        0,
    ]);
    simulator.me_mut().action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK / 2,
        &mut rng
    );
    simulator.me_mut().action.jump_speed = 0.0;
    while simulator.me().position().y() > 1.0 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng
        );
    }
    assert_eq!(
        simulator.current_time(),
        1.016666666666668
    );
}

#[test]
fn test_simulator_robot_jump_with_half_micro_ticks() {
    let world = example_world();
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        0,
        0,
    ]);
    simulator.me_mut().action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK / 2,
        &mut rng
    );
    simulator.me_mut().action.jump_speed = 0.0;
    while simulator.me().position().y() > 1.0 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK / 2,
            &mut rng
        );
    }
    assert_eq!(
        simulator.current_time(),
        1.016666666666668
    );
}
