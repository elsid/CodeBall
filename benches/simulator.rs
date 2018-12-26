extern crate my_strategy;

use criterion::{Criterion, criterion_group, criterion_main};
use my_strategy::my_strategy::random::{XorShiftRng, SeedableRng};
use my_strategy::my_strategy::simulator::Simulator;
use my_strategy::examples::example_world;

fn simulator_tick(c: &mut Criterion) {
    c.bench_function("simulator_tick", |b| {
        let mut simulator = Simulator::new(&example_world(), 1);
        let time_interval = simulator.rules().tick_time_interval();
        let micro_ticks_per_tick = simulator.rules().MICROTICKS_PER_TICK;
        let mut rng = XorShiftRng::from_seed([
            simulator.rules().seed as u32,
            (simulator.rules().seed >> 32) as u32,
            0,
            0,
        ]);
        b.iter(move || {
            simulator.tick(time_interval, micro_ticks_per_tick, &mut rng);
        })
    });
}

fn simulator_tick_with_half_micro_ticks(c: &mut Criterion) {
    c.bench_function("simulator_tick_with_half_micro_ticks", |b| {
        let mut simulator = Simulator::new(&example_world(), 1);
        let time_interval = simulator.rules().tick_time_interval();
        let micro_ticks_per_tick = simulator.rules().MICROTICKS_PER_TICK / 2;
        let mut rng = XorShiftRng::from_seed([
            simulator.rules().seed as u32,
            (simulator.rules().seed >> 32) as u32,
            0,
            0,
        ]);
        b.iter(move || {
            simulator.tick(time_interval, micro_ticks_per_tick, &mut rng);
        })
    });
}

criterion_group!(benches, simulator_tick, simulator_tick_with_half_micro_ticks);
criterion_main!(benches);
