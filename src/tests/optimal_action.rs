use crate::my_strategy::optimal_action::get_min_distance_between_spheres;

#[test]
fn test_get_min_distance_between_spheres() {
    assert_eq!(get_min_distance_between_spheres(2.0, 2.0, 1.0), Some(2.8284271247461903));
    assert_eq!(get_min_distance_between_spheres(3.0, 2.0, 1.0), Some(2.23606797749979));
    assert_eq!(get_min_distance_between_spheres(4.0, 2.0, 1.0), Some(0.0));
    assert_eq!(get_min_distance_between_spheres(5.0, 2.0, 1.0), None);
}
