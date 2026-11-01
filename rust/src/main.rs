// Blobby Volley 3 (Remastered) - Main Game Engine
use macroquad::prelude::*;
use glam::Vec2;

mod constants;
mod physics;
mod rules;
mod ai;
mod audio;
mod vfx;
mod input;
mod render;

use constants::*;
use physics::{PhysicWorld, MatchEvent, PlayerSide};
use rules::{GameLogic, RuleMode};
use ai::{BotController, AiDifficulty};
use audio::AudioManager;
use vfx::VfxManager;
use render::blob::{render_blobby, BlobTheme};
use render::ball::{render_ball, BallTrail};
use render::net::NetRenderer;
use render::arenas::{render_arena, ArenaType};
use render::ui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    MainMenu,
    Playing,
    Paused,
    Victory(PlayerSide),
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Blobby Volley 3 (Remastered)".to_string(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut audio = AudioManager::new().await;
    let mut vfx = VfxManager::new();
    let mut net = NetRenderer::new();
    let mut ball_trail = BallTrail::new();

    // Default match configurations
    let mut game_mode = GameMode::SinglePlayer;
    let mut difficulty = AiDifficulty::Pro;
    let mut rule_mode = RuleMode::Modern;
    let mut score_to_win = 15;
    let mut arena = ArenaType::SunsetBeach;
    let mut left_theme = BlobTheme::ElectricCyan;
    let mut right_theme = BlobTheme::SunsetOrange;

    let mut state = AppState::MainMenu;
    let mut menu_selected = 0;
    let mut pause_selected = 0;

    let mut physics = PhysicWorld::new();
    let mut logic = GameLogic::new(rule_mode, score_to_win);

    let mut bot_left = BotController::new(PlayerSide::Left, difficulty);
    let mut bot_right = BotController::new(PlayerSide::Right, difficulty);

    let mut physics_accumulator = 0.0;
    let mut global_time = 0.0;

    loop {
        let frame_dt = get_frame_time().min(0.05);
        global_time += frame_dt;

        // F11 or Alt+Enter for Fullscreen toggle
        if is_key_pressed(KeyCode::F11) || ((is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt)) && is_key_pressed(KeyCode::Enter)) {
            // Note: macroquad handles windowed/fullscreen where supported
        }

        // Global Mute toggle
        if is_key_pressed(KeyCode::M) {
            audio.toggle_mute();
        }

        match state {
            AppState::MainMenu => {
                // Menu Navigation
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    menu_selected = if menu_selected == 0 { 7 } else { menu_selected - 1 };
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    menu_selected = (menu_selected + 1) % 8;
                }

                // Change settings left/right
                let left_pressed = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
                let right_pressed = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);

                if left_pressed || right_pressed {
                    match menu_selected {
                        1 => game_mode = game_mode.next(),
                        2 => {
                            difficulty = difficulty.next();
                            bot_left.difficulty = difficulty;
                            bot_right.difficulty = difficulty;
                        }
                        3 => {
                            rule_mode = match rule_mode {
                                RuleMode::Modern => RuleMode::Classic,
                                RuleMode::Classic => RuleMode::Modern,
                            };
                            logic.rule_mode = rule_mode;
                        }
                        4 => {
                            score_to_win = match score_to_win {
                                5 => 10,
                                10 => 15,
                                15 => 21,
                                _ => 5,
                            };
                            logic.score_to_win = score_to_win;
                        }
                        5 => arena = arena.next(),
                        6 => left_theme = left_theme.next(),
                        7 => right_theme = right_theme.next(),
                        _ => {}
                    }
                }

                // Start game on Space / Enter
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if menu_selected == 0 {
                        // Start match!
                        logic = GameLogic::new(rule_mode, score_to_win);
                        physics = PhysicWorld::new();
                        physics.reset_players();
                        physics.reset_ball(Some(PlayerSide::Left));
                        logic.on_serve();
                        bot_left = BotController::new(PlayerSide::Left, difficulty);
                        bot_right = BotController::new(PlayerSide::Right, difficulty);
                        state = AppState::Playing;
                        audio.play_whistle();
                    } else {
                        // Cycling through option
                        match menu_selected {
                            1 => game_mode = game_mode.next(),
                            2 => {
                                difficulty = difficulty.next();
                                bot_left.difficulty = difficulty;
                                bot_right.difficulty = difficulty;
                            }
                            3 => {
                                rule_mode = match rule_mode {
                                    RuleMode::Modern => RuleMode::Classic,
                                    RuleMode::Classic => RuleMode::Modern,
                                };
                                logic.rule_mode = rule_mode;
                            }
                            4 => {
                                score_to_win = match score_to_win {
                                    5 => 10,
                                    10 => 15,
                                    15 => 21,
                                    _ => 5,
                                };
                                logic.score_to_win = score_to_win;
                            }
                            5 => arena = arena.next(),
                            6 => left_theme = left_theme.next(),
                            7 => right_theme = right_theme.next(),
                            _ => {}
                        }
                    }
                }
            }

            AppState::Playing => {
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                    state = AppState::Paused;
                    pause_selected = 0;
                }

                // Input collection
                let p1_input = match game_mode {
                    GameMode::SinglePlayer => input::read_player1_input(true),
                    GameMode::TwoPlayerLocal => input::read_player1_input(false),
                    GameMode::BotVsBot => bot_left.update(&physics, logic.serving_player, logic.is_game_running),
                };

                let p2_input = match game_mode {
                    GameMode::SinglePlayer => bot_right.update(&physics, logic.serving_player, logic.is_game_running),
                    GameMode::TwoPlayerLocal => input::read_player2_input(),
                    GameMode::BotVsBot => bot_right.update(&physics, logic.serving_player, logic.is_game_running),
                };

                // Fixed-timestep physics simulation loop
                physics_accumulator += frame_dt;
                while physics_accumulator >= PHYSICS_DT {
                    physics_accumulator -= PHYSICS_DT;

                    // Physical step
                    physics.step(
                        p1_input,
                        p2_input,
                        logic.is_ball_valid,
                        logic.is_game_running,
                    );
                    logic.step(PHYSICS_DT);

                    // Process physics collision events
                    for event in physics.take_events() {
                        match event {
                            MatchEvent::BallHitBlob { player, intensity } => {
                                logic.on_ball_hits_blob(player);
                                vfx.spawn_hit_shockwave(physics.ball.pos, intensity);
                                audio.play_ball_hit(intensity);
                                net.trigger_hit(intensity * 0.3);
                            }
                            MatchEvent::BallHitGround { side } => {
                                let was_valid = logic.is_ball_valid;
                                logic.on_ball_hits_ground(side);
                                vfx.spawn_sand_puff(Vec2::new(physics.ball.pos.x, GROUND_PLANE_HEIGHT_MAX), 8);
                                if was_valid && !logic.is_ball_valid {
                                    audio.play_whistle();
                                    audio.play_score();
                                }
                                if !logic.is_ball_valid {
                                    physics.ball.vel *= 0.6;
                                }
                            }
                            MatchEvent::BallHitWall { side } => {
                                logic.on_ball_hits_wall(side);
                                audio.play_ball_hit(0.35);
                            }
                            MatchEvent::BallHitNet { side } => {
                                logic.on_ball_hits_net(side);
                                net.trigger_hit(0.7);
                                audio.play_ball_hit(0.3);
                            }
                            MatchEvent::BallHitNetTop => {
                                logic.on_ball_hits_net(PlayerSide::Left);
                                net.trigger_hit(1.0);
                                audio.play_ball_hit(0.4);
                            }
                        }
                    }

                    // Check for round reset / serve
                    if logic.can_start_round(&physics) {
                        if let Some(winner) = logic.winning_player {
                            state = AppState::Victory(winner);
                            vfx.spawn_confetti_celebration(Vec2::new(400.0, 200.0), 90);
                            audio.play_score();
                            break;
                        } else {
                            physics.reset_ball(Some(logic.serving_player));
                            logic.on_serve();
                            audio.play_whistle();
                        }
                    }
                }

                // Update visual effects & net dynamics
                net.update(frame_dt);
                vfx.update(frame_dt);
                ball_trail.update(physics.ball.pos, physics.ball.vel.length());
            }

            AppState::Paused => {
                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::Playing;
                }
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    pause_selected = if pause_selected == 0 { 2 } else { pause_selected - 1 };
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    pause_selected = (pause_selected + 1) % 3;
                }
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    match pause_selected {
                        0 => state = AppState::Playing, // Resume
                        1 => {
                            // Restart match
                            logic.reset_match();
                            physics.reset_players();
                            physics.reset_ball(Some(PlayerSide::Left));
                            logic.on_serve();
                            state = AppState::Playing;
                            audio.play_whistle();
                        }
                        2 => state = AppState::MainMenu, // Main Menu
                        _ => {}
                    }
                }
            }

            AppState::Victory(_) => {
                vfx.update(frame_dt);
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }
            }
        }

        // =====================================================================
        // RENDERING PIPELINE
        // =====================================================================
        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        // 2026 Widescreen Camera Projection
        let screen_w = screen_width();
        let screen_h = screen_height();
        let aspect = screen_w / screen_h;
        let (cam_w, cam_h) = if aspect >= VIRTUAL_WIDTH / VIRTUAL_HEIGHT {
            (VIRTUAL_HEIGHT * aspect, VIRTUAL_HEIGHT)
        } else {
            (VIRTUAL_WIDTH, VIRTUAL_WIDTH / aspect)
        };

        let shake = vfx.get_shake_offset();
        let cam_target = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5) + shake;

        set_camera(&Camera2D {
            target: cam_target,
            zoom: Vec2::new(2.0 / cam_w, -2.0 / cam_h), // Inverted y for screen coords
            ..Default::default()
        });

        // 1. Render Arena Environment
        render_arena(arena, global_time);

        // 2. Render Volleyball Net
        net.render(global_time);

        // 3. Render 2026 Remastered Procedural Jelly Blobbies
        render_blobby(
            &physics.blobs[0],
            PlayerSide::Left,
            left_theme,
            physics.ball.pos,
            global_time,
        );
        render_blobby(
            &physics.blobs[1],
            PlayerSide::Right,
            right_theme,
            physics.ball.pos,
            global_time,
        );

        // 4. Render 3D Volleyball & Motion Trail
        render_ball(&physics.ball, &ball_trail);

        // 5. Render Particle Systems & Visual Juice
        vfx.render();

        // 6. Reset to default camera for crisp pixel-perfect UI overlay
        set_default_camera();

        // Remap virtual coords to UI coordinates
        set_camera(&Camera2D {
            target: Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
            zoom: Vec2::new(2.0 / cam_w, -2.0 / cam_h),
            ..Default::default()
        });

        match state {
            AppState::MainMenu => {
                render_main_menu(
                    game_mode,
                    difficulty,
                    rule_mode,
                    score_to_win,
                    arena,
                    left_theme,
                    right_theme,
                    menu_selected,
                    global_time,
                );
            }
            AppState::Playing => {
                let left_title = match game_mode {
                    GameMode::BotVsBot => "CPU 1",
                    _ => "PLAYER 1",
                };
                let right_title = match game_mode {
                    GameMode::SinglePlayer => format!("CPU ({})", difficulty.name()),
                    GameMode::TwoPlayerLocal => "PLAYER 2".to_string(),
                    GameMode::BotVsBot => format!("CPU 2 ({})", difficulty.name()),
                };

                render_hud(
                    &logic,
                    left_title,
                    &right_title,
                    left_theme,
                    right_theme,
                    global_time,
                );
            }
            AppState::Paused => {
                let left_title = "PLAYER 1";
                let right_title = "PLAYER 2";
                render_hud(&logic, left_title, right_title, left_theme, right_theme, global_time);
                render_pause_menu(pause_selected);
            }
            AppState::Victory(winner) => {
                render_victory_screen(winner, &logic, global_time);
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
