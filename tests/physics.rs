#[test]
fn test_get_min_distance_between_spheres() {
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    assert_eq!(get_min_distance_between_spheres(2.0, 2.0, 1.0), Some(2.8284271247461903));
    assert_eq!(get_min_distance_between_spheres(3.0, 2.0, 1.0), Some(2.23606797749979));
    assert_eq!(get_min_distance_between_spheres(4.0, 2.0, 1.0), Some(0.0));
    assert_eq!(get_min_distance_between_spheres(5.0, 2.0, 1.0), None);
}

#[test]
fn test_move_equation_get_position() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::MoveEquation;

    let move_equation = MoveEquation {
        initial_velocity: Vec3::new(1.0, 1.0, 0.0),
        initial_position: Vec3::new(0.0, 1.0, 0.0),
        acceleration: Vec3::new(0.0, -1.0, 0.0),
    };

    assert_eq!(move_equation.get_position(0.0), Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(move_equation.get_position(1.0), Vec3::new(1.0, 1.5, 0.0));
    assert_eq!(move_equation.get_position(2.0), Vec3::new(2.0, 1.0, 0.0));
    assert_eq!(move_equation.get_position(3.0), Vec3::new(3.0, -0.5, 0.0));
    assert_eq!(
        move_equation.get_position(1.9940153118966086)
            .distance(Vec3::new(2.0, 1.0, 0.0)),
        0.008450973527545045
    );
    assert_eq!(
        move_equation.get_position(1.610095947436655)
            .distance(Vec3::new(2.0, 1.0, 0.0)),
        0.5005527180526962
    );
}

#[test]
fn test_move_equation_get_time_to_target() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::MoveEquation;

    let move_equation = MoveEquation {
        initial_velocity: Vec3::new(1.0, 1.0, 0.0),
        initial_position: Vec3::new(0.0, 1.0, 0.0),
        acceleration: Vec3::new(0.0, -1.0, 0.0),
    };

    assert_eq!(
        move_equation.get_time_to_target(
            Vec3::new(2.0, 1.0, 0.0),
            1.0,
            3.0,
            0.0,
            10,
        ),
        1.9940153118966086
    );
    assert_eq!(
        move_equation.get_time_to_target(
            Vec3::new(2.0, 1.0, 0.0),
            1.0,
            1.9940153118966086,
            0.5,
            10,
        ),
        1.610095947436655
    );
    assert_eq!(
        move_equation.get_time_to_target(
            Vec3::new(3.0, 1.0, 0.0),
            1.0,
            3.0,
            0.0,
            10,
        ),
        2.009090308118533
    );
}

#[test]
fn test_move_equation_get_closest_possible_distance_to_target() {
    use my_strategy::my_strategy::vec3::Vec3;
    use my_strategy::my_strategy::physics::MoveEquation;

    let move_equation = MoveEquation {
        initial_velocity: Vec3::new(1.0, 1.0, 0.0),
        initial_position: Vec3::new(0.0, 1.0, 0.0),
        acceleration: Vec3::new(0.0, -1.0, 0.0),
    };

    assert_eq!(
        move_equation.get_closest_possible_distance_to_target(
            Vec3::new(2.0, 1.0, 0.0),
            1.0,
            3.0,
            10,
        ),
        0.008450973527545045
    );
    assert_eq!(
        move_equation.get_closest_possible_distance_to_target(
            Vec3::new(3.0, 1.0, 0.0),
            1.0,
            3.0,
            10,
        ),
        0.9909517667571945
    );
}
