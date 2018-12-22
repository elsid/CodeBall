use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::tests::world::default_arena;

#[test]
fn test_distance_and_normal_to_arena() {
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, 0.0, 0.0)),
        (0.0, Vec3::new(0.0, 1.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, 10.0, 0.0)),
        (10.0, Vec3::new(0.0, 1.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, -10.0, 0.0)),
        (-10.0, Vec3::new(0.0, 1.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(100.0, 100.0, 0.0)),
        (-109.18089343777659, Vec3::new(-0.6627595788049191, -0.7488322513769865, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(100.0, 10.0, 0.0)),
        (-70.0, Vec3::new(-1.0, 0.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, 10.0, 100.0)),
        (-50.08483775994799, Vec3::new(0.0, -0.056513312022655776, -0.9984018457335854))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(29.0, 10.0, 0.0)),
        (1.0, Vec3::new(-1.0, 0.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(-100.0, 100.0, 0.0)),
        (-109.18089343777659, Vec3::new(0.6627595788049191, -0.7488322513769865, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, 100.0, -100.0)),
        (-104.0420478129973, Vec3::new(0.0, -0.8688174591210289, 0.49513253046682293))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(-100.0, 10.0, 0.0)),
        (-70.0, Vec3::new(1.0, 0.0, 0.0))
    );
    assert_eq!(
        default_arena()
            .distance_and_normal_to_arena(Vec3::new(0.0, 10.0, -100.0)),
        (-50.08483775994799, Vec3::new(0.0, -0.056513312022655776, 0.9984018457335854))
    );
}
