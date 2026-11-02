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
mod config;

use constants::*;
use physics::{PhysicWorld, MatchEvent, PlayerSide};
use rules::{GameLogic, RuleMode};
use ai::{BotController, AiDifficulty};
use audio::AudioManager;
use vfx::VfxManager;
use net::{NetworkHost, NetworkClient, NetEvent, ServerEntry, query_master_server, get_local_ip, DEFAULT_PORT};
use render::blob::{render_blobby, BlobTheme};
use render::ball::{render_ball, BallTrail};
use render::net::NetRenderer;
use render::arenas::{render_arena, ArenaType};
use render::ui::*;
use config::UserConfig;

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    MainMenu,
    PlayerSetup { is_two_player: bool, active_field: usize },
    HostLobby,
    JoinScreen,
    ServerBrowser { selected_idx: usize, servers: Vec<ServerEntry>, is_refreshing: bool },
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
    let mut config = UserConfig::load();

    let mut audio = AudioManager::new().await;
    let mut vfx = VfxManager::new();
    let mut net_renderer = NetRenderer::new();
    let mut ball_trail = BallTrail::new();

    // Player profiles & identities
    let mut player1_name = config.player1_name.clone();
    let mut player2_name = config.player2_name.clone();

    // Match configurations from persistent user config
    let mut game_mode = GameMode::SinglePlayer;
    let mut difficulty = AiDifficulty::from_index(config.difficulty_index);
    let mut rule_mode = RuleMode::from_index(config.rule_mode_index);
    let mut score_to_win = config.score_to_win;
    let mut arena = ArenaType::from_index(config.arena_index);
    let mut left_theme = BlobTheme::from_index(config.left_theme_index);
    let mut right_theme = BlobTheme::from_index(config.right_theme_index);
    let mut fullscreen = config.fullscreen;

    if fullscreen {
        macroquad::window::set_fullscreen(true);
    }
    if config.sound_muted {
        audio.is_muted = true;
    }

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

    // Helper closure to save configuration
    let save_config = |cfg: &mut UserConfig,
                       p1: &str,
                       p2: &str,
                       fs: bool,
                       muted: bool,
                       score: i32,
                       ar: ArenaType,
                       diff: AiDifficulty,
                       rm: RuleMode,
                       lt: BlobTheme,
                       rt: BlobTheme| {
        cfg.player1_name = p1.to_string();
        cfg.player2_name = p2.to_string();
        cfg.fullscreen = fs;
        cfg.sound_muted = muted;
        cfg.score_to_win = score;
        cfg.arena_index = ar.to_index();
        cfg.difficulty_index = diff.to_index();
        cfg.rule_mode_index = rm.to_index();
        cfg.left_theme_index = lt.to_index();
        cfg.right_theme_index = rt.to_index();
        cfg.save();
    };

    loop {
        let frame_dt = get_frame_time().min(0.05);
        global_time += frame_dt;

        // Viewport coordinate transformation for virtual 800x600 resolution
        let screen_w = screen_width();
        let screen_h = screen_height();
        let aspect = screen_w / screen_h;
        let (cam_w, cam_h) = if aspect >= VIRTUAL_WIDTH / VIRTUAL_HEIGHT {
            (VIRTUAL_HEIGHT * aspect, VIRTUAL_HEIGHT)
        } else {
            (VIRTUAL_WIDTH, VIRTUAL_WIDTH / aspect)
        };
        let cam_target = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5);
        let cam_x0 = cam_target.x - cam_w * 0.5;
        let cam_y0 = cam_target.y - cam_h * 0.5;
        let (mx, my) = mouse_position();
        let virt_mouse = Vec2::new(
            cam_x0 + (mx / screen_w) * cam_w,
            cam_y0 + (my / screen_h) * cam_h,
        );

        // Global Fullscreen toggle (F11 or Alt+Enter)
        let alt_enter = (is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt))
            && is_key_pressed(KeyCode::Enter);
        if is_key_pressed(KeyCode::F11) || alt_enter {
            fullscreen = !fullscreen;
            macroquad::window::set_fullscreen(fullscreen);
            config.fullscreen = fullscreen;
            config.save();
        }

        // Global Mute toggle (M)
        if !is_typing_chat && state != AppState::JoinScreen && is_key_pressed(KeyCode::M) {
            audio.toggle_mute();
            config.sound_muted = audio.is_muted;
            config.save();
        }

        match state {
            AppState::MainMenu => {
                let layouts = get_main_menu_layout();

                // Keyboard Menu Navigation (Up / Down)
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    menu_selected = if menu_selected == 0 { 9 } else { menu_selected - 1 };
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    menu_selected = (menu_selected + 1) % 10;
                }

                // Helper to cycle a specific setting (forward or backward)
                let mut cycle_setting = |idx: usize, forward: bool| {
                    match idx {
                        1 => game_mode = if forward { game_mode.next() } else { game_mode.prev() },
                        2 => {
                            // Player Profile: click opens player profile setup
                        }
                        3 => {
                            difficulty = if forward { difficulty.next() } else { difficulty.prev() };
                            bot_left.difficulty = difficulty;
                            bot_right.difficulty = difficulty;
                        }
                        4 => {
                            rule_mode = if forward { rule_mode.next() } else { rule_mode.prev() };
                            logic.rule_mode = rule_mode;
                        }
                        5 => {
                            score_to_win = match score_to_win {
                                5 => if forward { 10 } else { 21 },
                                10 => if forward { 15 } else { 5 },
                                15 => if forward { 21 } else { 10 },
                                _ => if forward { 5 } else { 15 },
                            };
                            logic.score_to_win = score_to_win;
                        }
                        6 => arena = if forward { arena.next() } else { arena.prev() },
                        7 => left_theme = if forward { left_theme.next() } else { left_theme.prev() },
                        8 => right_theme = if forward { right_theme.next() } else { right_theme.prev() },
                        9 => {
                            fullscreen = !fullscreen;
                            macroquad::window::set_fullscreen(fullscreen);
                        }
                        _ => {}
                    }
                    save_config(
                        &mut config,
                        &player1_name,
                        &player2_name,
                        fullscreen,
                        audio.is_muted,
                        score_to_win,
                        arena,
                        difficulty,
                        rule_mode,
                        left_theme,
                        right_theme,
                    );
                };

                // Keyboard Left/Right to change value of highlighted row
                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    cycle_setting(menu_selected, false);
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    cycle_setting(menu_selected, true);
                }

                // Mouse click handler
                let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
                let mut action_start_match = false;
                let mut action_open_profile = false;

                if mouse_clicked {
                    for layout in &layouts {
                        let i = layout.index;
                        // 1. Check Left Arrow Button
                        if let Some(l_rect) = layout.left_arrow_rect {
                            if l_rect.contains(virt_mouse) {
                                menu_selected = i;
                                cycle_setting(i, false);
                                audio.play_ball_hit(0.2);
                                continue;
                            }
                        }
                        // 2. Check Right Arrow Button
                        if let Some(r_rect) = layout.right_arrow_rect {
                            if r_rect.contains(virt_mouse) {
                                menu_selected = i;
                                cycle_setting(i, true);
                                audio.play_ball_hit(0.2);
                                continue;
                            }
                        }
                        // 3. Check Row Body (outside arrow buttons)
                        if layout.row_rect.contains(virt_mouse) {
                            menu_selected = i;
                            if i == 2 {
                                action_open_profile = true;
                            } else {
                                // Clicking the center label/body of ANY setting row confirms and starts match!
                                action_start_match = true;
                            }
                            break;
                        }
                    }
                }

                // Keyboard Space or Enter confirms and starts match (or opens profile if row 2)
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if menu_selected == 2 {
                        action_open_profile = true;
                    } else {
                        action_start_match = true;
                    }
                }

                if action_open_profile {
                    state = AppState::PlayerSetup {
                        is_two_player: game_mode == GameMode::TwoPlayerLocal,
                        active_field: 0,
                    };
                    audio.play_ball_hit(0.3);
                } else if action_start_match {
                    save_config(
                        &mut config,
                        &player1_name,
                        &player2_name,
                        fullscreen,
                        audio.is_muted,
                        score_to_win,
                        arena,
                        difficulty,
                        rule_mode,
                        left_theme,
                        right_theme,
                    );

                    match game_mode {
                        GameMode::TwoPlayerLocal => {
                            // Prompt player 2 name on starting 2-player mode
                            state = AppState::PlayerSetup {
                                is_two_player: true,
                                active_field: 1, // Default focus on player 2
                            };
                            audio.play_ball_hit(0.3);
                        }
                        GameMode::HostOnline => {
                            if let Ok(host) = NetworkHost::bind(DEFAULT_PORT, &player1_name, arena.name()) {
                                net_host = Some(host);
                                state = AppState::HostLobby;
                            }
                        }
                        GameMode::JoinOnline => {
                            state = AppState::JoinScreen;
                        }
                        GameMode::ServerBrowser => {
                            let servers = query_master_server();
                            state = AppState::ServerBrowser {
                                selected_idx: 0,
                                servers,
                                is_refreshing: false,
                            };
                        }
                        _ => {
                            // SinglePlayer or BotVsBot: start immediately!
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
            }

            AppState::PlayerSetup { is_two_player, mut active_field } => {
                // Character input
                while let Some(c) = get_char_pressed() {
                    if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' {
                        if active_field == 0 && player1_name.len() < 16 {
                            player1_name.push(c);
                        } else if active_field == 1 && player2_name.len() < 16 {
                            player2_name.push(c);
                        }
                    }
                }

                if is_key_pressed(KeyCode::Backspace) {
                    if active_field == 0 {
                        player1_name.pop();
                    } else {
                        player2_name.pop();
                    }
                }

                // Switch fields via Tab or Up/Down
                if is_two_player && (is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Down)) {
                    active_field = 1 - active_field;
                    state = AppState::PlayerSetup { is_two_player, active_field };
                }

                // Confirm and start match on Enter
                if is_key_pressed(KeyCode::Enter) {
                    if player1_name.trim().is_empty() {
                        player1_name = "Player 1".to_string();
                    }
                    if player2_name.trim().is_empty() {
                        player2_name = "Player 2".to_string();
                    }
                    save_config(
                        &mut config,
                        &player1_name,
                        &player2_name,
                        fullscreen,
                        audio.is_muted,
                        score_to_win,
                        arena,
                        difficulty,
                        rule_mode,
                        left_theme,
                        right_theme,
                    );

                    // Launch match
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

                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }
            }

            AppState::ServerBrowser { ref mut selected_idx, ref mut servers, ref mut is_refreshing } => {
                let num_servers = servers.len();
                if num_servers > 0 {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        *selected_idx = if *selected_idx == 0 { num_servers - 1 } else { *selected_idx - 1 };
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        *selected_idx = (*selected_idx + 1) % num_servers;
                    }
                }

                if is_key_pressed(KeyCode::R) {
                    *is_refreshing = true;
                    *servers = query_master_server();
                    *is_refreshing = false;
                }

                if is_key_pressed(KeyCode::Enter) && num_servers > 0 {
                    let s = &servers[*selected_idx];
                    join_ip_input = format!("{}:{}", s.host, s.port);
                    let mut client = NetworkClient::new();
                    client.connect(&join_ip_input);
                    net_client = Some(client);
                    state = AppState::JoinScreen;
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }
            }

            AppState::HostLobby => {
                if is_key_pressed(KeyCode::Escape) {
                    net_host = None;
                    state = AppState::MainMenu;
                } else if let Some(ref mut host) = net_host {
                    host.update(global_time);
                    if host.is_client_connected {
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
                        let my_name = player1_name.as_str();
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
                    if is_key_pressed(KeyCode::T) || is_key_pressed(KeyCode::C) {
                        is_typing_chat = true;
                        chat_input.clear();
                    }

                    if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::P) {
                        state = AppState::Paused;
                        pause_selected = 0;
                    }
                }

                // Network Client update (Client Side)
                if let Some(ref mut client) = net_client {
                    client.update();

                    let my_input = input::read_player1_input(true);
                    client.send_input(my_input);

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

                    for (sender, text) in client.new_chat_messages.drain(..) {
                        chat_log.push((sender, text, global_time));
                        audio.play_score();
                    }

                    if !client.is_connected {
                        state = AppState::MainMenu;
                    }
                }
                // Host / Local authoritative simulation
                else {
                    let (p1_input, p2_input) = if let Some(ref mut host) = net_host {
                        host.update(global_time);
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

                        if let Some(ref mut host) = net_host {
                            host.send_game_state(&physics, &logic, &events);
                        }

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
                        0 => state = AppState::Playing,
                        1 => {
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

        let shake = vfx.get_shake_offset();
        let cam_target = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5) + shake;

        // 1. World Camera for Arena, Blobs, Ball & Particles
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
                    &player1_name,
                    difficulty,
                    rule_mode,
                    score_to_win,
                    arena,
                    left_theme,
                    right_theme,
                    fullscreen,
                    menu_selected,
                    Some(virt_mouse),
                    global_time,
                );
            }
            AppState::PlayerSetup { is_two_player, active_field } => {
                render_player_setup(&player1_name, &player2_name, is_two_player, active_field, global_time);
            }
            AppState::ServerBrowser { selected_idx, ref servers, is_refreshing } => {
                render_server_browser(servers, selected_idx, is_refreshing, global_time);
            }
            AppState::HostLobby => {
                let is_connected = net_host.as_ref().map_or(false, |h| h.is_client_connected);
                let room_name = net_host.as_ref().map_or("Match", |h| &h.room_name);
                let host_local_ip = net_host.as_ref().map_or(local_ip.as_str(), |h| &h.local_ip);
                let public_ep = net_host.as_ref().and_then(|h| h.public_endpoint.as_deref());
                render_host_lobby(room_name, DEFAULT_PORT, host_local_ip, public_ep, is_connected, global_time);
            }
            AppState::JoinScreen => {
                let is_connecting = net_client.as_ref().map_or(false, |c| c.is_connecting);
                let err = net_client.as_ref().and_then(|c| c.error_message.as_deref());
                render_join_screen(&join_ip_input, is_connecting, err, global_time);
            }
            AppState::Playing => {
                let left_title = if net_host.is_some() {
                    format!("{} (Host)", player1_name)
                } else if net_client.is_some() {
                    "Host (P1)".to_string()
                } else {
                    match game_mode {
                        GameMode::BotVsBot => "CPU 1".to_string(),
                        _ => player1_name.clone(),
                    }
                };

                let right_title = if net_host.is_some() {
                    "Client (P2)".to_string()
                } else if net_client.is_some() {
                    format!("{} (You)", player1_name)
                } else {
                    match game_mode {
                        GameMode::SinglePlayer => format!("CPU ({})", difficulty.name()),
                        GameMode::TwoPlayerLocal => player2_name.clone(),
                        GameMode::BotVsBot => format!("CPU 2 ({})", difficulty.name()),
                        _ => player2_name.clone(),
                    }
                };

                render_hud(
                    &logic,
                    &left_title,
                    &right_title,
                    left_theme,
                    right_theme,
                    global_time,
                );

                render_chat_overlay(&chat_log, is_typing_chat, &chat_input, global_time);
            }
            AppState::Paused => {
                let r_name = if game_mode == GameMode::SinglePlayer {
                    format!("CPU ({})", difficulty.name())
                } else {
                    player2_name.clone()
                };
                render_hud(&logic, &player1_name, &r_name, left_theme, right_theme, global_time);
                render_pause_menu(pause_selected);
            }
            AppState::Victory(winner) => {
                let winner_name = match winner {
                    PlayerSide::Left => &player1_name,
                    PlayerSide::Right => {
                        if game_mode == GameMode::SinglePlayer {
                            "Computer"
                        } else {
                            &player2_name
                        }
                    }
                };
                render_victory_screen(winner, winner_name, &logic, global_time);
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
