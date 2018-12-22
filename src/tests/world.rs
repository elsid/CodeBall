use crate::model::{Arena, Ball, Game, Player, Robot, Rules};
use crate::my_strategy::world::World;

pub fn default_world() -> World {
    World::new(default_me(), default_rules(), default_game())
}

pub fn default_game() -> Game {
    Game {
        current_tick: 0,
        players: vec![
            Player {
                id: 1,
                me: true,
                strategy_crashed: false,
                score: 0,
            },
            Player {
                id: 2,
                me: false,
                strategy_crashed: false,
                score: 0,
            },
        ],
        robots: vec![
            default_me(),
            Robot {
                id: 2,
                player_id: 2,
                is_teammate: false,
                x: 0.289095425043726,
                y: 1.0,
                z: 19.997910486728827,
                velocity_x: 0.0,
                velocity_y: 0.0,
                velocity_z: 0.0,
                radius: 1.0,
                nitro_amount: 0.0,
                touch: true,
                touch_normal_x: Some(0.0),
                touch_normal_y: Some(1.0),
                touch_normal_z: Some(0.0),
            }
        ],
        nitro_packs: vec![],
        ball: default_ball(),
    }
}

pub fn default_me() -> Robot {
    Robot {
        id: 1,
        player_id: 1,
        is_teammate: true,
        x: -0.289095425043726,
        y: 1.0,
        z: -19.997910486728827,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: 1.0,
        nitro_amount: 0.0,
        touch: true,
        touch_normal_x: Some(0.0),
        touch_normal_y: Some(1.0),
        touch_normal_z: Some(0.0),
    }
}

pub fn default_ball() -> Ball {
    Ball {
        x: 0.0,
        y: 7.837328533066,
        z: 0.0,
        velocity_x: 0.0,
        velocity_y: 0.0,
        velocity_z: 0.0,
        radius: 2.0,
    }
}

pub fn default_rules() -> Rules {
    Rules {
        max_tick_count: 18000,
        arena: default_arena(),
        team_size: 1,
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

pub fn default_arena() -> Arena {
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
