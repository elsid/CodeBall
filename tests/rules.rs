#[test]
fn test_time_for_distance() {
    use my_strategy::examples::example_rules;
    assert_eq!(
        example_rules().time_for_distance(0.0, 0.0),
        0.0,
    );

    assert_eq!(
        example_rules().time_for_distance(0.0, 1.0),
        0.1414213562373095,
    );
    assert_eq!(
        example_rules().time_for_distance(0.0, 2.0),
        0.2,
    );
    assert_eq!(
        example_rules().time_for_distance(0.0, 3.0),
        0.24494897427831783,
    );

    assert_eq!(
        example_rules().time_for_distance(1.0, 1.0),
        0.13177446878757826,
    );
    assert_eq!(
        example_rules().time_for_distance(1.0, 2.0),
        0.19024984394500785,
    );
    assert_eq!(
        example_rules().time_for_distance(1.0, 3.0),
        0.23515301344262524,
    );

    assert_eq!(
        example_rules().time_for_distance(-1.0, 1.0),
        0.16212670403551893,
    );
    assert_eq!(
        example_rules().time_for_distance(-1.0, 2.0),
        0.2204993765576342,
    );
    assert_eq!(
        example_rules().time_for_distance(-1.0, 3.0),
        0.2653568829277059,
    );
}