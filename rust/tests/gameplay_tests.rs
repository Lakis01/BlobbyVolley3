// Blobby Volley 3 - Integration & Verification Tests
use blobby_core::constants::*;
use blobby_core::physics::*;
use blobby_core::rules::*;
use blobby_core::ai::*;
use glam::Vec2;

#[test]
fn test_physics_constants() {
    assert_eq!(LEFT_PLANE, 0.0);
    assert_eq!(RIGHT_PLANE, 800.0);
    assert_eq!(GROUND_PLANE_HEIGHT_MAX, 500.0);
    assert_eq!(GROUND_PLANE_HEIGHT, 455.5);
    assert_eq!(NET_POSITION_X, 400.0);
    assert_eq!(NET_SPHERE_POSITION, 284.0);
    assert_eq!(BALL_RADIUS, 31.5);
}

#[test]
fn test_blobby_jump_mechanics() {
    let mut physics = PhysicWorld::new();
    let left_input = PlayerInput {
        left: false,
        right: false,
        up: true,
    };
    let right_input = PlayerInput::default();

    // Start on ground
    assert!(physics.blob_hit_ground(PlayerSide::Left));
    assert_eq!(physics.blobs[0].pos.y, GROUND_PLANE_HEIGHT);

    // Initial jump step
    physics.step(left_input, right_input, true, true);
    assert!(physics.blobs[0].vel.y < 0.0, "Velocity must be upwards after jump");
    assert!(!physics.blob_hit_ground(PlayerSide::Left));

    // Jump simulation to apex
    let mut min_y = physics.blobs[0].pos.y;
    for _ in 0..60 {
        physics.step(left_input, right_input, true, true);
        if physics.blobs[0].pos.y < min_y {
            min_y = physics.blobs[0].pos.y;
        }
    }

    // Apex distance from ground matches BLOBBY_MAX_JUMP_HEIGHT (249.125)
    let jump_height = GROUND_PLANE_HEIGHT - min_y;
    assert!(
        (jump_height - BLOBBY_MAX_JUMP_HEIGHT).abs() < 1.0,
        "Jump height was {}, expected ~{}",
        jump_height,
        BLOBBY_MAX_JUMP_HEIGHT
    );
}

#[test]
fn test_ball_ground_bounce() {
    let mut physics = PhysicWorld::new();
    physics.ball.pos = Vec2::new(200.0, 480.0);
    physics.ball.vel = Vec2::new(0.0, 10.0);

    // Step physics with game running
    physics.step(PlayerInput::default(), PlayerInput::default(), true, true);

    let events = physics.take_events();
    let hit_ground = events.iter().any(|e| matches!(e, MatchEvent::BallHitGround { side: PlayerSide::Left }));
    assert!(hit_ground, "Ball must trigger BallHitGround event");
    assert!(physics.ball.vel.y < 0.0, "Ball must bounce upwards");
}

#[test]
fn test_ball_net_reflection() {
    let mut physics = PhysicWorld::new();
    physics.ball.pos = Vec2::new(NET_POSITION_X - 10.0, NET_SPHERE_POSITION);
    physics.ball.vel = Vec2::new(12.0, 0.0);

    physics.step(PlayerInput::default(), PlayerInput::default(), true, true);

    let events = physics.take_events();
    let hit_net = events.iter().any(|e| matches!(e, MatchEvent::BallHitNet { .. } | MatchEvent::BallHitNetTop));
    assert!(hit_net, "Ball hitting net must trigger net event");
    assert!(physics.ball.vel.x < 0.0, "Ball moving right must bounce back left");
}

#[test]
fn test_modern_rules_scoring() {
    let mut logic = GameLogic::new(RuleMode::Modern, 15);
    assert_eq!(logic.scores, [0, 0]);

    // Ground hit on Right court awards point to Left
    logic.on_ball_hits_ground(PlayerSide::Right);
    assert_eq!(logic.scores, [1, 0]);
    assert_eq!(logic.serving_player, PlayerSide::Left);

    // Server serves next round
    logic.on_serve();

    // Ground hit on Left court awards point to Right
    logic.on_ball_hits_ground(PlayerSide::Left);
    assert_eq!(logic.scores, [1, 1]);
    assert_eq!(logic.serving_player, PlayerSide::Right);
}

#[test]
fn test_classic_rules_sideout() {
    let mut logic = GameLogic::new(RuleMode::Classic, 15);
    logic.serving_player = PlayerSide::Left;
    assert_eq!(logic.scores, [0, 0]);

    // Left is serving. Right faults -> Left scores 1 point!
    logic.on_ball_hits_ground(PlayerSide::Right);
    assert_eq!(logic.scores, [1, 0]);
    assert_eq!(logic.serving_player, PlayerSide::Left);

    // Left serves next round
    logic.on_serve();

    // Left is serving. Left faults -> Right gains serve, NO point!
    logic.on_ball_hits_ground(PlayerSide::Left);
    assert_eq!(logic.scores, [1, 0]);
    assert_eq!(logic.serving_player, PlayerSide::Right);

    // Right serves next round
    logic.on_serve();

    // Right is now serving. Left faults -> Right scores 1 point!
    logic.on_ball_hits_ground(PlayerSide::Left);
    assert_eq!(logic.scores, [1, 1]);
    assert_eq!(logic.serving_player, PlayerSide::Right);
}

#[test]
fn test_ai_trajectory_estimation() {
    let ball_pos = Vec2::new(200.0, 200.0);
    let ball_vel = Vec2::new(4.0, 2.0);

    let est = estimate_ball_trajectory(ball_pos, ball_vel, 400.0);
    assert!(est.is_some(), "Trajectory should predict target at y=400");
    let (target_x, _vx, ticks) = est.unwrap();
    assert!(target_x > 200.0, "Ball moving right should have target_x > 200");
    assert!(ticks > 0.0, "Ticks must be positive");
}

#[test]
fn test_bot_vs_bot_rally_simulation() {
    let mut physics = PhysicWorld::new();
    let mut logic = GameLogic::new(RuleMode::Modern, 15);
    physics.reset_ball(Some(PlayerSide::Left));
    logic.on_serve();

    let mut bot_master = BotController::new(PlayerSide::Left, AiDifficulty::Master);
    let mut bot_reduced = BotController::new(PlayerSide::Right, AiDifficulty::Pro);

    let mut total_blob_hits = 0;
    let mut longest_rally = 0;
    let mut current_rally = 0;

    for _step in 0..4000 {
        let p1_input = bot_master.update(&physics, logic.touches[0], logic.serving_player, logic.is_game_running);
        let p2_input = bot_reduced.update(&physics, logic.touches[1], logic.serving_player, logic.is_game_running);

        physics.step(p1_input, p2_input, logic.is_ball_valid, logic.is_game_running);
        logic.step(PHYSICS_DT);

        let events = physics.take_events();
        for event in events {
            match event {
                blobby_core::physics::MatchEvent::BallHitBlob { player, .. } => {
                    logic.on_ball_hits_blob(player);
                    total_blob_hits += 1;
                    current_rally += 1;
                    if current_rally > longest_rally {
                        longest_rally = current_rally;
                    }
                }
                blobby_core::physics::MatchEvent::BallHitGround { side } => {
                    logic.on_ball_hits_ground(side);
                    current_rally = 0;
                }
                blobby_core::physics::MatchEvent::BallHitWall { side } => {
                    logic.on_ball_hits_wall(side);
                }
                blobby_core::physics::MatchEvent::BallHitNet { side } => {
                    logic.on_ball_hits_net(side);
                }
                blobby_core::physics::MatchEvent::BallHitNetTop => {
                    logic.on_ball_hits_net(PlayerSide::Left);
                }
            }
        }

        if logic.can_start_round(&physics) {
            physics.reset_ball(Some(logic.serving_player));
            logic.on_serve();
            bot_master.reset_rally();
            bot_reduced.reset_rally();
        }
    }

    println!("Simulation 4000 ticks (~53s):");
    println!("Total blob hits: {}", total_blob_hits);
    println!("Longest rally: {} hits", longest_rally);
    println!("Score: Master (Left) {} - {} Pro (Right)", logic.scores[0], logic.scores[1]);
    assert!(total_blob_hits >= 25, "Master and Pro bots should produce competitive rallies (got {} hits)", total_blob_hits);
}
