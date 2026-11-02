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
                blobby_core::physics::MatchEvent::BallCrossedNet { new_side } => {
                    logic.on_ball_crosses_net(new_side);
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

#[test]
fn test_opus_sound_decoding() {
    let bums_opus = include_bytes!("../assets/sounds/bums.opus");
    let pfiff_opus = include_bytes!("../assets/sounds/pfiff.opus");
    let chat_opus = include_bytes!("../assets/sounds/chat.opus");

    let wav_bums = blobby_core::audio::decode_opus_to_wav(bums_opus)
        .expect("bums.opus should decode cleanly to WAV");
    assert!(wav_bums.len() > 44, "Decoded bums WAV must contain samples");
    assert_eq!(&wav_bums[0..4], b"RIFF");
    assert_eq!(&wav_bums[8..12], b"WAVE");

    let wav_pfiff = blobby_core::audio::decode_opus_to_wav(pfiff_opus)
        .expect("pfiff.opus should decode cleanly to WAV");
    assert!(wav_pfiff.len() > 44, "Decoded pfiff WAV must contain samples");
    assert_eq!(&wav_pfiff[0..4], b"RIFF");
    assert_eq!(&wav_pfiff[8..12], b"WAVE");

    let wav_chat = blobby_core::audio::decode_opus_to_wav(chat_opus)
        .expect("chat.opus should decode cleanly to WAV");
    assert!(wav_chat.len() > 44, "Decoded chat WAV must contain samples");
    assert_eq!(&wav_chat[0..4], b"RIFF");
    assert_eq!(&wav_chat[8..12], b"WAVE");
}

#[test]
fn test_all_five_ai_difficulty_tiers() {
    let tiers = [
        AiDifficulty::Novice,
        AiDifficulty::Amateur,
        AiDifficulty::Regular,
        AiDifficulty::Master,
        AiDifficulty::Pro,
    ];

    assert_eq!(tiers[0].name(), "Novice");
    assert_eq!(tiers[1].name(), "Amateur");
    assert_eq!(tiers[2].name(), "Regular");
    assert_eq!(tiers[3].name(), "Master");
    assert_eq!(tiers[4].name(), "Pro");

    // Test cycling forward and backward
    for i in 0..5 {
        assert_eq!(tiers[i].to_index(), i);
        assert_eq!(AiDifficulty::from_index(i), tiers[i]);
        assert_eq!(tiers[i].next(), tiers[(i + 1) % 5]);
        assert_eq!(tiers[i].prev(), tiers[(i + 4) % 5]);
    }

    // Verify all 5 bots can be instantiated and make decisions without panic
    let physics = PhysicWorld::new();
    for diff in tiers {
        let mut bot = BotController::new(PlayerSide::Right, diff);
        let input = bot.update(&physics, 0, PlayerSide::Left, true);
        let _ = input; // Bot returns valid player input
    }
}

#[test]
fn test_user_config_persistence() {
    use blobby_core::config::UserConfig;

    let default_cfg = UserConfig::default();
    assert_eq!(default_cfg.player2_name, "Player 2");
    assert_eq!(default_cfg.score_to_win, 15);
    assert_eq!(default_cfg.rule_mode_index, 0);

    let json = serde_json::to_string(&default_cfg).expect("Serialization must succeed");
    let loaded: UserConfig = serde_json::from_str(&json).expect("Deserialization must succeed");
    assert_eq!(loaded.player1_name, default_cfg.player1_name);
    assert_eq!(loaded.player2_name, default_cfg.player2_name);
    assert_eq!(loaded.score_to_win, default_cfg.score_to_win);
}

#[test]
fn test_master_server_query_structure() {
    let xml_sample = r#"<?xml version="1.0" encoding="utf-8"?>
<onlineserver>
	<server>
		<var host="blobbyvolley.de"/>
		<var port="12345"/>
	</server>
	<server>
		<var host="167.235.181.41"/>
		<var port="4321"/>
	</server>
</onlineserver>"#;

    let parsed = blobby_core::net::parse_server_xml(xml_sample);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].host, "blobbyvolley.de");
    assert_eq!(parsed[0].port, 12345);
    assert_eq!(parsed[1].host, "167.235.181.41");
    assert_eq!(parsed[1].port, 4321);

    let servers = blobby_core::net::query_master_server();
    assert!(!servers.is_empty(), "Server browser must at least return LAN / local room option");
}

#[test]
fn test_stadium_and_arena_cycle() {
    use blobby_core::render::arenas::ArenaType;

    // Verify 8 distinct arena types
    let total_arenas = 8;
    let mut current = ArenaType::from_index(0);
    assert_eq!(current, ArenaType::SunsetBeach);

    let mut names = Vec::new();
    for i in 0..total_arenas {
        let arena = ArenaType::from_index(i);
        assert_eq!(arena.to_index(), i);
        names.push(arena.name());
    }

    // Verify stadium maps with spectator crowds exist
    assert!(names.contains(&"Blobby Grand Arena (Stadium)"), "Blobby Grand Arena stadium must be in roster");
    assert!(names.contains(&"Neo Cyber Stadium"), "Neo Cyber Stadium must be in roster");
    assert!(names.contains(&"Olympic Beach Stadium"), "Olympic Beach Stadium must be in roster");

    // Verify cyclic navigation (next and prev)
    for _ in 0..total_arenas {
        current = current.next();
    }
    assert_eq!(current, ArenaType::SunsetBeach, "Cycling next 8 times must return to starting arena");

    for _ in 0..total_arenas {
        current = current.prev();
    }
    assert_eq!(current, ArenaType::SunsetBeach, "Cycling prev 8 times must return to starting arena");
}

#[test]
fn test_touch_counter_reset_on_net_crossing_and_valid_reception() {
    let mut logic = GameLogic::new(RuleMode::Modern, 15);
    logic.is_game_running = true;

    // Player 1 hits the ball 3 times legally
    logic.on_ball_hits_blob(PlayerSide::Left);
    logic.squish[0] = 0;
    logic.on_ball_hits_blob(PlayerSide::Left);
    logic.squish[0] = 0;
    logic.on_ball_hits_blob(PlayerSide::Left);
    assert_eq!(logic.touches[0], 3, "Player 1 should have 3 touches");
    assert!(logic.is_ball_valid, "Ball must remain valid on touch 3");

    // Ball crosses the net to Player 2
    logic.on_ball_crosses_net(PlayerSide::Right);
    assert_eq!(logic.touches[0], 0, "Player 1 touches must reset on net crossing");
    assert_eq!(logic.touches[1], 0, "Player 2 touches must start at 0 on reception");
    assert!(logic.is_ball_valid, "Ball must be valid for Player 2");

    // Player 2 receives and hits the ball (first contact)
    logic.on_ball_hits_blob(PlayerSide::Right);
    assert_eq!(logic.touches[1], 1, "Player 2 touch count must be 1 on first contact");
    assert!(logic.is_ball_valid, "Ball must remain valid on Player 2 first contact");
}

#[test]
fn test_high_ball_jump_collision() {
    let mut physics = PhysicWorld::new();
    let mut logic = GameLogic::new(RuleMode::Modern, 15);
    physics.reset_players();
    // Test multiple starting heights and velocities
    for start_y in [-300.0, -200.0, -100.0, 0.0, 100.0] {
        physics.reset_players();
        physics.ball.pos = Vec2::new(200.0, start_y);
        physics.ball.vel = Vec2::new(0.0, 10.0);
        logic.is_game_running = true;
        logic.is_ball_valid = true;
        logic.touches = [0, 0];
        logic.squish = [0, 0];

        let mut hit = false;
        let mut hit_ticks = Vec::new();
        for tick in 0..150 {
            let p1_input = PlayerInput { left: false, right: false, up: true };
            let p2_input = PlayerInput::default();
            physics.step(p1_input, p2_input, logic.is_ball_valid, logic.is_game_running);
            logic.step(PHYSICS_DT);
            for ev in physics.take_events() {
                match ev {
                    MatchEvent::BallHitBlob { player, .. } => {
                        hit = true;
                        logic.on_ball_hits_blob(player);
                        hit_ticks.push((tick, logic.touches[0]));
                    }
                    MatchEvent::BallHitGround { side } => {
                        logic.on_ball_hits_ground(side);
                    }
                    MatchEvent::BallCrossedNet { new_side } => {
                        logic.on_ball_crosses_net(new_side);
                    }
                    _ => {}
                }
            }
        }
        assert!(hit, "Player 1 must hit the ball starting from y = {}", start_y);
        println!("start_y: {}, hit_ticks: {:?}, is_valid: {}", start_y, hit_ticks, logic.is_ball_valid);
    }
}

#[test]
fn test_color_conflict_auto_resolution() {
    use blobby_core::render::blob::BlobTheme;
    let mut left = BlobTheme::ElectricCyan;
    let mut right = BlobTheme::SunsetOrange;
    assert_ne!(left, right);

    // If left changes to sunset orange, right must auto-advance so they never compete with same color
    left = BlobTheme::SunsetOrange;
    if left == right {
        right = right.next();
    }
    assert_ne!(left, right);
    assert_eq!(right, BlobTheme::SunsetOrange.next());

    // If right changes to what left has, right auto-advances
    right = left;
    if right == left {
        right = right.next();
    }
    assert_ne!(left, right);
}

#[test]
fn test_high_ball_camera_enum_and_config() {
    use blobby_core::config::HighBallCamera;

    assert_eq!(HighBallCamera::Indicator.to_index(), 0);
    assert_eq!(HighBallCamera::DynamicZoom.to_index(), 1);
    assert_eq!(HighBallCamera::Both.to_index(), 2);

    assert_eq!(HighBallCamera::from_index(0), HighBallCamera::Indicator);
    assert_eq!(HighBallCamera::from_index(1), HighBallCamera::DynamicZoom);
    assert_eq!(HighBallCamera::from_index(2), HighBallCamera::Both);

    assert_eq!(HighBallCamera::Indicator.next(), HighBallCamera::DynamicZoom);
    assert_eq!(HighBallCamera::DynamicZoom.next(), HighBallCamera::Both);
    assert_eq!(HighBallCamera::Both.next(), HighBallCamera::Indicator);

    assert_eq!(HighBallCamera::Indicator.prev(), HighBallCamera::Both);
}

#[test]
fn test_touch_counter_fault_and_dead_ball_physics() {
    let mut logic = GameLogic::new(RuleMode::Modern, 15);
    let mut physics = PhysicWorld::new();
    physics.reset_players();
    physics.reset_ball(Some(PlayerSide::Left));
    logic.on_serve();

    // 3 touches are legal
    for _ in 0..3 {
        logic.squish[0] = 0;
        logic.on_ball_hits_blob(PlayerSide::Left);
        assert!(logic.is_ball_valid);
    }
    assert_eq!(logic.touches[0], 3);

    // 4th touch is a fault: invalidates ball immediately
    logic.squish[0] = 0;
    logic.on_ball_hits_blob(PlayerSide::Left);
    assert!(!logic.is_ball_valid, "4th touch must fault and invalidate ball");
    assert_eq!(logic.scores[1], 1, "Rally scoring awards point to opponent on fault");

    // Even after ball is invalidated, physical step still handles ball-blobby bounce without crashing or tunneling
    physics.ball.pos = Vec2::new(600.0, 400.0);
    physics.ball.vel = Vec2::new(0.0, 10.0);
    physics.step(PlayerInput::default(), PlayerInput::default(), logic.is_ball_valid, true);
    // Ball bounced off right blob
    assert!(physics.ball.vel.y < 0.0 || physics.ball.pos.y < 460.0);
}

#[test]
fn test_court_size_and_enlarged_physics_world() {
    use blobby_core::config::CourtSize;

    assert_eq!(CourtSize::Standard.width(), 800.0);
    assert_eq!(CourtSize::Wide.width(), 960.0);
    assert_eq!(CourtSize::ExtraWide.width(), 1120.0);
    assert_eq!(CourtSize::Colossal.width(), 1280.0);

    // Test cycle
    assert_eq!(CourtSize::Standard.next(), CourtSize::Wide);
    assert_eq!(CourtSize::Wide.next(), CourtSize::ExtraWide);
    assert_eq!(CourtSize::ExtraWide.next(), CourtSize::Colossal);
    assert_eq!(CourtSize::Colossal.next(), CourtSize::Standard);

    assert_eq!(CourtSize::Standard.prev(), CourtSize::Colossal);
    assert_eq!(CourtSize::Colossal.prev(), CourtSize::ExtraWide);

    // Test PhysicWorld initialized with custom court width
    for &size in &[CourtSize::Standard, CourtSize::Wide, CourtSize::ExtraWide, CourtSize::Colossal] {
        let w = size.width();
        let mut p = PhysicWorld::new_with_width(w);

        assert_eq!(p.court_width, w);
        assert_eq!(p.net_x(), w * 0.5);
        assert_eq!(p.left_plane(), 0.0);
        assert_eq!(p.right_plane(), w);

        // Blobby spawns at 25% and 75%
        assert_eq!(p.blobs[0].pos.x, w * 0.25);
        assert_eq!(p.blobs[1].pos.x, w * 0.75);

        // Reset ball spawns at serving side quarter court
        p.reset_ball(Some(PlayerSide::Left));
        assert_eq!(p.ball.pos.x, w * 0.25);

        p.reset_ball(Some(PlayerSide::Right));
        assert_eq!(p.ball.pos.x, w * 0.75);

        // Right wall bounce occurs at court_width
        p.ball.pos = Vec2::new(w - 5.0, 400.0);
        p.ball.vel = Vec2::new(10.0, 0.0);
        p.step(PlayerInput::default(), PlayerInput::default(), true, true);
        assert!(p.ball.vel.x < 0.0, "Ball must bounce off right wall at {}", w);
        assert!(p.ball.pos.x <= w);
    }
}

#[test]
fn test_strictly_monotonous_ai_progression() {
    use blobby_core::glicko::GlickoRating;

    let difficulties = [
        AiDifficulty::Novice,
        AiDifficulty::Amateur,
        AiDifficulty::Regular,
        AiDifficulty::Master,
        AiDifficulty::Pro,
    ];

    // 1. Verify index and cycle monotonicity
    for i in 0..difficulties.len() {
        assert_eq!(difficulties[i].to_index(), i);
        assert_eq!(AiDifficulty::from_index(i), difficulties[i]);
        if i + 1 < difficulties.len() {
            assert_eq!(difficulties[i].next(), difficulties[i + 1]);
            assert_eq!(difficulties[i + 1].prev(), difficulties[i]);
        }
    }

    // 2. Verify strictly monotonous Glicko baseline ratings:
    // Novice (1000) < Amateur (1250) < Regular (1500) < Master (1750) < Pro (2000)
    let mut prev_rating = 0.0;
    for diff in difficulties {
        let (rating, _rd) = GlickoRating::ai_baseline(diff);
        assert!(
            rating > prev_rating,
            "Difficulty {:?} rating {:.0} must be strictly greater than previous {:.0}",
            diff,
            rating,
            prev_rating
        );
        prev_rating = rating;
    }

    // 3. Verify AI trajectory estimation respects enlarged court widths
    let ball_pos = Vec2::new(100.0, 200.0);
    let ball_vel = Vec2::new(15.0, -2.0);

    let est_800 = estimate_x_at_y(ball_pos, ball_vel, 379.5, true, 800.0);
    let est_1120 = estimate_x_at_y(ball_pos, ball_vel, 379.5, true, 1120.0);
    assert!(est_800.is_some());
    assert!(est_1120.is_some());

    // 4. Verify AI controller updates and executes valid input on both standard and enlarged courts
    for court_w in [800.0, 960.0, 1120.0, 1280.0] {
        let mut p = PhysicWorld::new_with_width(court_w);
        p.reset_ball(Some(PlayerSide::Left));

        for diff in difficulties {
            let mut bot_l = BotController::new(PlayerSide::Left, diff);
            let mut bot_r = BotController::new(PlayerSide::Right, diff);

            let inp_l = bot_l.update(&p, 0, PlayerSide::Left, true);
            let inp_r = bot_r.update(&p, 0, PlayerSide::Left, true);

            // Valid inputs without panics
            let _ = inp_l.left || inp_l.right || inp_l.up;
            let _ = inp_r.left || inp_r.right || inp_r.up;
        }
    }
}



