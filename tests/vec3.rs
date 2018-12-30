use my_strategy::my_strategy::vec3::Vec3;

#[test]
fn test_clamp() {
    assert_eq!(
        Vec3::new(2.0, 0.0, 0.0).clamp(3.0),
        Vec3::new(2.0, 0.0, 0.0)
    );
    assert_eq!(
        Vec3::new(0.0, 2.0, 0.0).clamp(3.0),
        Vec3::new(0.0, 2.0, 0.0)
    );
    assert_eq!(
        Vec3::new(0.0, 0.0, 2.0).clamp(3.0),
        Vec3::new(0.0, 0.0, 2.0)
    );
    assert_eq!(
        Vec3::new(2.0, 0.0, 0.0).clamp(1.0),
        Vec3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
        Vec3::new(0.0, 2.0, 0.0).clamp(1.0),
        Vec3::new(0.0, 1.0, 0.0)
    );
    assert_eq!(
        Vec3::new(0.0, 0.0, 2.0).clamp(1.0),
        Vec3::new(0.0, 0.0, 1.0)
    );
    assert_eq!(
        Vec3::new(2.0, 2.0, 2.0).clamp(4.0).norm(),
        3.4641016151377544
    );
    assert_eq!(
        Vec3::new(2.0, 2.0, 2.0).clamp(1.0).norm(),
        1.0
    );
}
