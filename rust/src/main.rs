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
mod glicko;

use constants::*;
use physics::{PhysicWorld, MatchEvent, PlayerSide};
use rules::{GameLogic, RuleMode};
use ai::{BotController, AiDifficulty};
use audio::AudioManager;
use vfx::VfxManager;
use net::{NetworkHost, NetworkClient, NetEvent, ServerEntry, query_master_server, get_local_ip, DEFAULT_PORT, QuickMatchAgent, MatchmakingStatus, LeaderboardEntry, fetch_leaderboard, submit_leaderboard_rating};
use render::blob::{render_blobby, BlobTheme};
use render::ball::{render_ball, BallTrail};
use render::net::NetRenderer;
use render::arenas::{render_arena, ArenaType, render_court_size_visualizer};
use render::ui::*;
use config::{UserConfig, EloSearchRange, country_badge, country_index, country_code_at, AVAILABLE_COUNTRIES};
use glicko::GlickoRating;

#[derive(Debug, Clone, PartialEq)]
enum AppState {
    MainMenu,
    ExitConfirm,
    Leaderboard { period_idx: usize, entries: Vec<LeaderboardEntry>, is_refreshing: bool },
    PlayerSetup { is_two_player: bool, active_field: usize },
    HostLobby,
    JoinScreen,
    ServerBrowser { selected_idx: usize, servers: Vec<ServerEntry>, is_refreshing: bool },
    MatchmakingQueue,
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
    // High-resolution TrueType typography (replaces Macroquad's low-res ProggyClean bitmap)
    let font_bytes = include_bytes!("../assets/fonts/Roboto-Bold.ttf");
    if let Ok(mut font) = load_ttf_font_from_bytes(font_bytes) {
        font.set_filter(miniquad::FilterMode::Linear);
        set_default_font(font);
    }

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

    // Online Networking & Matchmaking State
    let mut net_host: Option<NetworkHost> = None;
    let mut net_client: Option<NetworkClient> = None;
    let mut quick_match_agent: Option<QuickMatchAgent> = None;
    let mut active_match_opponent: Option<(String, f64, f64, String)> = None; // (name, rating, rd, country)
    let mut last_rating_update: Option<(f64, f64, &'static str)> = None; // (old_r, new_r, rank_title)
    let mut join_ip_input = format!("127.0.0.1:{}", DEFAULT_PORT);
    let local_ip = get_local_ip();

    // In-game Chat State
    let mut chat_log: Vec<(String, String, f32)> = Vec::new();
    let mut is_typing_chat = false;
    let mut chat_input = String::new();

    let mut physics_accumulator = 0.0;
    let mut global_time = 0.0;
    let mut online_serve_countdown: Option<f32> = None;
    let mut last_point_scorer: Option<PlayerSide> = None;
    let mut cheer_timer: f32 = 0.0;
    let mut cam_target_y = 300.0 - 35.0;
    let mut cam_zoom_factor = 1.0;
    let mut is_dragging_court_slider = false;
    let mut country_search_buf = String::new();
    let mut country_search_time = 0.0f32;

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
        cheer_timer = (cheer_timer - frame_dt).max(0.0);
        if let Some(ref mut cd) = online_serve_countdown {
            *cd -= frame_dt;
            if *cd <= 0.0 {
                online_serve_countdown = None;
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            is_dragging_court_slider = false;
        }

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
                let items = render::ui::get_menu_items_for_mode(game_mode);
                let layouts = render::ui::get_main_menu_layout(&items);

                if menu_selected >= items.len() {
                    menu_selected = items.len().saturating_sub(1);
                }

                // Keyboard Menu Navigation (Up / Down)
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    menu_selected = if menu_selected == 0 { items.len().saturating_sub(1) } else { menu_selected - 1 };
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    menu_selected = (menu_selected + 1) % items.len().max(1);
                }

                // Macro to cycle a specific setting (forward or backward) without closure borrow conflicts
                macro_rules! cycle_setting {
                    ($item:expr, $forward:expr) => {
                        match $item {
                            render::ui::MenuItem::GameMode => {
                                game_mode = if $forward { game_mode.next() } else { game_mode.prev() };
                            }
                            render::ui::MenuItem::PlayerProfile => {}
                            render::ui::MenuItem::AiDifficulty => {
                                difficulty = if $forward { difficulty.next() } else { difficulty.prev() };
                                bot_left.difficulty = difficulty;
                                bot_right.difficulty = difficulty;
                            }
                            render::ui::MenuItem::MatchmakingRange => {
                                let cur_range = EloSearchRange::from_index(config.elo_search_range);
                                let new_range = if $forward { cur_range.next() } else { cur_range.prev() };
                                config.elo_search_range = new_range.to_index();
                            }
                            render::ui::MenuItem::Rules => {
                                rule_mode = if $forward { rule_mode.next() } else { rule_mode.prev() };
                                logic.rule_mode = rule_mode;
                            }
                            render::ui::MenuItem::PointsToWin => {
                                score_to_win = match score_to_win {
                                    5 => if $forward { 10 } else { 21 },
                                    10 => if $forward { 15 } else { 5 },
                                    15 => if $forward { 21 } else { 10 },
                                    _ => if $forward { 5 } else { 15 },
                                };
                                logic.score_to_win = score_to_win;
                            }
                            render::ui::MenuItem::CourtSize => {
                                let step = 0.05;
                                if $forward {
                                    config.court_width_factor = (config.court_width_factor + step).min(1.0);
                                } else {
                                    config.court_width_factor = (config.court_width_factor - step).max(0.0);
                                }
                                config.court_width_factor = (config.court_width_factor * 20.0).round() / 20.0;
                                config.court_size = (config.court_width_factor * 3.0).round() as usize;
                            }
                            render::ui::MenuItem::Arena => arena = if $forward { arena.next() } else { arena.prev() },
                            render::ui::MenuItem::P1Color => {
                                left_theme = if $forward { left_theme.next() } else { left_theme.prev() };
                                if left_theme == right_theme {
                                    right_theme = right_theme.next();
                                }
                            }
                            render::ui::MenuItem::P2Color => {
                                right_theme = if $forward { right_theme.next() } else { right_theme.prev() };
                                if right_theme == left_theme {
                                    right_theme = right_theme.next();
                                }
                            }
                            render::ui::MenuItem::HighBallCamera => {
                                let cur_cam = crate::config::HighBallCamera::from_index(config.high_ball_camera);
                                let new_cam = if $forward { cur_cam.next() } else { cur_cam.prev() };
                                config.high_ball_camera = new_cam.to_index();
                            }
                            render::ui::MenuItem::Fullscreen => {
                                fullscreen = !fullscreen;
                                macroquad::window::set_fullscreen(fullscreen);
                            }
                            render::ui::MenuItem::StartMatch => {}
                        }
                        let cur_len = render::ui::get_menu_items_for_mode(game_mode).len();
                        if menu_selected >= cur_len {
                            menu_selected = cur_len.saturating_sub(1);
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
                }

                // Keyboard Left/Right to change value of highlighted row
                if !items.is_empty() {
                    let sel_item = items[menu_selected];
                    if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                        cycle_setting!(sel_item, false);
                    }
                    if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                        cycle_setting!(sel_item, true);
                    }
                }

                // Slider dragging update
                if is_dragging_court_slider && is_mouse_button_down(MouseButton::Left) {
                    if let Some(layout) = layouts.iter().find(|l| l.item == render::ui::MenuItem::CourtSize) {
                        if let Some(s_rect) = layout.slider_rect {
                            let factor = ((virt_mouse.x - s_rect.x) / s_rect.w).clamp(0.0, 1.0);
                            config.court_width_factor = (factor * 100.0).round() / 100.0;
                            config.court_size = (config.court_width_factor * 3.0).round() as usize;
                            config.save();
                        }
                    }
                }

                // Mouse click handler
                let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
                let mut action_start_match = false;
                let mut action_open_profile = false;

                if mouse_clicked {
                    for layout in &layouts {
                        let i = layout.index;
                        let item = layout.item;
                        // 1. Check Left Arrow Button
                        if let Some(l_rect) = layout.left_arrow_rect {
                            if l_rect.contains(virt_mouse) {
                                menu_selected = i;
                                cycle_setting!(item, false);
                                audio.play_ball_hit(0.2);
                                continue;
                            }
                        }
                        // 2. Check Right Arrow Button
                        if let Some(r_rect) = layout.right_arrow_rect {
                            if r_rect.contains(virt_mouse) {
                                menu_selected = i;
                                cycle_setting!(item, true);
                                audio.play_ball_hit(0.2);
                                continue;
                            }
                        }
                        // 3. Check Slider Track (Schieberegler)
                        if let Some(s_rect) = layout.slider_rect {
                            if s_rect.contains(virt_mouse) {
                                menu_selected = i;
                                is_dragging_court_slider = true;
                                let factor = ((virt_mouse.x - s_rect.x) / s_rect.w).clamp(0.0, 1.0);
                                config.court_width_factor = (factor * 100.0).round() / 100.0;
                                config.court_size = (config.court_width_factor * 3.0).round() as usize;
                                config.save();
                                audio.play_ball_hit(0.15);
                                continue;
                            }
                        }
                        // 4. Check Row Body (outside arrow buttons and slider)
                        if layout.row_rect.contains(virt_mouse) {
                            menu_selected = i;
                            if item == render::ui::MenuItem::PlayerProfile {
                                action_open_profile = true;
                            } else {
                                // Clicking the center label/body of ANY setting row confirms and starts match!
                                action_start_match = true;
                            }
                            break;
                        }
                    }
                }

                // Keyboard Space or Enter confirms and starts match (or opens profile if profile row)
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if menu_selected < items.len() && items[menu_selected] == render::ui::MenuItem::PlayerProfile {
                        action_open_profile = true;
                    } else {
                        action_start_match = true;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::ExitConfirm;
                    audio.play_ball_hit(0.2);
                }

                if is_key_pressed(KeyCode::L) {
                    let entries = fetch_leaderboard(
                        config.matchmaking_registry_url.as_deref(),
                        "month",
                        &player1_name,
                        config.glicko.rating,
                        config.matches_won,
                        config.matches_played,
                    );
                    state = AppState::Leaderboard {
                        period_idx: 1,
                        entries,
                        is_refreshing: false,
                    };
                    audio.play_ball_hit(0.3);
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

                    if left_theme == right_theme {
                        right_theme = right_theme.next();
                    }

                    match game_mode {
                        GameMode::QuickMatch => {
                            score_to_win = 15;
                            logic.score_to_win = 15;
                            quick_match_agent = Some(QuickMatchAgent::new(
                                &player1_name,
                                &config.country_code,
                                config.glicko.rating,
                                config.glicko.deviation,
                                arena.to_index(),
                                config.elo_search_range,
                                config.matchmaking_registry_url.clone(),
                                global_time,
                            ));
                            state = AppState::MatchmakingQueue;
                            audio.play_ball_hit(0.3);
                        }
                        GameMode::SinglePlayer => {
                            let (cpu_r, cpu_rd) = GlickoRating::ai_baseline(difficulty);
                            active_match_opponent = Some((format!("CPU ({})", difficulty.name()), cpu_r, cpu_rd, "".to_string()));
                            logic = GameLogic::new(rule_mode, score_to_win);
                            let court_w = config.get_court_width(aspect);
                            physics = PhysicWorld::new_with_width(court_w);
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
                        GameMode::TwoPlayerLocal => {
                            active_match_opponent = None;
                            // Prompt player 2 name on starting 2-player mode
                            state = AppState::PlayerSetup {
                                is_two_player: true,
                                active_field: 1, // Default focus on player 2
                            };
                            audio.play_ball_hit(0.3);
                        }
                        GameMode::HostOnline => {
                            active_match_opponent = None;
                            if let Ok(host) = NetworkHost::bind(DEFAULT_PORT, &player1_name, &config.country_code, arena.name()) {
                                net_host = Some(host);
                                state = AppState::HostLobby;
                            }
                        }
                        GameMode::JoinOnline => {
                            active_match_opponent = None;
                            state = AppState::JoinScreen;
                        }
                        GameMode::ServerBrowser => {
                            active_match_opponent = None;
                            let servers = query_master_server();
                            state = AppState::ServerBrowser {
                                selected_idx: 0,
                                servers,
                                is_refreshing: false,
                            };
                        }
                        GameMode::BotVsBot => {
                            active_match_opponent = None;
                            logic = GameLogic::new(rule_mode, score_to_win);
                            let court_w = config.get_court_width(aspect);
                            physics = PhysicWorld::new_with_width(court_w);
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

            AppState::ExitConfirm => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Y) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Backspace) {
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    let card_w = 480.0;
                    let card_h = 240.0;
                    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
                    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;
                    let btn1_rect = Rect::new(card_x + 40.0, card_y + 125.0, 185.0, 44.0);
                    let btn2_rect = Rect::new(card_x + 255.0, card_y + 125.0, 185.0, 44.0);

                    if btn1_rect.contains(virt_mouse) {
                        std::process::exit(0);
                    } else if btn2_rect.contains(virt_mouse) {
                        state = AppState::MainMenu;
                        audio.play_ball_hit(0.2);
                    }
                }
            }

            AppState::Leaderboard { ref mut period_idx, ref mut entries, ref mut is_refreshing } => {
                let mut period_changed = false;

                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    *period_idx = if *period_idx == 0 { 2 } else { *period_idx - 1 };
                    period_changed = true;
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    *period_idx = (*period_idx + 1) % 3;
                    period_changed = true;
                }
                if is_key_pressed(KeyCode::Key1) {
                    *period_idx = 0;
                    period_changed = true;
                }
                if is_key_pressed(KeyCode::Key2) {
                    *period_idx = 1;
                    period_changed = true;
                }
                if is_key_pressed(KeyCode::Key3) {
                    *period_idx = 2;
                    period_changed = true;
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    let _card_w = 720.0;
                    let card_h = 490.0;
                    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;
                    let tab_w = 170.0;
                    let tab_h = 28.0;
                    let tab_spacing = 20.0;
                    let start_tab_x = (VIRTUAL_WIDTH - (tab_w * 3.0 + tab_spacing * 2.0)) * 0.5;
                    let tab_y = card_y + 74.0;

                    for i in 0..3 {
                        let tx = start_tab_x + i as f32 * (tab_w + tab_spacing);
                        let tab_rect = Rect::new(tx, tab_y, tab_w, tab_h);
                        if tab_rect.contains(virt_mouse) && *period_idx != i {
                            *period_idx = i;
                            period_changed = true;
                            audio.play_ball_hit(0.2);
                            break;
                        }
                    }
                }

                if period_changed || is_key_pressed(KeyCode::R) {
                    *is_refreshing = true;
                    let p_str = match *period_idx {
                        0 => "week",
                        1 => "month",
                        _ => "year",
                    };
                    *entries = fetch_leaderboard(
                        config.matchmaking_registry_url.as_deref(),
                        p_str,
                        &player1_name,
                        config.glicko.rating,
                        config.matches_won,
                        config.matches_played,
                    );
                    *is_refreshing = false;
                    audio.play_ball_hit(0.2);
                }

                if is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
                }
            }

            AppState::PlayerSetup { is_two_player, mut active_field } => {
                let country_field_idx = if is_two_player { 2 } else { 1 };
                let total_fields = if is_two_player { 3 } else { 2 };

                // Character input
                while let Some(c) = get_char_pressed() {
                    if active_field == country_field_idx && c.is_alphabetic() {
                        let now = global_time;
                        // If typed quickly (< 0.85s apart), accumulate sequence (e.g. "g-e-r" -> Germany)
                        // If typed slowly (>= 0.85s apart), start fresh single-key jump ("G" -> Gabon, "E" -> Ecuador, "R" -> Romania)
                        if now - country_search_time > 0.85 {
                            country_search_buf.clear();
                        }
                        country_search_buf.push(c.to_ascii_uppercase());
                        country_search_time = now;

                        if let Some(idx) = config::find_country_by_search(&country_search_buf) {
                            config.country_code = country_code_at(idx).to_string();
                            audio.play_ball_hit(0.2);
                        } else {
                            // If multi-char sequence didn't match, fallback to just this letter
                            country_search_buf = c.to_ascii_uppercase().to_string();
                            if let Some(idx) = config::find_country_by_search(&country_search_buf) {
                                config.country_code = country_code_at(idx).to_string();
                                audio.play_ball_hit(0.2);
                            }
                        }
                    } else if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' {
                        if active_field == 0 && player1_name.len() < 16 {
                            player1_name.push(c);
                        } else if is_two_player && active_field == 1 && player2_name.len() < 16 {
                            player2_name.push(c);
                        }
                    }
                }

                if is_key_pressed(KeyCode::Backspace) {
                    if active_field == 0 {
                        player1_name.pop();
                    } else if is_two_player && active_field == 1 {
                        player2_name.pop();
                    } else if active_field == country_field_idx {
                        if !country_search_buf.is_empty() {
                            country_search_buf.pop();
                            if let Some(idx) = config::find_country_by_search(&country_search_buf) {
                                config.country_code = country_code_at(idx).to_string();
                            }
                        } else {
                            config.country_code = "".to_string();
                        }
                        audio.play_ball_hit(0.15);
                    }
                }

                // Country cycle on Left / Right when country field is active
                if active_field == country_field_idx {
                    let mut c_idx = country_index(&config.country_code);
                    if is_key_pressed(KeyCode::Left) {
                        country_search_buf.clear();
                        c_idx = (c_idx + AVAILABLE_COUNTRIES.len() - 1) % AVAILABLE_COUNTRIES.len();
                        config.country_code = country_code_at(c_idx).to_string();
                        audio.play_ball_hit(0.15);
                    }
                    if is_key_pressed(KeyCode::Right) {
                        country_search_buf.clear();
                        c_idx = (c_idx + 1) % AVAILABLE_COUNTRIES.len();
                        config.country_code = country_code_at(c_idx).to_string();
                        audio.play_ball_hit(0.15);
                    }
                }

                // Switch fields via Tab or Up/Down
                if is_key_pressed(KeyCode::Tab) || is_key_pressed(KeyCode::Down) {
                    country_search_buf.clear();
                    active_field = (active_field + 1) % total_fields;
                    state = AppState::PlayerSetup { is_two_player, active_field };
                }
                if is_key_pressed(KeyCode::Up) {
                    country_search_buf.clear();
                    active_field = (active_field + total_fields - 1) % total_fields;
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
                    let court_w = config.get_court_width(aspect);
                    physics = PhysicWorld::new_with_width(court_w);
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

            AppState::MatchmakingQueue => {
                let mut cancel_search = false;
                if let Some(ref mut agent) = quick_match_agent {
                    if let Some(offer) = agent.active_offer.clone() {
                        if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::Space) {
                            agent.accept_cross_arena(&offer.opponent_id);
                            audio.play_whistle();
                        } else if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
                            agent.decline_cross_arena();
                            audio.play_ball_hit(0.2);
                        }
                    } else {
                        if is_key_pressed(KeyCode::Escape) {
                            agent.cancel();
                            cancel_search = true;
                        } else if is_key_pressed(KeyCode::E) {
                            let new_range = agent.cycle_elo_range();
                            config.elo_search_range = new_range.to_index();
                            config.save();
                            audio.play_ball_hit(0.2);
                        } else if is_key_pressed(KeyCode::A) {
                            let next_ar = agent.cycle_arena(true);
                            arena = ArenaType::from_index(next_ar);
                            config.arena_index = next_ar;
                            config.save();
                            audio.play_ball_hit(0.2);
                        }
                    }

                    if !cancel_search {
                        match agent.poll(global_time) {
                            MatchmakingStatus::Searching { .. } => {}
                            MatchmakingStatus::CrossArenaOfferReceived(_) => {}
                            MatchmakingStatus::MatchedAsHost { opponent_name, opponent_country, opponent_rating, opponent_rd, agreed_arena } => {
                                audio.play_score();
                                arena = ArenaType::from_index(agreed_arena);
                                score_to_win = 15;
                                logic.score_to_win = 15;
                                if left_theme == right_theme {
                                    right_theme = right_theme.next();
                                }
                                if let Ok(host) = NetworkHost::bind(DEFAULT_PORT, &player1_name, &config.country_code, arena.name()) {
                                    net_host = Some(host);
                                    active_match_opponent = Some((opponent_name, opponent_rating, opponent_rd, opponent_country));
                                    state = AppState::HostLobby;
                                } else {
                                    state = AppState::MainMenu;
                                }
                                cancel_search = true;
                            }
                            MatchmakingStatus::MatchedAsClient { target_addr, opponent_name, opponent_country, opponent_rating, opponent_rd, agreed_arena } => {
                                audio.play_score();
                                arena = ArenaType::from_index(agreed_arena);
                                score_to_win = 15;
                                logic.score_to_win = 15;
                                if left_theme == right_theme {
                                    right_theme = right_theme.next();
                                }
                                let mut client = NetworkClient::new();
                                client.connect(&target_addr);
                                net_client = Some(client);
                                active_match_opponent = Some((opponent_name, opponent_rating, opponent_rd, opponent_country));
                                state = AppState::JoinScreen;
                                cancel_search = true;
                            }
                        }
                    }
                } else if is_key_pressed(KeyCode::Escape) {
                    state = AppState::MainMenu;
                }

                if cancel_search {
                    quick_match_agent = None;
                    if state == AppState::MatchmakingQueue {
                        state = AppState::MainMenu;
                    }
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

                    if let Some(winner) = logic.check_win() {
                        if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                            let score = if winner == PlayerSide::Right { 1.0 } else { 0.0 };
                            let old_r = config.glicko.rating;
                            let new_rating = config.glicko.update_match(opp_r, opp_rd, score);
                            config.matches_played += 1;
                            if score == 1.0 {
                                config.matches_won += 1;
                            }
                            config.glicko = new_rating;
                            config.save();
                            submit_leaderboard_rating(
                                config.matchmaking_registry_url.as_deref(),
                                "local_player",
                                &player1_name,
                                new_rating.rating,
                                new_rating.deviation,
                                config.matches_won,
                                config.matches_played,
                            );
                            last_rating_update = Some((old_r, new_rating.rating, new_rating.rank_title()));
                        } else {
                            last_rating_update = None;
                        }
                        state = AppState::Victory(winner);
                        vfx.spawn_confetti_celebration(Vec2::new(400.0, 200.0), 90);
                        audio.play_score();
                    }

                    if !client.is_connected {
                        if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                            let old_r = config.glicko.rating;
                            let new_rating = config.glicko.update_match(opp_r, opp_rd, 0.0);
                            config.matches_played += 1;
                            config.glicko = new_rating;
                            config.save();
                            submit_leaderboard_rating(
                                config.matchmaking_registry_url.as_deref(),
                                "local_player",
                                &player1_name,
                                new_rating.rating,
                                new_rating.deviation,
                                config.matches_won,
                                config.matches_played,
                            );
                            last_rating_update = Some((old_r, new_rating.rating, new_rating.rank_title()));
                        }
                        net_client = None;
                        state = AppState::MainMenu;
                    }
                }
                // Host / Local authoritative simulation
                else {
                    let (p1_input, p2_input) = if let Some(ref mut host) = net_host {
                        host.update(global_time);
                        if !host.is_client_connected && active_match_opponent.is_some() && logic.winning_player.is_none() {
                            // Opponent disconnected/forfeited - host wins!
                            if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                                let old_r = config.glicko.rating;
                                let new_rating = config.glicko.update_match(opp_r, opp_rd, 1.0);
                                config.matches_played += 1;
                                config.matches_won += 1;
                                config.glicko = new_rating;
                                config.save();
                                submit_leaderboard_rating(
                                    config.matchmaking_registry_url.as_deref(),
                                    "local_player",
                                    &player1_name,
                                    new_rating.rating,
                                    new_rating.deviation,
                                    config.matches_won,
                                    config.matches_played,
                                );
                                last_rating_update = Some((old_r, new_rating.rating, new_rating.rank_title()));
                            }
                            state = AppState::Victory(PlayerSide::Left);
                            audio.play_score();
                        }
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
                                    let was_valid = logic.is_ball_valid;
                                    logic.on_ball_hits_blob(player);
                                    vfx.spawn_hit_shockwave(physics.ball.pos, intensity);
                                    audio.play_ball_hit(intensity);
                                    net_renderer.trigger_hit(intensity * 0.3);
                                    if was_valid && !logic.is_ball_valid {
                                        audio.play_whistle();
                                        audio.play_score();
                                        last_point_scorer = Some(player.opponent());
                                        cheer_timer = 2.5;
                                    }
                                }
                                MatchEvent::BallHitGround { side } => {
                                    let was_valid = logic.is_ball_valid;
                                    logic.on_ball_hits_ground(side);
                                    vfx.spawn_sand_puff(Vec2::new(physics.ball.pos.x, GROUND_PLANE_HEIGHT_MAX), 8);
                                    if was_valid && !logic.is_ball_valid {
                                        audio.play_whistle();
                                        audio.play_score();
                                        last_point_scorer = Some(side.opponent());
                                        cheer_timer = 2.5;
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
                                MatchEvent::BallCrossedNet { new_side } => {
                                    logic.on_ball_crosses_net(new_side);
                                }
                            }
                        }

                        if logic.can_start_round(&physics) {
                            if let Some(winner) = logic.winning_player {
                                if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                                    let score = if winner == PlayerSide::Left { 1.0 } else { 0.0 };
                                    let old_r = config.glicko.rating;
                                    let new_rating = config.glicko.update_match(opp_r, opp_rd, score);
                                    config.matches_played += 1;
                                    if score == 1.0 {
                                        config.matches_won += 1;
                                    }
                                    config.glicko = new_rating;
                                    config.save();
                                    submit_leaderboard_rating(
                                        config.matchmaking_registry_url.as_deref(),
                                        "local_player",
                                        &player1_name,
                                        new_rating.rating,
                                        new_rating.deviation,
                                        config.matches_won,
                                        config.matches_played,
                                    );
                                    last_rating_update = Some((old_r, new_rating.rating, new_rating.rank_title()));
                                } else {
                                    last_rating_update = None;
                                }

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
                                if net_host.is_some() || net_client.is_some() {
                                    online_serve_countdown = Some(7.0);
                                }
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
                            // Abandonment penalty for online match forfeit
                            if (net_host.is_some() || net_client.is_some()) && active_match_opponent.is_some() {
                                if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                                    let old_r = config.glicko.rating;
                                    let new_rating = config.glicko.update_match(opp_r, opp_rd, 0.0);
                                    config.matches_played += 1;
                                    config.glicko = new_rating;
                                    config.save();
                                    submit_leaderboard_rating(
                                        config.matchmaking_registry_url.as_deref(),
                                        "local_player",
                                        &player1_name,
                                        new_rating.rating,
                                        new_rating.deviation,
                                        config.matches_won,
                                        config.matches_played,
                                    );
                                    last_rating_update = Some((old_r, new_rating.rating, new_rating.rank_title()));
                                }
                            }
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
        // Synchronize background court width in main menu / non-playing states
        let is_menu_active = state == AppState::MainMenu || state == AppState::ExitConfirm;
        let menu_items = if is_menu_active {
            render::ui::get_menu_items_for_mode(game_mode)
        } else {
            Vec::new()
        };
        let is_sizing_court = is_menu_active && (
            menu_items.get(menu_selected) == Some(&render::ui::MenuItem::CourtSize)
            || is_dragging_court_slider
        );

        if is_menu_active {
            let is_local_mode = matches!(game_mode, GameMode::SinglePlayer | GameMode::TwoPlayerLocal | GameMode::BotVsBot);
            let target_court_w = if is_local_mode {
                config.get_court_width(aspect)
            } else {
                800.0
            };
            physics.set_court_width(target_court_w);
        }

        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        // Dynamic camera zoom out for high balls & shifted 2D scenery down
        let allow_zoom = config.high_ball_camera == 1; // 0 = Indicator (rock-solid camera), 1 = DynamicZoom
        let target_cam_y = if state == AppState::Playing || matches!(state, AppState::Victory(_)) {
            if allow_zoom && physics.ball.pos.y < 150.0 {
                (265.0 - (150.0 - physics.ball.pos.y) * 0.45).max(160.0)
            } else {
                265.0
            }
        } else {
            300.0
        };

        let target_zoom = if allow_zoom && (state == AppState::Playing || matches!(state, AppState::Victory(_))) && physics.ball.pos.y < 130.0 {
            1.0 - ((130.0 - physics.ball.pos.y) / 450.0).clamp(0.0, 0.32)
        } else {
            1.0
        };

        let cam_lerp = (frame_dt * 9.0).min(1.0);
        cam_target_y += (target_cam_y - cam_target_y) * cam_lerp;
        cam_zoom_factor += (target_zoom - cam_zoom_factor) * cam_lerp;

        let shake = vfx.get_shake_offset();
        let world_court_w = physics.court_width;
        let world_cam_target = Vec2::new(world_court_w * 0.5, cam_target_y) + shake;
        let (world_base_w, world_base_h) = if aspect >= world_court_w / VIRTUAL_HEIGHT {
            (VIRTUAL_HEIGHT * aspect, VIRTUAL_HEIGHT)
        } else {
            (world_court_w, world_court_w / aspect)
        };
        let world_cam_w = world_base_w / cam_zoom_factor;
        let world_cam_h = world_base_h / cam_zoom_factor;
        let world_cam_x0 = world_cam_target.x - world_cam_w * 0.5;

        // 1. World Camera for Arena, Blobs, Ball & Particles
        set_camera(&Camera2D {
            target: world_cam_target,
            zoom: Vec2::new(2.0 / world_cam_w, 2.0 / world_cam_h),
            ..Default::default()
        });

        let cheer = render::arenas::CheerState {
            last_scorer: last_point_scorer,
            cheer_timer,
            match_winner: match state {
                AppState::Victory(w) => Some(w),
                _ => None,
            },
            left_theme,
            right_theme,
        };

        // 1. Render Arena Environment
        render_arena(arena, global_time, world_cam_x0, world_cam_w, physics.ball.pos, cheer, world_court_w);

        // 2. Render Volleyball Net
        net_renderer.render(global_time, physics.net_x());

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

        // 5b. Real-Time Court Size Visualizer in World Space when sizing court in menu
        if is_sizing_court {
            render_court_size_visualizer(global_time, world_court_w, aspect);
        }

        // 6. UI Overlay Camera
        set_default_camera();
        set_camera(&Camera2D {
            target: Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
            zoom: Vec2::new(2.0 / cam_w, 2.0 / cam_h),
            ..Default::default()
        });

        match state {
            AppState::MainMenu => {
                let items = render::ui::get_menu_items_for_mode(game_mode);
                render_main_menu(
                    game_mode,
                    &items,
                    &player1_name,
                    &config.country_code,
                    difficulty,
                    EloSearchRange::from_index(config.elo_search_range),
                    rule_mode,
                    score_to_win,
                    config.court_width_factor,
                    aspect,
                    arena,
                    left_theme,
                    right_theme,
                    config.high_ball_camera,
                    fullscreen,
                    menu_selected,
                    Some(virt_mouse),
                    global_time,
                );
            }
            AppState::ExitConfirm => {
                let items = render::ui::get_menu_items_for_mode(game_mode);
                render_main_menu(
                    game_mode,
                    &items,
                    &player1_name,
                    &config.country_code,
                    difficulty,
                    EloSearchRange::from_index(config.elo_search_range),
                    rule_mode,
                    score_to_win,
                    config.court_width_factor,
                    aspect,
                    arena,
                    left_theme,
                    right_theme,
                    config.high_ball_camera,
                    fullscreen,
                    menu_selected,
                    Some(virt_mouse),
                    global_time,
                );
                render_exit_confirm(global_time);
            }
            AppState::Leaderboard { period_idx, ref entries, is_refreshing } => {
                render_leaderboard(period_idx, entries, &player1_name, is_refreshing, global_time);
            }
            AppState::PlayerSetup { is_two_player, active_field } => {
                render_player_setup(&player1_name, &config.country_code, &player2_name, is_two_player, active_field, &country_search_buf, global_time);
            }
            AppState::ServerBrowser { selected_idx, ref servers, is_refreshing } => {
                render_server_browser(servers, selected_idx, is_refreshing, global_time);
            }
            AppState::MatchmakingQueue => {
                let (rating, tol, elapsed, range_mode, cross_offer, challenges) = if let Some(ref agent) = quick_match_agent {
                    let el = (global_time - agent.search_start_time).max(0.0) as f64;
                    let intervals = (el / 5.0).floor() as i32;
                    let tol = if agent.elo_range == EloSearchRange::Unrestricted {
                        f64::INFINITY
                    } else {
                        agent.elo_range.initial_tolerance() * (4.0 / 3.0_f64).powi(intervals)
                    };
                    (agent.rating, tol, el as f32, agent.elo_range, agent.active_offer.as_ref(), &agent.active_challenges[..])
                } else {
                    (config.glicko.rating, 100.0, 0.0, EloSearchRange::from_index(config.elo_search_range), None, &[][..])
                };
                render_matchmaking_queue(
                    &player1_name,
                    &config.country_code,
                    rating,
                    tol,
                    range_mode,
                    arena,
                    elapsed,
                    config.glicko.rank_title(),
                    challenges,
                    cross_offer,
                    global_time,
                );
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
                    if config.country_code.is_empty() {
                        format!("{} [{:.0}] (Host)", player1_name, config.glicko.rating)
                    } else {
                        format!("{} {} [{:.0}] (Host)", country_badge(&config.country_code), player1_name, config.glicko.rating)
                    }
                } else if net_client.is_some() {
                    if let Some((ref opp_name, opp_r, _, ref opp_c)) = active_match_opponent {
                        if opp_c.is_empty() {
                            format!("{} [{:.0}]", opp_name, opp_r)
                        } else {
                            format!("{} {} [{:.0}]", country_badge(opp_c), opp_name, opp_r)
                        }
                    } else {
                        "Host (P1)".to_string()
                    }
                } else {
                    match game_mode {
                        GameMode::BotVsBot => format!("CPU 1 [{:.0}]", GlickoRating::ai_baseline(difficulty).0),
                        _ => {
                            if config.country_code.is_empty() {
                                format!("{} [{:.0}]", player1_name, config.glicko.rating)
                            } else {
                                format!("{} {} [{:.0}]", country_badge(&config.country_code), player1_name, config.glicko.rating)
                            }
                        }
                    }
                };

                let right_title = if net_host.is_some() {
                    if let Some((ref opp_name, opp_r, _, ref opp_c)) = active_match_opponent {
                        if opp_c.is_empty() {
                            format!("{} [{:.0}]", opp_name, opp_r)
                        } else {
                            format!("{} {} [{:.0}]", country_badge(opp_c), opp_name, opp_r)
                        }
                    } else {
                        "Client (P2)".to_string()
                    }
                } else if net_client.is_some() {
                    if config.country_code.is_empty() {
                        format!("{} [{:.0}] (You)", player1_name, config.glicko.rating)
                    } else {
                        format!("{} {} [{:.0}] (You)", country_badge(&config.country_code), player1_name, config.glicko.rating)
                    }
                } else {
                    match game_mode {
                        GameMode::SinglePlayer => format!("CPU ({}) [{:.0}]", difficulty.name(), GlickoRating::ai_baseline(difficulty).0),
                        GameMode::TwoPlayerLocal => player2_name.clone(),
                        GameMode::BotVsBot => format!("CPU 2 ({}) [{:.0}]", difficulty.name(), GlickoRating::ai_baseline(difficulty).0),
                        _ => player2_name.clone(),
                    }
                };

                render_hud(
                    &logic,
                    &left_title,
                    &right_title,
                    left_theme,
                    right_theme,
                    physics.ball.pos,
                    config.high_ball_camera,
                    global_time,
                    online_serve_countdown,
                    physics.court_width,
                );

                render_chat_overlay(&chat_log, is_typing_chat, &chat_input, global_time);
            }
            AppState::Paused => {
                let r_name = if game_mode == GameMode::SinglePlayer {
                    format!("CPU ({})", difficulty.name())
                } else {
                    player2_name.clone()
                };
                render_hud(
                    &logic,
                    &player1_name,
                    &r_name,
                    left_theme,
                    right_theme,
                    physics.ball.pos,
                    config.high_ball_camera,
                    global_time,
                    None,
                    physics.court_width,
                );
                render_pause_menu(pause_selected);
            }
            AppState::Victory(winner) => {
                let winner_name = match winner {
                    PlayerSide::Left => {
                        if net_client.is_some() {
                            active_match_opponent.as_ref().map(|(n, _, _, _)| n.as_str()).unwrap_or("Host")
                        } else {
                            &player1_name
                        }
                    }
                    PlayerSide::Right => {
                        if net_client.is_some() {
                            &player1_name
                        } else if let Some((ref opp_name, _, _, _)) = active_match_opponent {
                            opp_name.as_str()
                        } else if game_mode == GameMode::SinglePlayer {
                            "Computer"
                        } else {
                            &player2_name
                        }
                    }
                };
                render_victory_screen(winner, winner_name, &logic, last_rating_update, global_time);
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
