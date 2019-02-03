extern crate my_strategy;

use criterion::{Criterion, criterion_group, criterion_main};

use my_strategy::my_strategy::simulator::Simulator;
use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;
use my_strategy::my_strategy::random::XorShiftRng;

fn two_robots_with_nitro(c: &mut Criterion) {
    use my_strategy::examples::{GameType, example_world, example_rng};
    use my_strategy::my_strategy::simulator::Simulator;
    use my_strategy::my_strategy::my_strategy_impl::MyStrategyImpl;

    c.bench_function("two_robots_with_nitro", |b| {
        let world = example_world(GameType::TwoRobotsWithNitro);
        let mut rng = example_rng(&world.rules);
        let mut simulator = Simulator::new(&world, 1);
        let mut strategies = world.game.players.iter()
            .map(|player| {
                (
                    player.id,
                    MyStrategyImpl::new(
                        simulator.robots().iter()
                            .find(|v| v.base().player_id == player.id)
                            .unwrap().base(),
                        &simulator.rules(),
                        &simulator.game(),
                    ),
                )
            })
            .collect::<Vec<_>>();
        b.iter(move || {
            tick(&mut strategies, &mut simulator, &mut rng);
        });
    });
}

fn tick(strategies: &mut Vec<(i32, MyStrategyImpl)>, simulator: &mut Simulator, rng: &mut XorShiftRng) {
    use my_strategy::model::Action;
    use my_strategy::strategy::Strategy;

    for (player_id, strategy) in strategies.iter_mut() {
        let actions = simulator.robots().iter()
            .filter(|v| v.base().player_id == *player_id)
            .map(|v| {
                let mut action = Action::default();
                strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
                (v.id(), action)
            })
            .collect::<Vec<_>>();
        for (id, action) in actions {
            simulator.robots_mut().iter_mut()
                .find(|v| v.id() == id)
                .map(|v| *v.action_mut() = action);
        }
    }
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        rng,
    );
}

criterion_group!(benches, two_robots_with_nitro);
criterion_main!(benches);
