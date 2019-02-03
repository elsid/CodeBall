extern crate serde;

#[macro_use]
extern crate serde_derive;

#[allow(dead_code)]
mod model;

#[allow(dead_code)]
mod my_strategy;

#[allow(dead_code)]
mod strategy;

#[allow(dead_code)]
mod examples;

use crate::my_strategy::world::World;
use crate::my_strategy::vec3::Vec3;
use crate::my_strategy::random::XorShiftRng;
use crate::my_strategy::simulator::Simulator;
use crate::my_strategy::my_strategy_impl::MyStrategyImpl;

const DURATION: i32 = 150;
const SIMULATIONS: usize = 1000;

fn main() {
    match std::env::args().nth(1).unwrap().as_str() {
        "generate_empty" => generate_empty(),
        "check_goalkeeper" => check_goalkeeper(),
        _ => unimplemented!(),
    }
}

fn check_goalkeeper() {
    use std::io::BufRead;

    let world = examples::example_world(examples::GameType::OneRobotWithNitro);

    let stdin = std::io::stdin();
    let locked_stdin = stdin.lock();
    for line in locked_stdin.lines() {
        let line = line.unwrap();
        let mut simulation: Simulation = serde_json::from_str(&line).unwrap();

        simulation.goalkeeper = Some(simulate_goalkeeper(
            simulation.parameters.ball_position,
            simulation.parameters.ball_velocity * simulation.parameters.speed,
            world.clone()
        ));

        println!("{}", serde_json::to_string(&simulation).unwrap());
    }
}

fn simulate_goalkeeper(ball_position: Vec3, ball_velocity: Vec3, mut world: World) -> Result {
    use crate::examples::example_rng;

    world.me.set_position(world.rules.get_goalkeeper_position());
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(ball_position);
    world.game.ball.set_velocity(ball_velocity);

    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 1);
    let mut my_strategy = MyStrategyImpl::new(
        &simulator.me().base(),
        &simulator.rules(),
        &simulator.game(),
    );

    simulate_while(Some(&mut my_strategy), &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < DURATION
    });

    Result {
        score: simulator.score(),
        tick: simulator.current_tick(),
    }
}

fn generate_empty() {
    use crate::my_strategy::random::SeedableRng;

    let simulations: usize = std::env::args().nth(2).map(|v| v.parse().unwrap()).unwrap_or(SIMULATIONS);

    eprintln!("simulations: {}", simulations);

    let seed: u32 = std::env::args().nth(3).map(|v| v.parse().unwrap()).unwrap_or(4170596740);

    eprintln!("seed: {}", seed);

    let world = examples::example_world(examples::GameType::OneRobotWithNitro);

    let bounds: Bounds = default_bounds(&world);

    let mut rng = XorShiftRng::from_seed([seed, 943075939, 3311701793, 474463886]);

    for id in 0..simulations {
        let parameters = bounds.generate(&mut rng);

        let goal = simulate_empty(
            parameters.ball_position,
            parameters.ball_velocity * parameters.speed,
            world.clone()
        );

        let simulation = Simulation { id, parameters, empty: goal, goalkeeper: None };
        println!("{}", serde_json::to_string(&simulation).unwrap());
    }
}

fn default_bounds(world: &World) -> Bounds {
    Bounds {
        ball_position: Vec3Bounds {
            x: Range {
                min: -world.rules.arena.width / 2.0 + world.rules.BALL_RADIUS,
                max: world.rules.arena.width / 2.0 - world.rules.BALL_RADIUS,
            },
            y: Range {
                min: world.rules.BALL_RADIUS,
                max: world.rules.arena.height - world.rules.BALL_RADIUS,
            },
            z: Range {
                min: -world.rules.arena.depth / 2.0 + world.rules.BALL_RADIUS,
                max: world.rules.arena.depth / 2.0 - world.rules.BALL_RADIUS,
            },
        },
        ball_velocity: Vec3Bounds {
            x: Range { min: -1.0, max: 1.0 },
            y: Range { min: -1.0, max: 1.0 },
            z: Range { min: -1.0, max: 0.0 },
        },
        speed: Range { min: 0.0, max: world.rules.MAX_ENTITY_SPEED },
    }
}

struct Bounds {
    pub ball_position: Vec3Bounds,
    pub ball_velocity: Vec3Bounds,
    pub speed: Range,
}

impl Bounds {
    pub fn generate(&self, rng: &mut XorShiftRng) -> Parameters {
        Parameters {
            ball_position: self.ball_position.generate(rng),
            ball_velocity: self.ball_velocity.generate(rng).normalized(),
            speed: self.speed.generate(rng),
        }
    }
}

struct Vec3Bounds {
    pub x: Range,
    pub y: Range,
    pub z: Range,
}

impl Vec3Bounds {
    pub fn generate(&self, rng: &mut XorShiftRng) -> Vec3 {
        Vec3::new(self.x.generate(rng), self.y.generate(rng), self.z.generate(rng))
    }
}

struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    pub fn generate(&self, rng: &mut XorShiftRng) -> f64 {
        use crate::my_strategy::random::Rng;

        rng.gen_range(self.min, self.max)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Simulation {
    pub id: usize,
    pub parameters: Parameters,
    pub empty: Result,
    pub goalkeeper: Option<Result>,
}

#[derive(Serialize, Deserialize)]
pub struct Parameters {
    pub ball_position: Vec3,
    pub ball_velocity: Vec3,
    pub speed: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Result {
    score: i32,
    tick: i32,
}

fn simulate_empty(ball_position: Vec3, ball_velocity: Vec3, mut world: World) -> Result {
    use crate::examples::example_rng;
    use crate::my_strategy::simulator::Simulator;

    world.me.set_position(world.rules.get_goalkeeper_position());
    let me = world.me.clone();
    world.game.robots.iter_mut()
        .find(|v| v.id == me.id)
        .map(|v| *v = me.clone());
    world.game.ball.set_position(ball_position);
    world.game.ball.set_velocity(ball_velocity);

    let mut rng = example_rng(&world.rules);
    let mut simulator = Simulator::new(&world, 1);

    simulator.robots_mut().iter_mut()
        .for_each(|v| v.set_ignore(true));

    simulate_while(None, &mut simulator, &mut rng, |simulator| {
        simulator.score() == 0 && simulator.current_tick() < DURATION
    });

    Result {
        score: simulator.score(),
        tick: simulator.current_tick(),
    }
}

pub fn simulate_while<P>(mut my_strategy: Option<&mut MyStrategyImpl>, simulator: &mut Simulator,
                         rng: &mut XorShiftRng, predicate: P)
    where P: Fn(&mut Simulator) -> bool {
    use crate::model::Action;
    use crate::strategy::Strategy;

    while predicate(simulator) {
        if let Some(my_strategy) = &mut my_strategy {
            let mut action = Action::default();
            my_strategy.act(simulator.me().base(), simulator.rules(), &simulator.game(), &mut action);
            *simulator.me_mut().action_mut() = action;
        }
        simulator.tick(
            simulator.rules().tick_time_interval(),
            simulator.rules().MICROTICKS_PER_TICK,
            rng,
        );
    }
}