#[test]
fn test_robot_get_optimal_action() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::render::Render;
    use my_strategy::examples::{example_world, example_rng};

    let world = example_world();
    let mut rng = example_rng();
    let mut render = Render::new();
    let result = world.me.get_optimal_action(&world, &mut rng, &mut render);

    assert_eq!(result.action.target_velocity(), Vec3::new(-15.112130922109685, 0.07490666241593641, 25.915591600134114));
    assert_eq!(result.action.jump_speed, 0.0);
    assert_eq!(result.score, 1075);
}
