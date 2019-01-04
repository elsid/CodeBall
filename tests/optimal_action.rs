#[test]
fn test_robot_get_optimal_action() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::render::Render;
    use my_strategy::examples::{example_world, example_rng};

    let world = example_world();
    let mut rng = example_rng();
    let mut render = Render::new();
    let result = world.me.get_optimal_action(&world, &mut rng, &mut render);

    assert_eq!(result.action.target_velocity(), Vec3::new(-15.112130922109685, 0.0749066624159364, 25.915591600134114));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1079);
}

#[test]
fn test_robot_get_optimal_action_should_not_jump_on_ball_top() {
    use my_strategy::my_strategy::world::World;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::render::Render;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::examples::example_world;

    let mut world = example_world();
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);
    let mut render = Render::new();

    world.me.id = 2;
    world.me.set_position(Vec3::new(-5.838617159216834, 1.0, -10.508900380791133));
    world.me.set_velocity(Vec3::new(16.91322406429886, 0.0, 24.77759529887104));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 5.233161866399729;
    world.game.ball.velocity_y = -12.500000000000554;

    let result = world.me.get_optimal_action(&world, &mut rng, &mut render);

    assert_eq!(result.action.target_velocity(), Vec3::new(15.205946498380044, 0.0904449080449424, 25.860607317056818));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1245);
}

#[test]
fn test_robot_get_optimal_action_far_jump() {
    use my_strategy::my_strategy::world::World;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::render::Render;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::examples::example_world;

    let mut world = example_world();
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);
    let mut render = Render::new();

    world.me.id = 2;
    world.me.set_position(Vec3::new(0.0, 1.0, -5.0));
    world.me.set_velocity(Vec3::new(0.0, 0.0, 30.0));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 6.0;
    world.game.ball.velocity_y = 0.0;

    let result = world.me.get_optimal_action(&world, &mut rng, &mut render);

    assert_eq!(result.action.target_velocity(), Vec3::new(0.0, 0.0, 30.0));
    assert_eq!(result.action.jump_speed, 15.0);
    assert_eq!(result.score, 1161);
}
