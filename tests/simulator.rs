use my_strategy::my_strategy::vec3::Vec3;
use my_strategy::my_strategy::simulator::{Simulator, RobotCollisionType, RobotExt, BallExt};
use my_strategy::my_strategy::entity::Entity;
use my_strategy::my_strategy::common::IsBetween;
use my_strategy::examples::{GameType, example_world, example_rules, example_me, example_ball, example_rng};

#[test]
fn test_simulator_tick_robot_walk() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    let max_speed = simulator.rules().ROBOT_MAX_GROUND_SPEED;
    simulator.me_mut().action_mut().set_target_velocity(Vec3::only_z(max_speed));
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.449218438858473)
    );
    assert_eq!(simulator.me().touch_normal(), Some(Vec3::new(0.0, 1.0, 0.0)));
}

#[test]
fn test_simulator_tick_robot_jump() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.2931412499999937, -17.463246216636257)
    );
    assert_eq!(simulator.me().touch_normal(), None);
}

#[test]
fn test_simulator_tick_robot_jump_with_half_micro_ticks() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.2931412499999937, -17.463246216636257)
    );
}

#[test]
fn test_simulator_robot_jump() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK / 2,
        &mut rng,
    );
    simulator.me_mut().action_mut().jump_speed = 0.0;
    while simulator.me().position().y() > 1.0 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(
        simulator.current_time(),
        1.016666666666668
    );
}

#[test]
fn test_simulator_robot_jump_with_half_micro_ticks() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 1.0, -17.463246216636257)
    );
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK / 2,
        &mut rng,
    );
    simulator.me_mut().action_mut().jump_speed = 0.0;
    while simulator.me().position().y() > 1.0 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK / 2,
            &mut rng,
        );
    }
    assert_eq!(
        simulator.current_time(),
        1.016666666666668
    );
}

#[test]
fn test_simulator_robot_kick_ball() {
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    let world = {
        let mut world = example_world(GameType::TwoRobots);
        world.game.ball.y = world.rules.BALL_RADIUS + 0.4;
        world.game.ball.set_velocity(Vec3::new(0.0, -5.0, -5.0));
        let distance = get_min_distance_between_spheres(
            world.game.ball.y,
            world.rules.BALL_RADIUS,
            world.rules.ROBOT_MIN_RADIUS,
        );
        assert!(distance.is_some());
        let robot_position = world.game.ball.position().with_y(1.0)
            - Vec3::new(0.0, 0.0, distance.unwrap() + 1e-3);
        world.game.robots[0].set_position(robot_position);
        world.game.robots[0].set_velocity(Vec3::new(0.0, 0.0, world.rules.ROBOT_MAX_GROUND_SPEED));
        world
    };
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(simulator.ball().position(), Vec3::new(0.0, 2.694228217927261, 0.6398122069549653));
    assert_eq!(simulator.ball().velocity(), Vec3::new(0.0, 16.18748483167559, 36.05203378168932));
    assert_eq!(simulator.me().collision_type(), RobotCollisionType::KickBall);
    while simulator.ball().position().y().is_between(
        world.rules.BALL_RADIUS + 0.1,
        world.rules.arena.goal_height - world.rules.BALL_RADIUS - 1e-2,
    ) && simulator.ball().position().z() < world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS
        {
            simulator.tick(
                simulator.rules().tick_time_interval(),
                simulator.rules().MICROTICKS_PER_TICK,
                &mut rng,
            );
        }
    assert_eq!(simulator.current_time(), 1.1333333333333342);
    assert_eq!(simulator.ball().position(), Vec3::new(0.0, 2.066086279964874, 40.89791659650675));
    assert_eq!(simulator.ball().velocity(), Vec3::new(0.0, -17.312515168326318, 36.05203378168932));
}

#[test]
fn test_simulator_wait_for_goal() {
    let world = {
        let mut world = example_world(GameType::TwoRobots);
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
    let mut rng = example_rng(&world.rules);
    for _ in 0..37 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(3.617314710299518, 6.5360488907168355, 42.16899575426824));
    assert_eq!(simulator.score(), 1);
}

#[test]
fn test_simulator_throw_ball_by_plus_x_and_z() {
    let world = {
        let mut world = example_world(GameType::TwoRobots);
        world.game.ball.velocity_x = world.rules.MAX_ENTITY_SPEED / 2.0;
        world.game.ball.velocity_y = 0.0;
        world.game.ball.velocity_z = world.rules.MAX_ENTITY_SPEED / 2.0;
        world
    };
    let mut simulator = Simulator::new(&world, world.game.robots[0].id);
    let mut rng = example_rng(&world.rules);
    ;
    for _ in 0..10 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(8.333333333333501, 7.420661866399502, 8.333333333333501));
    for _ in 0..90 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(-23.451917024130786, 3.123213400000252, 18.627066409350064));
}

#[test]
fn test_simulator_throw_ball_by_plus_x_and_neg_z() {
    let world = {
        let mut world = example_world(GameType::TwoRobots);
        world.game.ball.velocity_x = world.rules.MAX_ENTITY_SPEED / 2.0;
        world.game.ball.velocity_y = 0.0;
        world.game.ball.velocity_z = -world.rules.MAX_ENTITY_SPEED / 2.0;
        world
    };
    let mut simulator = Simulator::new(&world, world.game.robots[0].id);
    let mut rng = example_rng(&world.rules);
    ;
    for _ in 0..10 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(8.333333333333501, 7.420661866399502, -8.333333333333501));
    for _ in 0..90 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(-23.451917024130786, 3.123213400000252, -18.627066409350064));
}

#[test]
fn test_simulator_throw_ball_by_neg_x_and_plus_z() {
    let world = {
        let mut world = example_world(GameType::TwoRobots);
        world.game.ball.velocity_x = -world.rules.MAX_ENTITY_SPEED / 2.0;
        world.game.ball.velocity_y = 0.0;
        world.game.ball.velocity_z = world.rules.MAX_ENTITY_SPEED / 2.0;
        world
    };
    let mut simulator = Simulator::new(&world, world.game.robots[0].id);
    let mut rng = example_rng(&world.rules);
    for _ in 0..10 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(-8.333333333333501, 7.420661866399502, 8.333333333333501));
    for _ in 0..90 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(23.451917024130786, 3.123213400000252, 18.627066409350064));
}

#[test]
fn test_simulator_throw_ball_by_neg_x_and_z() {
    let world = {
        let mut world = example_world(GameType::TwoRobots);
        world.game.ball.velocity_x = -world.rules.MAX_ENTITY_SPEED / 2.0;
        world.game.ball.velocity_y = 0.0;
        world.game.ball.velocity_z = -world.rules.MAX_ENTITY_SPEED / 2.0;
        world
    };
    let mut simulator = Simulator::new(&world, world.game.robots[0].id);
    let mut rng = example_rng(&world.rules);
    ;
    for _ in 0..10 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(-8.333333333333501, 7.420661866399502, -8.333333333333501));
    for _ in 0..90 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.ball().position(), Vec3::new(23.451917024130786, 3.123213400000252, -18.627066409350064));
}

#[test]
fn test_simulator_collide_jumping_robot_and_ball() {
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    let rules = example_rules(GameType::TwoRobots);
    let mut me = RobotExt::from_robot(&example_me(GameType::TwoRobots, &rules), &rules);
    let mut ball = BallExt::from_ball(&example_ball(&rules), &rules);
    ball.set_position(Vec3::new(0.0, rules.BALL_RADIUS, 0.0));
    let distance = get_min_distance_between_spheres(
        ball.position().y(),
        rules.BALL_RADIUS,
        rules.ROBOT_MIN_RADIUS,
    ).unwrap();
    me.set_position(ball.position().with_y(rules.ROBOT_MIN_RADIUS) - Vec3::only_z(distance));
    me.jump(rules.ROBOT_MAX_JUMP_SPEED, &rules);
    Simulator::collide(|| rules.mean_e(), &mut me, &mut ball);
    assert_eq!(ball.velocity().norm(), 14.499999999999998);
}

#[test]
fn test_simulator_collide_jumping_and_moving_robot_and_ball() {
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    let rules = example_rules(GameType::TwoRobots);
    let mut me = RobotExt::from_robot(&example_me(GameType::TwoRobots, &rules), &rules);
    let mut ball = BallExt::from_ball(&example_ball(&rules), &rules);
    ball.set_position(Vec3::new(0.0, rules.BALL_RADIUS, 0.0));
    let distance = get_min_distance_between_spheres(
        ball.position().y(),
        rules.BALL_RADIUS,
        rules.ROBOT_MIN_RADIUS,
    ).unwrap();
    me.set_position(ball.position().with_y(rules.ROBOT_MIN_RADIUS) - Vec3::only_z(distance));
    me.set_velocity(Vec3::new(0.0, 0.0, rules.ROBOT_MAX_GROUND_SPEED));
    me.jump(rules.ROBOT_MAX_JUMP_SPEED, &rules);
    Simulator::collide(|| rules.mean_e(), &mut me, &mut ball);
    assert_eq!(ball.velocity().norm(), 41.84146220587983);
}

#[test]
fn test_simulator_tick_ball_to_goal() {
    let mut world = example_world(GameType::TwoRobots);
    world.game.ball.set_position(Vec3::new(0.0, 2.6720979333335597, 40.50000000000227));
    world.game.ball.set_velocity(Vec3::new(0.0, -1.7128000000001131, 30.0));
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(
        simulator.ball().position(),
        Vec3::new(0.0, 2.6393846000002408, 41.00000000000252)
    );
}

#[test]
fn test_simulator_robot_turn_left() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let target = Vec3::new(3.0, 1.0, 0.0);
    simulator.me_mut().set_position(Vec3::new(10.0, 1.0, 0.0));
    simulator.me_mut().set_velocity(Vec3::only_z(30.0));
    let mut rng = example_rng(&world.rules);
    while simulator.me().position().distance(target) > 0.5 {
        let position = simulator.me().position();
        simulator.me_mut().action_mut().set_target_velocity(
            (target - position).normalized() * world.rules.ROBOT_MAX_GROUND_SPEED
        );
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.current_tick(), 40);
}

#[test]
fn test_simulator_robot_turn_back() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let target = Vec3::new(10.0, 1.0, -7.0);
    simulator.me_mut().set_position(Vec3::new(10.0, 1.0, 0.0));
    simulator.me_mut().set_velocity(Vec3::only_z(30.0));
    let mut rng = example_rng(&world.rules);
    while simulator.me().position().distance(target) > 0.5 {
        let position = simulator.me().position();
        simulator.me_mut().action_mut().set_target_velocity(
            (target - position).normalized() * world.rules.ROBOT_MAX_GROUND_SPEED
        );
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.current_tick(), 49);
}

#[test]
fn test_simulator_robot_walk_on_wall() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    let mut prev_y = simulator.me().position().y();
    let mut max_y = simulator.me().position().y();
    while prev_y - simulator.me().position().y() <= 1e-3 {
        if simulator.me().velocity().norm() < 1.0 {
            simulator.me_mut().action_mut()
                .set_target_velocity(Vec3::only_x(world.rules.ROBOT_MAX_GROUND_SPEED));
        } else {
            let v = world.rules.arena
                .projected_at(simulator.me().position(), simulator.me().velocity())
                .normalized() * world.rules.ROBOT_MAX_GROUND_SPEED;
            simulator.me_mut().action_mut().set_target_velocity(v);
        }
        prev_y = simulator.me().position().y();
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
        max_y = max_y.max(simulator.me().position().y());
    }
    assert_eq!(max_y, 16.67704416378882);
    assert_eq!(
        simulator.me().position(),
        Vec3::new(27.271399780469423, 16.66802419546084, -17.463246216636257)
    );
    assert_eq!(
        simulator.me().normal_to_arena(),
        Vec3::new(-0.7586573602400657, -0.6514898385650963, -0.0)
    );
    assert_eq!(simulator.me().distance_to_arena(), 1.369790627065421);
}

#[test]
fn test_simulator_tick_robot_jump_using_nitro_with_nitro() {
    let world = example_world(GameType::TwoRobotsWithNitro);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.me_mut().action_mut().target_velocity_y = simulator.rules().MAX_ENTITY_SPEED;
    simulator.me_mut().action_mut().use_nitro = true;
    for _ in 0..76 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 18.924958333333343, -17.463246216636257)
    );
    assert_eq!(simulator.me().nitro_amount(), 0.0);
}

#[test]
fn test_simulator_tick_robot_jump_using_nitro_without_nitro() {
    let world = example_world(GameType::TwoRobots);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.me_mut().action_mut().jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.me_mut().action_mut().target_velocity_y = simulator.rules().MAX_ENTITY_SPEED;
    simulator.me_mut().action_mut().use_nitro = true;
    for _ in 0..76 {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(
        simulator.me().position(),
        Vec3::new(9.748591261158683, 3.984697916666674, -17.463246216636257)
    );
    assert_eq!(simulator.me().nitro_amount(), 0.0);
}

#[test]
fn test_simulator_tick_robot_walk_to_nitro_pack() {
    let world = example_world(GameType::TwoRobotsWithNitro);
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    let nearest_nitro_pack = simulator.nitro_packs().iter()
        .map(|v| (v.position().distance(simulator.me().position()).round() as i32, v))
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, v)| v.clone())
        .unwrap();
    let target_velocity = (nearest_nitro_pack.position() - simulator.me().position())
        .normalized() * world.rules.ROBOT_MAX_GROUND_SPEED;
    simulator.me_mut().action_mut().set_target_velocity(target_velocity);
    while nearest_nitro_pack.position().distance(simulator.me().position())
        > simulator.me().velocity().norm() * simulator.rules().tick_time_interval() {
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            &mut rng,
        );
    }
    assert_eq!(simulator.me().nitro_amount(), world.rules.MAX_NITRO_AMOUNT);
    let nitro_pack = simulator.nitro_packs().iter()
        .find(|v| v.id == nearest_nitro_pack.id).unwrap();
    assert_eq!(nitro_pack.respawn_ticks, Some(597));
}

#[test]
fn test_simulator_ball_hit_bottom_corner() {
    let mut world = example_world(GameType::TwoRobots);
    world.game.ball.set_position(Vec3::new(-24.4378654601576, 2.462833999159444, 34.82362575766481));
    world.game.ball.set_velocity(Vec3::new(6.239753468929283, 2.5046756681021467, 43.74873027140119));
    let mut simulator = Simulator::new(&world, world.me.id);
    let mut rng = example_rng(&world.rules);
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng,
    );
    assert_eq!(
        simulator.ball().position(),
        Vec3::new(-24.0123078567098, 2.79461722629136, 35.210174346110826)
    );
    assert_eq!(
        simulator.ball().velocity(),
        Vec3::new(28.051208402063356, 21.960551576178723, 20.51046243838915)
    );
}
