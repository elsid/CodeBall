#[test]
fn test_get_min_distance_between_spheres() {
    use my_strategy::my_strategy::physics::get_min_distance_between_spheres;

    assert_eq!(get_min_distance_between_spheres(2.0, 2.0, 1.0), Some(2.8284271247461903));
    assert_eq!(get_min_distance_between_spheres(3.0, 2.0, 1.0), Some(2.23606797749979));
    assert_eq!(get_min_distance_between_spheres(4.0, 2.0, 1.0), Some(0.0));
    assert_eq!(get_min_distance_between_spheres(5.0, 2.0, 1.0), None);
}

#[test]
fn test_move_equation_get_time_at_y() {
    use my_strategy::my_strategy::physics::MoveEquation;
    use my_strategy::my_strategy::vec3::Vec3;

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(0.0),
            initial_velocity: Vec3::only_y(0.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![0.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(1.0),
            initial_velocity: Vec3::only_y(0.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        Vec::<f64>::new()
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(0.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        Vec::<f64>::new()
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(0.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![0.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(0.0),
            initial_velocity: Vec3::only_y(-1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![0.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![-1.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![1.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(1.0),
            initial_velocity: Vec3::only_y(-1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![1.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(-1.0),
            acceleration: Vec3::only_y(0.0),
        }.get_time_at_y(0.0),
        vec![-1.0]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(1.0),
        }.get_time_at_y(0.0),
        Vec::<f64>::new()
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(1.0),
        }.get_time_at_y(0.0),
        vec![0.7320508075688772, 2.732050807568877]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(-1.0),
        }.get_time_at_y(0.0),
        Vec::<f64>::new()
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(-1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(1.0),
        }.get_time_at_y(0.0),
        vec![0.7320508075688772, 2.732050807568877]
    );

    assert_eq!(
        MoveEquation {
            initial_position: Vec3::only_y(1.0),
            initial_velocity: Vec3::only_y(1.0),
            acceleration: Vec3::only_y(-1.0),
        }.get_time_at_y(0.0),
        vec![2.732050807568877]
    );
}
