#[test]
fn test_rotation() {
    use my_strategy::my_strategy::mat3::Mat3;
    use my_strategy::my_strategy::vec3::Vec3;

    assert_eq!(
        Mat3::rotation(Vec3::i(), std::f64::consts::PI / 2.0) * Vec3::i(),
        Vec3::i()
    );
    assert_eq!(
        Mat3::rotation(Vec3::j(), std::f64::consts::PI / 2.0) * Vec3::j(),
        Vec3::j()
    );
    assert_eq!(
        Mat3::rotation(Vec3::k(), std::f64::consts::PI / 2.0) * Vec3::k(),
        Vec3::k()
    );

    assert_eq!(
        Mat3::rotation(Vec3::i(), std::f64::consts::PI / 2.0) * Vec3::j(),
        Vec3::new(0.0, 0.00000000000000006123233995736766, -1.0)
    );
    assert_eq!(
        Mat3::rotation(Vec3::i(), std::f64::consts::PI / 2.0) * Vec3::k(),
        Vec3::new(0.0, 1.0, 0.00000000000000006123233995736766)
    );

    assert_eq!(
        Mat3::rotation(Vec3::j(), std::f64::consts::PI / 2.0) * Vec3::i(),
        Vec3::new(0.00000000000000006123233995736766, 0.0, 1.0)
    );
    assert_eq!(
        Mat3::rotation(Vec3::j(), std::f64::consts::PI / 2.0) * Vec3::k(),
        Vec3::new(-1.0, 0.0, 0.00000000000000006123233995736766)
    );

    assert_eq!(
        Mat3::rotation(Vec3::k(), std::f64::consts::PI / 2.0) * Vec3::i(),
        Vec3::new(0.00000000000000006123233995736766, -1.0, 0.0)
    );
    assert_eq!(
        Mat3::rotation(Vec3::k(), std::f64::consts::PI / 2.0) * Vec3::i(),
        Vec3::new(0.00000000000000006123233995736766, -1.0, 0.0)
    );
}
