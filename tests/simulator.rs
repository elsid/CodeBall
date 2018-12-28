use my_strategy::my_strategy::vec3::Vec3;
use my_strategy::my_strategy::simulator::{Simulator, CollisionType};
use my_strategy::my_strategy::entity::Entity;
use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
use my_strategy::my_strategy::optimal_action::get_min_distance_between_spheres;
use my_strategy::my_strategy::common::{IsBetween, Square};
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

#[test]
fn test_simulator_robot_kick_ball() {
    let world = {
        let mut world = example_world();
        world.game.ball.y = world.rules.BALL_RADIUS + 0.4;
        world.game.ball.set_velocity(Vec3::new(0.0, -5.0, -5.0));
        let distance = get_min_distance_between_spheres(
            world.game.ball.y,
            world.rules.BALL_RADIUS,
            world.rules.ROBOT_MIN_RADIUS
        );
        assert!(distance.is_some());
        let robot_position = world.game.ball.position().with_y(1.0)
            - Vec3::new(0.0, 0.0, distance.unwrap() + 1e-3);
        world.game.robots[0].set_position(robot_position);
        world.game.robots[0].set_velocity(Vec3::new(0.0, 0.0, world.rules.ROBOT_MAX_GROUND_SPEED));
        world
    };
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
    assert_eq!(simulator.ball().position(), Vec3::new(0.0, 2.685785194346652, 0.6238366102032489));
    assert_eq!(simulator.ball().velocity(), Vec3::new(0.0, 15.671092310398691, 35.07455928925282));
    assert_eq!(simulator.me().ball_collision_type(), CollisionType::Kick);
    while simulator.ball().position().y().is_between(
            world.rules.BALL_RADIUS + 0.1,
            world.rules.arena.goal_height - world.rules.BALL_RADIUS - 1e-2
        ) && simulator.ball().position().z() < world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS
    {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng
        );
    }
    assert_eq!(simulator.current_time(), 1.100000000000001);
    assert_eq!(simulator.ball().position(), Vec3::new(0.0, 2.0586351972784565, 38.62127584022837));
    assert_eq!(simulator.ball().velocity(), Vec3::new(0.0, -16.828907689603298, 35.07455928925282));
}

#[test]
fn test_simulator_wait_for_goal() {
    let world = {
        let mut world = example_world();
        world.game.ball.y = 7.584569693698086;
        world.game.ball.x = 2.354339378140074;
        world.game.ball.z = 27.7479348041067;
        world.game.ball.velocity_x = 2.048068106203642;
        world.game.ball.velocity_y = -27.116734448465703;
        world.game.ball.velocity_z = 24.13826180412662;
        world.rules.seed = 2793871283;
        world
    };
    let mut simulator = Simulator::new(&world, world.game.robots[0].id);
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        1841971383,
        1904458926,
    ]);
    for _ in 0..34 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(3.5149113049892926, 6.909288205242904, 46.22324077130158));
    assert_eq!(simulator.score(), 1);
}

#[test]
fn test_simulator_theoretic() {
    let mut world = example_world();
    let initial_position = Vec3::new(0.0, 2.685785194346652, 0.6238366102032489);
    let initial_velocity = Vec3::new(0.0, 15.671092310398691, 35.07455928925282);
    let final_position = Vec3::new(0.0, 2.0586351972784565, 38.62127584022837);
    let final_velocity = Vec3::new(0.0, -16.828907689603298, 35.07455928925282);
    let acceleration = Vec3::new(0.0, -world.rules.GRAVITY, 0.0);
    let time = 1.100000000000001;
    let initial_velocity_1 = (final_position - initial_position - acceleration * time.square() / 2.0) / time;
    let final_position_1 = initial_position + initial_velocity * time + acceleration * time.sqrt() / 2.0;
    let final_velocity_1 = initial_velocity + acceleration * time;
    let final_position_2 = initial_position + final_velocity * time - acceleration * time.sqrt() / 2.0;
    assert_eq!(initial_velocity_1, Vec3::new(0.0,  15.92986363902893, 34.54312657275008));
    assert_eq!(final_position_1, Vec3::new(0.0,  4.19185401323295, 39.205851828381384));
    assert_eq!(final_velocity_1, Vec3::new(0.0,  -17.328907689601337, 35.07455928925282));
    assert_eq!(final_position_2, Vec3::new(0.0, -0.09388054166470994, 39.205851828381384));
}
