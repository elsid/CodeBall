#[test]
fn test_new() {
    use my_strategy::examples::{example_world, example_rng};
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::orders::Order;

    let world = example_world();
    let mut rng = example_rng();
    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(-15.381757714697283, 0.0, 25.75658225786858));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1128);
}

#[test]
fn test_new_should_not_jump_on_ball_top() {
    use my_strategy::examples::example_world;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::Order;

    let mut world = example_world();
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);

    world.me.id = 2;
    world.me.set_position(Vec3::new(-5.838617159216834, 1.0, -10.508900380791133));
    world.me.set_velocity(Vec3::new(16.91322406429886, 0.0, 24.77759529887104));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 5.233161866399729;
    world.game.ball.velocity_y = -12.500000000000554;

    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(17.249527726527795, 0.0, 24.54493416596893));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1355);
}

#[test]
fn test_new_far_jump() {
    use my_strategy::examples::example_world;
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
    use my_strategy::my_strategy::orders::Order;

    let mut world = example_world();
    let mut rng = XorShiftRng::from_seed([
        1662648909,
        2447818268,
        201539282,
        3684906436,
    ]);

    world.me.id = 2;
    world.me.set_position(Vec3::new(0.0, 1.0, -5.0));
    world.me.set_velocity(Vec3::new(0.0, 0.0, 30.0));
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == 2)
        .map(|v| *v = me);
    world.game.ball.y = 6.0;
    world.game.ball.velocity_y = 0.0;

    let result = Order::new(&world.me, &world, &mut rng).unwrap();

    assert_eq!(result.action.target_velocity(), Vec3::new(0.0, 0.0, 30.0));
    assert_eq!(result.action.jump_speed, 15.0);
    assert_eq!(result.score, 1171);
}
