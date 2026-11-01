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
mod net;
mod render;

use constants::*;
use physics::{PhysicWorld, MatchEvent, PlayerSide};
use rules::{GameLogic, RuleMode};
use ai::{BotController, AiDifficulty};
use audio::AudioManager;
use vfx::VfxManager;
use net::{NetworkHost, NetworkClient, NetEvent, get_local_ip, DEFAULT_PORT};
use render::blob::{render_blobby, BlobTheme};
use render::ball::{render_ball, BallTrail};
use render::net::NetRenderer;
use render::arenas::{render_arena, ArenaType};
use render::ui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    MainMenu,
    HostLobby,
    JoinScreen,
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
    let mut net_renderer = NetRenderer::new();
    let mut ball_trail = BallTrail::new();

    // Default match configurations
    let mut game_mode = GameMode::SinglePlayer;
    let mut difficulty = AiDifficulty::Master;
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

    // Online Networking State
    let mut net_host: Option<NetworkHost> = None;
    let mut net_client: Option<NetworkClient> = None;
    let mut join_ip_input = format!("127.0.0.1:{}", DEFAULT_PORT);
    let local_ip = get_local_ip();

    // In-game Chat State
    let mut chat_log: Vec<(String, String, f32)> = Vec::new();
    let mut is_typing_chat = false;
    let mut chat_input = String::new();

    let mut physics_accumulator = 0.0;
    let mut global_time = 0.0;

    loop {
        let frame_dt = get_frame_time().min(0.05);
        global_time += frame_dt;

        // Global Mute toggle (M)
        if !is_typing_chat && state != AppState::JoinScreen && is_key_pressed(KeyCode::M) {
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
                    match menu_selected {
                        0 => {
                            match game_mode {
                                GameMode::HostOnline => {
                                    if let Ok(host) = NetworkHost::bind(DEFAULT_PORT) {
                                        net_host = Some(host);
                                        state = AppState::HostLobby;
                                    }
                                }
                                GameMode::JoinOnline => {
                                    state = AppState::JoinScreen;
                                }
                                _ => {
                                    // Start local match
                                    logic = GameLogic::new(rule_mode, score_to_win);
                                    physics = PhysicWorld::new();
                                    physics.reset_players();
                                    physics.reset_ball(Some(PlayerSide::Left));
                                    logic.on_serve();
                                    bot_left = BotController::new(PlayerSide::Left, difficulty);
                                    bot_right = BotController::new(PlayerSide::Right, difficulty);
                                    bot_left.reset_rally();
                                    bot_right.reset_rally();
                                    state = AppState::Playing;
                                    audio.play_whistle();
                                }
                            }
                        }
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

            AppState::HostLobby => {
                if is_key_pressed(KeyCode::Escape) {
                    net_host = None;
                    state = AppState::MainMenu;
                } else if let Some(ref mut host) = net_host {
                    host.update();
                    if host.is_client_connected {
                        // Opponent connected! Initialize authoritative match
                        logic = GameLogic::new(rule_mode, score_to_win);
                        physics = PhysicWorld::new();
                        physics.reset_players();
                        physics.reset_ball(Some(PlayerSide::Left));
                        logic.on_serve();
                        chat_log.clear();
                        state = AppState::Playing;
                        audio.play_whistle();
                    }
                }
            }

            AppState::JoinScreen => {
                if is_key_pressed(KeyCode::Escape) {
                    net_client = None;
                    state = AppState::MainMenu;
                } else {
                    // Text typing for server address
                    while let Some(c) = get_char_pressed() {
                        if (c.is_ascii_digit() || c == '.' || c == ':') && join_ip_input.len() < 24 {
                            join_ip_input.push(c);
                        }
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        join_ip_input.pop();
                    }

                    if is_key_pressed(KeyCode::Enter) {
                        let mut client = NetworkClient::new();
                        client.connect(&join_ip_input);
                        if client.is_connected {
                            net_client = Some(client);
                            chat_log.clear();
                            state = AppState::Playing;
                            audio.play_whistle();
                        } else {
                            net_client = Some(client);
                        }
                    }

                    if let Some(ref mut client) = net_client {
                        client.update();
                        if client.is_connected {
                            chat_log.clear();
                            state = AppState::Playing;
                            audio.play_whistle();
                        }
                    }
                }
            }

            AppState::Playing => {
                // In-game Chat Handling
                if is_typing_chat {
                    while let Some(c) = get_char_pressed() {
                        if c.is_ascii_graphic() || c == ' ' {
                            if chat_input.len() < 40 {
                                chat_input.push(c);
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Backspace) {
                        chat_input.pop();
                    }
                    if is_key_pressed(KeyCode::Enter) && !chat_input.trim().is_empty() {
                        let my_name = if net_host.is_some() { "Host" } else { "Client" };
                        let text = chat_input.clone();
                        chat_log.push((my_name.to_string(), text.clone(), global_time));
                        audio.play_score();

                        if let Some(ref mut host) = net_host {
                            host.send_chat(my_name, &text);
                        } else if let Some(ref mut client) = net_client {
                            client.send_chat(my_name, &text);
                        }
                        chat_input.clear();
                        is_typing_chat = false;
                    } else if is_key_pressed(KeyCode::Escape) {
                        chat_input.clear();
                        is_typing_chat = false;
                    }
                } else {
                    // Open chat on 'T' or 'C'
                    if is_key_pressed(KeyCode::T) || is_key_pressed(KeyCode::C) {
                        is_typing_chat = true;
                        chat_input.clear();
                    }

                    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                        state = AppState::Paused;
                        pause_selected = 0;
                    }
                }

                // =============================================================
                // Network Client update (Client Side)
                // =============================================================
                if let Some(ref mut client) = net_client {
                    client.update();

                    // Send local input (Right Player / Player 2)
                    let my_input = input::read_player1_input(true);
                    client.send_input(my_input);

                    // Receive authoritative game state from host
                    if let Some(ref net_state) = client.latest_state {
                        physics.ball.pos = Vec2::new(net_state.ball_pos.0, net_state.ball_pos.1);
                        physics.ball.vel = Vec2::new(net_state.ball_vel.0, net_state.ball_vel.1);
                        physics.ball.rotation = net_state.ball_rotation;

                        physics.blobs[0].pos = Vec2::new(net_state.p1_pos.0, net_state.p1_pos.1);
                        physics.blobs[0].vel = Vec2::new(net_state.p1_vel.0, net_state.p1_vel.1);
                        physics.blobs[0].squash_factor = net_state.p1_squash;

                        physics.blobs[1].pos = Vec2::new(net_state.p2_pos.0, net_state.p2_pos.1);
                        physics.blobs[1].vel = Vec2::new(net_state.p2_vel.0, net_state.p2_vel.1);
                        physics.blobs[1].squash_factor = net_state.p2_squash;

                        logic.scores = net_state.scores;
                        logic.serving_player = if net_state.serving_player == 0 { PlayerSide::Left } else { PlayerSide::Right };
                        logic.is_game_running = net_state.is_game_running;
                        logic.is_ball_valid = net_state.is_ball_valid;

                        for ev in &net_state.events {
                            match ev {
                                NetEvent::HitBlob { intensity, .. } => {
                                    audio.play_ball_hit(*intensity);
                                    vfx.spawn_hit_shockwave(physics.ball.pos, *intensity);
                                    net_renderer.trigger_hit(*intensity * 0.3);
                                }
                                NetEvent::HitGround { .. } => {
                                    audio.play_score();
                                    vfx.spawn_sand_puff(Vec2::new(physics.ball.pos.x, GROUND_PLANE_HEIGHT_MAX), 8);
                                }
                                NetEvent::HitWall => audio.play_ball_hit(0.35),
                                NetEvent::HitNet => {
                                    audio.play_ball_hit(0.3);
                                    net_renderer.trigger_hit(0.8);
                                }
                                NetEvent::ScorePoint => audio.play_score(),
                                NetEvent::Whistle => audio.play_whistle(),
                            }
                        }
                    }

                    // Process chat
                    for (sender, text) in client.new_chat_messages.drain(..) {
                        chat_log.push((sender, text, global_time));
                        audio.play_score();
                    }

                    if !client.is_connected {
                        // Host disconnected
                        state = AppState::MainMenu;
                    }
                }
                // =============================================================
                // Host / Local authoritative simulation
                // =============================================================
                else {
                    let (p1_input, p2_input) = if let Some(ref mut host) = net_host {
                        host.update();
                        for (sender, text) in host.new_chat_messages.drain(..) {
                            chat_log.push((sender, text, global_time));
                            audio.play_score();
                        }
                        (input::read_player1_input(true), host.client_input)
                    } else {
                        let p1 = match game_mode {
                            GameMode::SinglePlayer => input::read_player1_input(true),
                            GameMode::TwoPlayerLocal => input::read_player1_input(false),
                            GameMode::BotVsBot => bot_left.update(&physics, logic.touches[0], logic.serving_player, logic.is_game_running),
                            _ => input::read_player1_input(true),
                        };

                        let p2 = match game_mode {
                            GameMode::SinglePlayer => bot_right.update(&physics, logic.touches[1], logic.serving_player, logic.is_game_running),
                            GameMode::TwoPlayerLocal => input::read_player2_input(),
                            GameMode::BotVsBot => bot_right.update(&physics, logic.touches[1], logic.serving_player, logic.is_game_running),
                            _ => bot_right.update(&physics, logic.touches[1], logic.serving_player, logic.is_game_running),
                        };
                        (p1, p2)
                    };

                    // Fixed-timestep simulation loop (75 Hz)
                    physics_accumulator += frame_dt;
                    while physics_accumulator >= PHYSICS_DT {
                        physics_accumulator -= PHYSICS_DT;

                        physics.step(
                            p1_input,
                            p2_input,
                            logic.is_ball_valid,
                            logic.is_game_running,
                        );
                        logic.step(PHYSICS_DT);

                        let events = physics.take_events();

                        // Send authoritative state to network client if hosting
                        if let Some(ref mut host) = net_host {
                            host.send_game_state(&physics, &logic, &events);
                        }

                        // Process collision events
                        for event in events {
                            match event {
                                MatchEvent::BallHitBlob { player, intensity } => {
                                    logic.on_ball_hits_blob(player);
                                    vfx.spawn_hit_shockwave(physics.ball.pos, intensity);
                                    audio.play_ball_hit(intensity);
                                    net_renderer.trigger_hit(intensity * 0.3);
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
                                    net_renderer.trigger_hit(0.7);
                                    audio.play_ball_hit(0.3);
                                }
                                MatchEvent::BallHitNetTop => {
                                    logic.on_ball_hits_net(PlayerSide::Left);
                                    net_renderer.trigger_hit(1.0);
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
                                bot_left.reset_rally();
                                bot_right.reset_rally();
                                audio.play_whistle();
                            }
                        }
                    }
                }

                // Update visual effects & net dynamics
                net_renderer.update(frame_dt);
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
                            bot_left.reset_rally();
                            bot_right.reset_rally();
                            state = AppState::Playing;
                            audio.play_whistle();
                        }
                        2 => {
                            net_host = None;
                            net_client = None;
                            state = AppState::MainMenu;
                        }
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

        // World camera
        set_camera(&Camera2D {
            target: cam_target,
            zoom: Vec2::new(2.0 / cam_w, 2.0 / cam_h),
            ..Default::default()
        });

        // 1. Render Arena Environment
        render_arena(arena, global_time);

        // 2. Render Volleyball Net
        net_renderer.render(global_time);

        // 3. Render Procedural Jelly Blobbies
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

        // 6. UI Overlay Camera
        set_default_camera();
        set_camera(&Camera2D {
            target: Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
            zoom: Vec2::new(2.0 / cam_w, 2.0 / cam_h),
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
            AppState::HostLobby => {
                let is_connected = net_host.as_ref().map_or(false, |h| h.is_client_connected);
                render_host_lobby(DEFAULT_PORT, &local_ip, is_connected, global_time);
            }
            AppState::JoinScreen => {
                let is_connecting = net_client.as_ref().map_or(false, |c| c.is_connecting);
                let err = net_client.as_ref().and_then(|c| c.error_message.as_deref());
                render_join_screen(&join_ip_input, is_connecting, err, global_time);
            }
            AppState::Playing => {
                let left_title = if net_host.is_some() {
                    "HOST (P1)"
                } else if net_client.is_some() {
                    "HOST (P1)"
                } else {
                    match game_mode {
                        GameMode::BotVsBot => "CPU 1",
                        _ => "PLAYER 1",
                    }
                };

                let right_title = if net_host.is_some() {
                    "CLIENT (P2)".to_string()
                } else if net_client.is_some() {
                    "YOU (P2)".to_string()
                } else {
                    match game_mode {
                        GameMode::SinglePlayer => format!("CPU ({})", difficulty.name()),
                        GameMode::TwoPlayerLocal => "PLAYER 2".to_string(),
                        GameMode::BotVsBot => format!("CPU 2 ({})", difficulty.name()),
                        _ => "PLAYER 2".to_string(),
                    }
                };

                render_hud(
                    &logic,
                    left_title,
                    &right_title,
                    left_theme,
                    right_theme,
                    global_time,
                );

                // Render in-game chat overlay
                render_chat_overlay(&chat_log, is_typing_chat, &chat_input, global_time);
            }
            AppState::Paused => {
                render_hud(&logic, "PLAYER 1", "PLAYER 2", left_theme, right_theme, global_time);
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
