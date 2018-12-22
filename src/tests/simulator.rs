use crate::model::Action;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::entity::Entity;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::tests::world::default_world;

#[test]
fn test_simulator_robot_jump() {
    let mut simulator = Simulator::new(&default_world());
    let mut action = Action::default();
    let mut rng = XorShiftRng::from_seed([
        simulator.rules().seed as u32,
        (simulator.rules().seed >> 32) as u32,
        0,
        0,
    ]);
    action.jump_speed = simulator.rules().ROBOT_MAX_JUMP_SPEED;
    simulator.tick(
        simulator.rules().tick_time_interval(),
        simulator.rules().MICROTICKS_PER_TICK,
        &mut rng
    );
    assert_eq!(
        simulator.me().position(),
        Vec3::new(-0.289095425043726, 1.2931412499999937, -19.997910486728827)
    );
}
