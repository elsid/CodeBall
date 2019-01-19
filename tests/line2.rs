#[test]
fn test_possible_intersection() {
    use my_strategy::my_strategy::line2::Line2;
    use my_strategy::my_strategy::vec2::Vec2;

    assert_eq!(
        Line2::new(Vec2::new(0.0, 2.0), Vec2::new(1.0, 2.0))
            .possible_intersection(&Line2::new(Vec2::new(-7.0, -2.0), Vec2::new(4.0, 3.0))),
        Some(Vec2::new(1.8, 2.0))
    );
}
