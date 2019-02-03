extern crate my_strategy;

use criterion::{Criterion, criterion_group, criterion_main};
use my_strategy::my_strategy::simulator::BallExt;
use my_strategy::my_strategy::entity::Entity;
use my_strategy::my_strategy::vec3::Vec3;
use my_strategy::examples::{GameType, example_ball, example_arena, example_rules};

fn arena_collide(c: &mut Criterion) {
    c.bench_function("arena_collide", |b| {
        let arena = example_arena();
        let rules = example_rules(GameType::TwoRobots);
        let ball = example_ball(&rules);
        b.iter(move || {
            let mut ball = BallExt::new(
                ball.clone(),
                rules.BALL_MASS,
                rules.BALL_ARENA_E,
            );
            ball.set_position(Vec3::new(100.0, 100.0, 100.0));
            arena.collide(&mut ball);
        })
    });
}

criterion_group!(benches, arena_collide);
criterion_main!(benches);
