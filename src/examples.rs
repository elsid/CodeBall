use crate::model::{Arena, Ball, Game, Player, Robot, Rules, NitroPack};
use crate::my_strategy::world::World;
use crate::my_strategy::random::{XorShiftRng, SeedableRng};
use crate::my_strategy::config::Config;

#[derive(Copy, Clone)]
pub enum GameType {
    TwoRobots,
    TwoRobotsWithNitro,
    ThreeRobotsWithNitro,
    OneRobotWithNitro,
}

impl GameType {
    pub fn nitro(self) -> bool {
        match self {
            GameType::TwoRobots => false,
            GameType::TwoRobotsWithNitro => true,
            GameType::ThreeRobotsWithNitro => true,
            GameType::OneRobotWithNitro => true,
        }
    }

    pub fn team_size(self) -> usize {
        match self {
            GameType::TwoRobots => 2,
            GameType::TwoRobotsWithNitro => 2,
            GameType::ThreeRobotsWithNitro => 3,
            GameType::OneRobotWithNitro => 1,
        }
    }
}

pub fn example_rng(rules: &Rules) -> XorShiftRng {
    XorShiftRng::from_seed([
        rules.seed as u32,
        (rules.seed >> 32) as u32,
        1841971383,
        1904458926,
    ])
}

pub fn example_world(game_type: GameType) -> World {
    let rules = example_rules(game_type);
    World::new(
        Config::default(),
        example_me(game_type, &rules),
        rules.clone(),
        example_game(game_type, &rules),
    )
}

pub fn example_game(game_type: GameType, rules: &Rules) -> Game {
    Game {
        current_tick: 0,
        players: vec![
            Player { id: 1, me: true, strategy_crashed: false, score: 0 },
            Player { id: 2, me: false, strategy_crashed: false, score: 0 },
        ],
        robots: if game_type.team_size() == 1 {
            vec![
                example_me(game_type, rules),
                example_opponent_1(game_type, rules),
            ]
        } else if game_type.team_size() == 2 {
            vec![
                example_me(game_type, rules),
                example_teammate_1(game_type, rules),
                example_opponent_1(game_type, rules),
                example_opponent_2(game_type, rules),
            ]
        } else {
            vec![
                example_me(game_type, rules),
                example_teammate_2(game_type, rules),
                example_teammate_1(game_type, rules),
                example_opponent_1(game_type, rules),
                example_opponent_3(game_type, rules),
                example_opponent_2(game_type, rules),
            ]
        },
        nitro_packs: if game_type.nitro() {
            vec![
                NitroPack {
                    id: 1,
                    x: -rules.NITRO_PACK_X,
                    y: rules.NITRO_PACK_Y,
                    z: -rules.NITRO_PACK_Z,
                    radius: rules.NITRO_PACK_RADIUS,
                    nitro_amount: rules.NITRO_PACK_AMOUNT,
                    respawn_ticks: None,
                },
                NitroPack {
                    id: 2,
                    x: -rules.NITRO_PACK_X,
                    y: rules.NITRO_PACK_Y,
                    z: rules.NITRO_PACK_Z,
                    radius: rules.NITRO_PACK_RADIUS,
                    nitro_amount: rules.NITRO_PACK_AMOUNT,
                    respawn_ticks: None,
                },
                NitroPack {
                    id: 3,
                    x: rules.NITRO_PACK_X,
                    y: rules.NITRO_PACK_Y,
                    z: -rules.NITRO_PACK_Z,
                    radius: rules.NITRO_PACK_RADIUS,
                    nitro_amount: rules.NITRO_PACK_AMOUNT,
                    respawn_ticks: None,
                },
                NitroPack {
                    id: 4,
                    x: rules.NITRO_PACK_X,
                    y: rules.NITRO_PACK_Y,
                    z: rules.NITRO_PACK_Z,
                    radius: rules.NITRO_PACK_RADIUS,
                    nitro_amount: rules.NITRO_PACK_AMOUNT,
                    respawn_ticks: None,
                },
            ]
        } else {
            vec![]
        },
        ball: example_ball(rules),
    }
}

pub fn example_me(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: 1,
        player_id: 1,
        is_teammate: true,
        x: 9.748591261158683,
        y: rules.ROBOT_RADIUS,
        z: -17.463246216636257,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_teammate_1(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: if game_type.team_size() == 2 {
            2
        } else {
            3
        },
        player_id: 1,
        is_teammate: true,
        x: -10.24931922557014,
        y: rules.ROBOT_RADIUS,
        z: -17.17415079159253,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_teammate_2(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: 2,
        player_id: 1,
        is_teammate: true,
        x: -0.289095425043726,
        y: rules.ROBOT_RADIUS,
        z: -19.997910486728827,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_opponent_1(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: if game_type.team_size() == 1 {
            2
        } else if game_type.team_size() == 2 {
            3
        } else {
            4
        },
        player_id: 2,
        is_teammate: false,
        x: -9.748591261158683,
        y: rules.ROBOT_RADIUS,
        z: 17.463246216636257,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_opponent_2(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: if game_type.team_size() == 2 {
            4
        } else {
            6
        },
        player_id: 2,
        is_teammate: false,
        x: 10.24931922557014,
        y: rules.ROBOT_RADIUS,
        z: 17.17415079159253,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_opponent_3(game_type: GameType, rules: &Rules) -> Robot {
    Robot {
        id: if game_type.team_size() == 2 {
            4
        } else {
            5
        },
        player_id: 2,
        is_teammate: false,
        x: 0.289095425043726,
        y: rules.ROBOT_RADIUS,
        z: 19.997910486728827,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.ROBOT_RADIUS,
        nitro_amount: if game_type.nitro() {
            rules.START_NITRO_AMOUNT
        } else {
            0.0
        },
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn example_ball(rules: &Rules) -> Ball {
    Ball {
        x: 0.0,
        y: 7.837328533066,
        z: 0.0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: rules.BALL_RADIUS,
    }
}

pub fn example_rules(game_type: GameType) -> Rules {
    Rules {
        max_tick_count: 18000,
        arena: example_arena(),
        team_size: game_type.team_size() as i32,
        seed: 42,
        ROBOT_MIN_RADIUS: 1.0,
        ROBOT_MAX_RADIUS: 1.05,
        ROBOT_MAX_JUMP_SPEED: 15.0,
        ROBOT_ACCELERATION: 100.0,
        ROBOT_NITRO_ACCELERATION: 30.0,
        ROBOT_MAX_GROUND_SPEED: 30.0,
        ROBOT_ARENA_E: 0.0,
        ROBOT_RADIUS: 1.0,
        ROBOT_MASS: 2.0,
        TICKS_PER_SECOND: 60,
        MICROTICKS_PER_TICK: 100,
        RESET_TICKS: 120,
        BALL_ARENA_E: 0.7,
        BALL_RADIUS: 2.0,
        BALL_MASS: 1.0,
        MIN_HIT_E: 0.4,
        MAX_HIT_E: 0.5,
        MAX_ENTITY_SPEED: 100.0,
        MAX_NITRO_AMOUNT: 100.0,
        START_NITRO_AMOUNT: 50.0,
        NITRO_POINT_VELOCITY_CHANGE: 0.6,
        NITRO_PACK_X: 20.0,
        NITRO_PACK_Y: 1.0,
        NITRO_PACK_Z: 30.0,
        NITRO_PACK_RADIUS: 0.5,
        NITRO_PACK_AMOUNT: 100.0,
        NITRO_PACK_RESPAWN_TICKS: 600,
        GRAVITY: 30.0,
    }
}

pub fn example_arena() -> Arena {
    Arena {
        width: 60.0,
        height: 20.0,
        depth: 80.0,
        bottom_radius: 3.0,
        top_radius: 7.0,
        corner_radius: 13.0,
        goal_top_radius: 3.0,
        goal_width: 30.0,
        goal_height: 10.0,
        goal_depth: 10.0,
        goal_side_radius: 1.0,
    }
}
