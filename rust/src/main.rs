// Blobby Volley 3 - Main Game Engine
use std::sync::mpsc;
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
use net::{
    NetworkHost, NetworkClient, NetEvent, MatchConfigData, ServerEntry,
    query_master_server_with_registry, get_local_ip, DEFAULT_PORT,
    QuickMatchAgent, MatchmakingStatus, LeaderboardEntry, fetch_leaderboard,
    submit_leaderboard_rating,
};
use render::blob::{render_blobby, BlobTheme};
use render::ball::{render_ball, BallTrail};
use render::net::NetRenderer;
use render::arenas::{render_arena, ArenaType, render_court_size_visualizer};
use render::ui::*;
use config::{UserConfig, EloSearchRange, country_badge, country_index, country_code_at, AVAILABLE_COUNTRIES};
use glicko::GlickoRating;

#[derive(Debug, Clone, Copy, Default)]
struct HoldRepeat {
    active_dir: Option<i8>,
    hold_elapsed: f32,
    repeat_accum: f32,
}

impl HoldRepeat {
    fn reset(&mut self) {
        self.active_dir = None;
        self.hold_elapsed = 0.0;
        self.repeat_accum = 0.0;
    }

    fn update(
        &mut self,
        dir_held: Option<i8>,
        dt: f32,
        initial_delay: f32,
        repeat_interval: Option<f32>,
    ) -> Option<i8> {
        let Some(dir) = dir_held else {
            self.reset();
            return None;
        };

        if self.active_dir != Some(dir) {
            self.active_dir = Some(dir);
            self.hold_elapsed = 0.0;
            self.repeat_accum = 0.0;
            return Some(dir);
        }

        let interval = repeat_interval?;

        self.hold_elapsed += dt;
        if self.hold_elapsed >= initial_delay {
            self.repeat_accum += dt;
            if self.repeat_accum >= interval {
                self.repeat_accum -= interval;
                return Some(dir);
            }
        }
        None
    }
}

fn record_ranked_match_result(
    config: &mut UserConfig,
    player1_name: &str,
    opp_r: f64,
    opp_rd: f64,
    score: f64,
) -> (f64, f64, &'static str) {
    let old_r = config.glicko.rating;
    let new_rating = config.glicko.update_match(opp_r, opp_rd, score);
    config.matches_played += 1;
    if score >= 0.99 {
        config.matches_won += 1;
    }
    config.glicko = new_rating;
    config.save();

    let reg_url = config.matchmaking_registry_url.clone();
    let pid = config.player_id.clone();
    let pname = player1_name.to_string();
    let r = new_rating.rating;
    let rd = new_rating.deviation;
    let won = config.matches_won;
    let played = config.matches_played;
    std::thread::spawn(move || {
        submit_leaderboard_rating(
            reg_url.as_deref(),
            &pid,
            &pname,
            r,
            rd,
            won,
            played,
        );
    });

    (old_r, new_rating.rating, new_rating.rank_title())
}

#[allow(clippy::too_many_arguments)]
fn format_hud_titles(
    net_host: &Option<NetworkHost>,
    net_client: &Option<NetworkClient>,
    active_match_opponent: &Option<(String, f64, f64, String)>,
    game_mode: GameMode,
    player1_name: &str,
    player2_name: &str,
    config: &UserConfig,
    difficulty: AiDifficulty,
) -> (String, String) {
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
            GameMode::TwoPlayerLocal => player2_name.to_string(),
            GameMode::BotVsBot => format!("CPU 2 ({}) [{:.0}]", difficulty.name(), GlickoRating::ai_baseline(difficulty).0),
            _ => player2_name.to_string(),
        }
    };

    (left_title, right_title)
}

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
        window_title: "Blobby Volley 3".to_string(),
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
    audio.volume = config.volume.clamp(0.0, 1.0);
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
    let mut is_ranked_match = false;
    let mut last_rating_update: Option<(f64, f64, &'static str)> = None; // (old_r, new_r, rank_title)
    let mut join_ip_input = format!("127.0.0.1:{}", DEFAULT_PORT);
    let local_ip = get_local_ip();
    let mut server_browser_rx: Option<mpsc::Receiver<Vec<ServerEntry>>> = None;
    let mut leaderboard_rx: Option<mpsc::Receiver<Vec<LeaderboardEntry>>> = None;

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
    let mut is_dragging_volume_slider = false;
    let mut menu_hold = HoldRepeat::default();
    let mut country_hold = HoldRepeat::default();
    let mut backspace_hold = HoldRepeat::default();
    let mut active_menu_mouse_arrow: Option<(MenuItem, i8)> = None;
    let mut active_country_mouse_arrow: Option<i8> = None;
    let mut menu_config_dirty = false;
    let mut country_search_buf = String::new();
    let mut country_search_time = 0.0f32;

    // Helper closure to sync configuration fields in memory
    let sync_config = |cfg: &mut UserConfig,
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
    };

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
        sync_config(cfg, p1, p2, fs, muted, score, ar, diff, rm, lt, rt);
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
            active_menu_mouse_arrow = None;
            active_country_mouse_arrow = None;
            if is_dragging_court_slider || is_dragging_volume_slider {
                is_dragging_court_slider = false;
                is_dragging_volume_slider = false;
                config.save();
            }
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
                let prev_selected = menu_selected;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    menu_selected = if menu_selected == 0 { items.len().saturating_sub(1) } else { menu_selected - 1 };
                }
                if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    menu_selected = (menu_selected + 1) % items.len().max(1);
                }
                if menu_selected != prev_selected {
                    menu_hold.reset();
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
                            render::ui::MenuItem::Volume => {
                                let delta = if $forward { 0.05 } else { -0.05 };
                                config.volume = (((config.volume + delta) * 20.0).round() / 20.0).clamp(0.0, 1.0);
                                audio.volume = config.volume;
                                if config.volume > 0.0 && audio.is_muted {
                                    audio.is_muted = false;
                                    config.sound_muted = false;
                                }
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
                        sync_config(
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
                        menu_config_dirty = true;
                    };
                }

                // Continuous Hold-to-Repeat for Keyboard Left/Right and Mouse `<` / `>` arrows
                let mut held_dir: Option<i8> = None;
                let mut held_item: Option<MenuItem> = None;

                let kb_left = is_key_down(KeyCode::Left)
                    || is_key_down(KeyCode::A)
                    || is_key_pressed(KeyCode::Left)
                    || is_key_pressed(KeyCode::A);
                let kb_right = is_key_down(KeyCode::Right)
                    || is_key_down(KeyCode::D)
                    || is_key_pressed(KeyCode::Right)
                    || is_key_pressed(KeyCode::D);

                if is_mouse_button_pressed(MouseButton::Left)
                    && !is_dragging_court_slider
                    && !is_dragging_volume_slider
                {
                    active_menu_mouse_arrow = None;
                    for layout in &layouts {
                        if layout.left_arrow_rect.is_some_and(|r| r.contains(virt_mouse)) {
                            if menu_selected != layout.index {
                                menu_selected = layout.index;
                                menu_hold.reset();
                            }
                            active_menu_mouse_arrow = Some((layout.item, -1));
                            break;
                        }
                        if layout.right_arrow_rect.is_some_and(|r| r.contains(virt_mouse)) {
                            if menu_selected != layout.index {
                                menu_selected = layout.index;
                                menu_hold.reset();
                            }
                            active_menu_mouse_arrow = Some((layout.item, 1));
                            break;
                        }
                    }
                } else if !is_mouse_button_down(MouseButton::Left) {
                    active_menu_mouse_arrow = None;
                }

                if kb_left && !kb_right {
                    held_dir = Some(-1);
                    held_item = items.get(menu_selected).copied();
                } else if kb_right && !kb_left {
                    held_dir = Some(1);
                    held_item = items.get(menu_selected).copied();
                } else if let Some((locked_item, locked_dir)) = active_menu_mouse_arrow {
                    if !is_dragging_court_slider && !is_dragging_volume_slider {
                        if let Some(layout) = layouts.iter().find(|l| l.item == locked_item) {
                            let target_rect = if locked_dir < 0 {
                                layout.left_arrow_rect
                            } else {
                                layout.right_arrow_rect
                            };
                            if target_rect.is_some_and(|r| r.contains(virt_mouse)) {
                                held_dir = Some(locked_dir);
                                held_item = Some(locked_item);
                            }
                        }
                    }
                }

                let repeat_interval = match held_item {
                    Some(MenuItem::CourtSize) | Some(MenuItem::Volume) => Some(0.045),
                    Some(MenuItem::GameMode)
                    | Some(MenuItem::AiDifficulty)
                    | Some(MenuItem::PointsToWin)
                    | Some(MenuItem::Arena)
                    | Some(MenuItem::P1Color)
                    | Some(MenuItem::P2Color) => Some(0.14),
                    _ => None,
                };

                if let Some(dir) = menu_hold.update(held_dir, frame_dt, 0.28, repeat_interval) {
                    if let Some(item) = held_item {
                        cycle_setting!(item, dir > 0);
                        audio.play_ball_hit(0.18);
                    }
                }

                if held_dir.is_none() && menu_config_dirty {
                    config.save();
                    menu_config_dirty = false;
                }

                // Slider dragging update (saves once on MouseButton::Left release)
                if is_dragging_court_slider && is_mouse_button_down(MouseButton::Left) {
                    if let Some(layout) = layouts.iter().find(|l| l.item == MenuItem::CourtSize) {
                        if let Some(s_rect) = layout.slider_rect {
                            let factor = ((virt_mouse.x - s_rect.x) / s_rect.w).clamp(0.0, 1.0);
                            config.court_width_factor = (factor * 100.0).round() / 100.0;
                            config.court_size = (config.court_width_factor * 3.0).round() as usize;
                        }
                    }
                }
                if is_dragging_volume_slider && is_mouse_button_down(MouseButton::Left) {
                    if let Some(layout) = layouts.iter().find(|l| l.item == MenuItem::Volume) {
                        if let Some(s_rect) = layout.slider_rect {
                            let factor = ((virt_mouse.x - s_rect.x) / s_rect.w).clamp(0.0, 1.0);
                            config.volume = (factor * 100.0).round() / 100.0;
                            audio.volume = config.volume;
                            if config.volume > 0.0 && audio.is_muted {
                                audio.is_muted = false;
                                config.sound_muted = false;
                            }
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
                        // 1. Arrow buttons (handled by menu_hold above)
                        if layout.left_arrow_rect.is_some_and(|r| r.contains(virt_mouse))
                            || layout.right_arrow_rect.is_some_and(|r| r.contains(virt_mouse))
                        {
                            break;
                        }
                        // 2. Slider Track (Schieberegler)
                        if let Some(s_rect) = layout.slider_rect {
                            if s_rect.contains(virt_mouse) {
                                menu_selected = i;
                                let factor = ((virt_mouse.x - s_rect.x) / s_rect.w).clamp(0.0, 1.0);
                                if item == MenuItem::CourtSize {
                                    is_dragging_court_slider = true;
                                    config.court_width_factor = (factor * 100.0).round() / 100.0;
                                    config.court_size = (config.court_width_factor * 3.0).round() as usize;
                                } else if item == MenuItem::Volume {
                                    is_dragging_volume_slider = true;
                                    config.volume = (factor * 100.0).round() / 100.0;
                                    audio.volume = config.volume;
                                    if config.volume > 0.0 && audio.is_muted {
                                        audio.is_muted = false;
                                        config.sound_muted = false;
                                    }
                                }
                                audio.play_ball_hit(0.15);
                                break;
                            }
                        }
                        // 3. Row Body (outside arrow buttons and slider)
                        if layout.row_rect.contains(virt_mouse) {
                            menu_selected = i;
                            if item == MenuItem::StartMatch {
                                action_start_match = true;
                            } else if item == MenuItem::PlayerProfile {
                                action_open_profile = true;
                            } else {
                                audio.play_ball_hit(0.12);
                            }
                            break;
                        }
                    }
                }

                // Keyboard Space or Enter confirms and starts match (or opens profile if profile row)
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    if menu_selected < items.len() && items[menu_selected] == MenuItem::PlayerProfile {
                        action_open_profile = true;
                    } else {
                        action_start_match = true;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    if menu_config_dirty {
                        config.save();
                        menu_config_dirty = false;
                    }
                    menu_hold.reset();
                    state = AppState::ExitConfirm;
                    audio.play_ball_hit(0.2);
                }

                if is_key_pressed(KeyCode::L) {
                    if menu_config_dirty {
                        config.save();
                        menu_config_dirty = false;
                    }
                    menu_hold.reset();
                    let reg_url = config.matchmaking_registry_url.clone();
                    let pname = player1_name.clone();
                    let rating = config.glicko.rating;
                    let won = config.matches_won;
                    let played = config.matches_played;
                    let (tx, rx) = mpsc::channel();
                    leaderboard_rx = Some(rx);
                    std::thread::spawn(move || {
                        let entries = fetch_leaderboard(reg_url.as_deref(), "month", &pname, rating, won, played);
                        let _ = tx.send(entries);
                    });
                    state = AppState::Leaderboard {
                        period_idx: 1,
                        entries: Vec::new(),
                        is_refreshing: true,
                    };
                    audio.play_ball_hit(0.3);
                }

                if action_open_profile {
                    if menu_config_dirty {
                        config.save();
                        menu_config_dirty = false;
                    }
                    menu_hold.reset();
                    state = AppState::PlayerSetup {
                        is_two_player: game_mode == GameMode::TwoPlayerLocal,
                        active_field: 0,
                    };
                    audio.play_ball_hit(0.3);
                } else if action_start_match {
                    menu_hold.reset();
                    menu_config_dirty = false;
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

                    active_match_opponent = None;
                    is_ranked_match = false;
                    last_rating_update = None;
                    last_point_scorer = None;
                    cheer_timer = 0.0;

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
                            // Prompt player 2 name on starting 2-player mode
                            state = AppState::PlayerSetup {
                                is_two_player: true,
                                active_field: 1, // Default focus on player 2
                            };
                            audio.play_ball_hit(0.3);
                        }
                        GameMode::HostOnline => {
                            if let Ok(mut host) = NetworkHost::bind(DEFAULT_PORT, &player1_name, &config.country_code, arena.name()) {
                                host.set_match_config(MatchConfigData {
                                    score_to_win,
                                    rule_mode_index: rule_mode.to_index(),
                                    arena_index: arena.to_index(),
                                    host_theme_index: left_theme.to_index(),
                                    client_theme_index: right_theme.to_index(),
                                    host_name: player1_name.clone(),
                                    host_country: config.country_code.clone(),
                                    host_rating: config.glicko.rating,
                                });
                                net_host = Some(host);
                                state = AppState::HostLobby;
                            }
                        }
                        GameMode::JoinOnline => {
                            state = AppState::JoinScreen;
                        }
                        GameMode::ServerBrowser => {
                            let reg_url = config.matchmaking_registry_url.clone();
                            let (tx, rx) = mpsc::channel();
                            server_browser_rx = Some(rx);
                            std::thread::spawn(move || {
                                let list = query_master_server_with_registry(reg_url.as_deref());
                                let _ = tx.send(list);
                            });
                            state = AppState::ServerBrowser {
                                selected_idx: 0,
                                servers: Vec::new(),
                                is_refreshing: true,
                            };
                        }
                        GameMode::BotVsBot => {
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
                if let Some(ref rx) = leaderboard_rx {
                    match rx.try_recv() {
                        Ok(list) => {
                            *entries = list;
                            *is_refreshing = false;
                            leaderboard_rx = None;
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            *is_refreshing = false;
                            leaderboard_rx = None;
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                }

                let mut trigger_refresh = false;

                if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                    *period_idx = if *period_idx == 0 { 2 } else { *period_idx - 1 };
                    trigger_refresh = true;
                }
                if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                    *period_idx = (*period_idx + 1) % 3;
                    trigger_refresh = true;
                }
                if is_key_pressed(KeyCode::Key1) {
                    *period_idx = 0;
                    trigger_refresh = true;
                }
                if is_key_pressed(KeyCode::Key2) {
                    *period_idx = 1;
                    trigger_refresh = true;
                }
                if is_key_pressed(KeyCode::Key3) {
                    *period_idx = 2;
                    trigger_refresh = true;
                }

                let (tab_rects, refresh_rect, back_rect) = get_leaderboard_rects();
                let mut go_back = is_key_pressed(KeyCode::Escape);
                if is_mouse_button_pressed(MouseButton::Left) {
                    for (i, tab_rect) in tab_rects.iter().enumerate() {
                        if tab_rect.contains(virt_mouse) && *period_idx != i {
                            *period_idx = i;
                            trigger_refresh = true;
                            break;
                        }
                    }
                    if refresh_rect.contains(virt_mouse) {
                        trigger_refresh = true;
                    } else if back_rect.contains(virt_mouse) {
                        go_back = true;
                    }
                }

                if !go_back && (trigger_refresh || is_key_pressed(KeyCode::R)) {
                    *is_refreshing = true;
                    let p_str = match *period_idx {
                        0 => "week",
                        1 => "month",
                        _ => "year",
                    };
                    let reg_url = config.matchmaking_registry_url.clone();
                    let pname = player1_name.clone();
                    let rating = config.glicko.rating;
                    let won = config.matches_won;
                    let played = config.matches_played;
                    let (tx, rx) = mpsc::channel();
                    leaderboard_rx = Some(rx);
                    std::thread::spawn(move || {
                        let list = fetch_leaderboard(reg_url.as_deref(), p_str, &pname, rating, won, played);
                        let _ = tx.send(list);
                    });
                    audio.play_ball_hit(0.2);
                }

                if go_back {
                    leaderboard_rx = None;
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
                }
            }

            AppState::PlayerSetup { is_two_player, mut active_field } => {
                let country_field_idx = if is_two_player { 2 } else { 1 };
                let total_fields = if is_two_player { 3 } else { 2 };
                let setup_layout = get_player_setup_rects(is_two_player);

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

                // Backspace with hold-to-repeat
                let bs_held = if is_key_down(KeyCode::Backspace) || is_key_pressed(KeyCode::Backspace) {
                    Some(-1)
                } else {
                    None
                };
                if backspace_hold.update(bs_held, frame_dt, 0.32, Some(0.045)).is_some() {
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

                // Country cycle with hold-to-repeat (keyboard Left/Right when active OR mouse holding `<`/`>`)
                let mut country_dir_held: Option<i8> = None;
                if active_field == country_field_idx {
                    let kb_l = is_key_down(KeyCode::Left) || is_key_pressed(KeyCode::Left);
                    let kb_r = is_key_down(KeyCode::Right) || is_key_pressed(KeyCode::Right);
                    if kb_l && !kb_r {
                        country_dir_held = Some(-1);
                    } else if kb_r && !kb_l {
                        country_dir_held = Some(1);
                    }
                }
                if is_mouse_button_pressed(MouseButton::Left) {
                    active_country_mouse_arrow = None;
                    if setup_layout.country_left_arrow.contains(virt_mouse) {
                        if active_field != country_field_idx {
                            active_field = country_field_idx;
                            state = AppState::PlayerSetup { is_two_player, active_field };
                        }
                        active_country_mouse_arrow = Some(-1);
                    } else if setup_layout.country_right_arrow.contains(virt_mouse) {
                        if active_field != country_field_idx {
                            active_field = country_field_idx;
                            state = AppState::PlayerSetup { is_two_player, active_field };
                        }
                        active_country_mouse_arrow = Some(1);
                    }
                } else if !is_mouse_button_down(MouseButton::Left) {
                    active_country_mouse_arrow = None;
                }

                if country_dir_held.is_none() {
                    if let Some(locked_dir) = active_country_mouse_arrow {
                        let target_rect = if locked_dir < 0 {
                            setup_layout.country_left_arrow
                        } else {
                            setup_layout.country_right_arrow
                        };
                        if target_rect.contains(virt_mouse) {
                            country_dir_held = Some(locked_dir);
                        }
                    }
                }

                if let Some(dir) = country_hold.update(country_dir_held, frame_dt, 0.28, Some(0.045)) {
                    country_search_buf.clear();
                    let len = AVAILABLE_COUNTRIES.len();
                    let mut c_idx = country_index(&config.country_code);
                    c_idx = if dir < 0 {
                        (c_idx + len - 1) % len
                    } else {
                        (c_idx + 1) % len
                    };
                    config.country_code = country_code_at(c_idx).to_string();
                    audio.play_ball_hit(0.15);
                }

                // Mouse clicks on fields & buttons
                let mut confirm_setup = false;
                let mut cancel_setup = false;
                if is_mouse_button_pressed(MouseButton::Left) {
                    if setup_layout.country_left_arrow.contains(virt_mouse)
                        || setup_layout.country_right_arrow.contains(virt_mouse)
                    {
                        // Handled by country_hold above
                    } else if setup_layout.p1_rect.contains(virt_mouse) {
                        country_search_buf.clear();
                        active_field = 0;
                        state = AppState::PlayerSetup { is_two_player, active_field };
                        audio.play_ball_hit(0.12);
                    } else if setup_layout.p2_rect.is_some_and(|r| r.contains(virt_mouse)) {
                        country_search_buf.clear();
                        active_field = 1;
                        state = AppState::PlayerSetup { is_two_player, active_field };
                        audio.play_ball_hit(0.12);
                    } else if setup_layout.country_rect.contains(virt_mouse) {
                        active_field = country_field_idx;
                        state = AppState::PlayerSetup { is_two_player, active_field };
                        audio.play_ball_hit(0.12);
                    } else if setup_layout.confirm_btn.contains(virt_mouse) {
                        confirm_setup = true;
                    } else if setup_layout.cancel_btn.contains(virt_mouse) {
                        cancel_setup = true;
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

                if is_key_pressed(KeyCode::Enter) {
                    confirm_setup = true;
                }
                if is_key_pressed(KeyCode::Escape) {
                    cancel_setup = true;
                }

                if cancel_setup {
                    country_hold.reset();
                    backspace_hold.reset();
                    country_search_buf.clear();
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
                } else if confirm_setup {
                    country_hold.reset();
                    backspace_hold.reset();
                    country_search_buf.clear();
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

                    if is_two_player || matches!(game_mode, GameMode::SinglePlayer | GameMode::TwoPlayerLocal | GameMode::BotVsBot) {
                        active_match_opponent = None;
                        is_ranked_match = false;
                        last_rating_update = None;
                        last_point_scorer = None;
                        cheer_timer = 0.0;
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
                    } else {
                        state = AppState::MainMenu;
                        audio.play_ball_hit(0.25);
                    }
                }
            }

            AppState::ServerBrowser { ref mut selected_idx, ref mut servers, ref mut is_refreshing } => {
                if let Some(ref rx) = server_browser_rx {
                    match rx.try_recv() {
                        Ok(list) => {
                            *servers = list;
                            *is_refreshing = false;
                            if *selected_idx >= servers.len() {
                                *selected_idx = 0;
                            }
                            server_browser_rx = None;
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            *is_refreshing = false;
                            server_browser_rx = None;
                        }
                        Err(mpsc::TryRecvError::Empty) => {}
                    }
                }

                let num_servers = servers.len();
                if num_servers > 0 {
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        *selected_idx = if *selected_idx == 0 { num_servers - 1 } else { *selected_idx - 1 };
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        *selected_idx = (*selected_idx + 1) % num_servers;
                    }
                }

                let (row_rects, refresh_rect, back_rect) = get_server_browser_rects(num_servers);
                let mut do_refresh = is_key_pressed(KeyCode::R);
                let mut do_connect = is_key_pressed(KeyCode::Enter) && num_servers > 0;
                let mut do_back = is_key_pressed(KeyCode::Escape);

                if is_mouse_button_pressed(MouseButton::Left) {
                    let mut hit_row = false;
                    for (i, r) in row_rects.iter().enumerate() {
                        if r.contains(virt_mouse) {
                            if *selected_idx == i {
                                do_connect = true;
                            } else {
                                *selected_idx = i;
                                audio.play_ball_hit(0.15);
                            }
                            hit_row = true;
                            break;
                        }
                    }
                    if !hit_row {
                        if refresh_rect.contains(virt_mouse) {
                            do_refresh = true;
                        } else if back_rect.contains(virt_mouse) {
                            do_back = true;
                        }
                    }
                }

                if do_refresh {
                    *is_refreshing = true;
                    let reg_url = config.matchmaking_registry_url.clone();
                    let (tx, rx) = mpsc::channel();
                    server_browser_rx = Some(rx);
                    std::thread::spawn(move || {
                        let list = query_master_server_with_registry(reg_url.as_deref());
                        let _ = tx.send(list);
                    });
                    audio.play_ball_hit(0.2);
                }

                if do_connect && *selected_idx < servers.len() {
                    let s = &servers[*selected_idx];
                    join_ip_input = format!("{}:{}", s.host, s.port);
                    let mut client = NetworkClient::new();
                    client.connect_async(&join_ip_input);
                    net_client = Some(client);
                    active_match_opponent = None;
                    is_ranked_match = false;
                    server_browser_rx = None;
                    state = AppState::JoinScreen;
                    audio.play_ball_hit(0.25);
                } else if do_back {
                    server_browser_rx = None;
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
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
                                if let Ok(mut host) = NetworkHost::bind(DEFAULT_PORT, &player1_name, &config.country_code, arena.name()) {
                                    host.set_match_config(MatchConfigData {
                                        score_to_win,
                                        rule_mode_index: rule_mode.to_index(),
                                        arena_index: arena.to_index(),
                                        host_theme_index: left_theme.to_index(),
                                        client_theme_index: right_theme.to_index(),
                                        host_name: player1_name.clone(),
                                        host_country: config.country_code.clone(),
                                        host_rating: config.glicko.rating,
                                    });
                                    net_host = Some(host);
                                    active_match_opponent = Some((opponent_name, opponent_rating, opponent_rd, opponent_country));
                                    is_ranked_match = true;
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
                                join_ip_input = target_addr.clone();
                                let mut client = NetworkClient::new();
                                client.connect_async(&target_addr);
                                net_client = Some(client);
                                active_match_opponent = Some((opponent_name, opponent_rating, opponent_rd, opponent_country));
                                is_ranked_match = true;
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
                    active_match_opponent = None;
                    is_ranked_match = false;
                    state = AppState::MainMenu;
                } else if let Some(ref mut host) = net_host {
                    host.update(global_time);
                    if let Some((cname, ccountry, crating)) = host.client_hello.take() {
                        if active_match_opponent.is_none() {
                            active_match_opponent = Some((cname, crating, 100.0, ccountry));
                        }
                    }
                    if host.is_client_connected {
                        logic = GameLogic::new(rule_mode, score_to_win);
                        physics = PhysicWorld::new();
                        physics.reset_players();
                        physics.reset_ball(Some(PlayerSide::Left));
                        logic.on_serve();
                        chat_log.clear();
                        last_point_scorer = None;
                        cheer_timer = 0.0;
                        state = AppState::Playing;
                        audio.play_whistle();
                    }
                }
            }

            AppState::JoinScreen => {
                if is_key_pressed(KeyCode::Escape) {
                    net_client = None;
                    active_match_opponent = None;
                    is_ranked_match = false;
                    backspace_hold.reset();
                    state = AppState::MainMenu;
                } else {
                    while let Some(c) = get_char_pressed() {
                        if (c.is_ascii_digit() || c == '.' || c == ':') && join_ip_input.len() < 24 {
                            join_ip_input.push(c);
                        }
                    }
                    let bs_held = if is_key_down(KeyCode::Backspace) || is_key_pressed(KeyCode::Backspace) {
                        Some(-1)
                    } else {
                        None
                    };
                    if backspace_hold.update(bs_held, frame_dt, 0.32, Some(0.045)).is_some() {
                        join_ip_input.pop();
                    }

                    if is_key_pressed(KeyCode::Enter) {
                        let is_already_connecting = net_client.as_ref().is_some_and(|c| c.is_connecting);
                        if !is_already_connecting {
                            is_ranked_match = false;
                            active_match_opponent = None;
                            let mut client = NetworkClient::new();
                            client.connect_async(&join_ip_input);
                            net_client = Some(client);
                        }
                    }

                    if let Some(ref mut client) = net_client {
                        client.update();
                        if client.is_connected {
                            client.send_hello(&player1_name, &config.country_code, config.glicko.rating);
                            physics = PhysicWorld::new();
                            logic = GameLogic::new(rule_mode, score_to_win);
                            if let Some(cfg) = client.received_match_config.take() {
                                score_to_win = cfg.score_to_win;
                                rule_mode = RuleMode::from_index(cfg.rule_mode_index);
                                logic = GameLogic::new(rule_mode, score_to_win);
                                arena = ArenaType::from_index(cfg.arena_index);
                                left_theme = BlobTheme::from_index(cfg.host_theme_index);
                                right_theme = BlobTheme::from_index(cfg.client_theme_index);
                                if active_match_opponent.is_none() {
                                    active_match_opponent = Some((cfg.host_name, cfg.host_rating, 100.0, cfg.host_country));
                                }
                            }
                            chat_log.clear();
                            last_point_scorer = None;
                            cheer_timer = 0.0;
                            backspace_hold.reset();
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
                        if (c.is_ascii_graphic() || c == ' ') && chat_input.len() < 40 {
                            chat_input.push(c);
                        }
                    }
                    let bs_held = if is_key_down(KeyCode::Backspace) || is_key_pressed(KeyCode::Backspace) {
                        Some(-1)
                    } else {
                        None
                    };
                    if backspace_hold.update(bs_held, frame_dt, 0.32, Some(0.045)).is_some() {
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
                        backspace_hold.reset();
                        is_typing_chat = false;
                    } else if is_key_pressed(KeyCode::Escape) {
                        chat_input.clear();
                        backspace_hold.reset();
                        is_typing_chat = false;
                    }
                } else {
                    backspace_hold.reset();
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

                    if let Some(cfg) = client.received_match_config.take() {
                        score_to_win = cfg.score_to_win;
                        rule_mode = RuleMode::from_index(cfg.rule_mode_index);
                        logic = GameLogic::new(rule_mode, score_to_win);
                        arena = ArenaType::from_index(cfg.arena_index);
                        left_theme = BlobTheme::from_index(cfg.host_theme_index);
                        right_theme = BlobTheme::from_index(cfg.client_theme_index);
                        if active_match_opponent.is_none() {
                            active_match_opponent = Some((cfg.host_name, cfg.host_rating, 100.0, cfg.host_country));
                        }
                    }

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
                    }

                    for ev in client.take_events() {
                        match ev {
                            NetEvent::HitBlob { intensity, .. } => {
                                audio.play_ball_hit(intensity);
                                vfx.spawn_hit_shockwave(physics.ball.pos, intensity);
                                net_renderer.trigger_hit(intensity * 0.3);
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

                    for (sender, text) in client.new_chat_messages.drain(..) {
                        chat_log.push((sender, text, global_time));
                        audio.play_score();
                    }

                    if let Some(winner) = logic.check_win() {
                        if is_ranked_match {
                            if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                                let score = if winner == PlayerSide::Right { 1.0 } else { 0.0 };
                                last_rating_update = Some(record_ranked_match_result(
                                    &mut config,
                                    &player1_name,
                                    opp_r,
                                    opp_rd,
                                    score,
                                ));
                            }
                            is_ranked_match = false;
                        } else {
                            last_rating_update = None;
                        }
                        state = AppState::Victory(winner);
                        vfx.spawn_confetti_celebration(Vec2::new(400.0, 200.0), 90);
                        audio.play_score();
                    } else if !client.is_connected {
                        if is_ranked_match {
                            if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                                last_rating_update = Some(record_ranked_match_result(
                                    &mut config,
                                    &player1_name,
                                    opp_r,
                                    opp_rd,
                                    0.0,
                                ));
                            }
                            is_ranked_match = false;
                        }
                        active_match_opponent = None;
                        net_client = None;
                        state = AppState::MainMenu;
                    }
                }
                // Host / Local authoritative simulation
                else {
                    let mut host_disconnected_unranked = false;
                    let (p1_input, p2_input) = if let Some(ref mut host) = net_host {
                        host.update(global_time);
                        if let Some((cname, ccountry, crating)) = host.client_hello.take() {
                            if active_match_opponent.is_none() {
                                active_match_opponent = Some((cname, crating, 100.0, ccountry));
                            }
                        }
                        if !host.is_client_connected && logic.winning_player.is_none() {
                            if is_ranked_match {
                                // Opponent disconnected/forfeited - host wins!
                                if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                                    last_rating_update = Some(record_ranked_match_result(
                                        &mut config,
                                        &player1_name,
                                        opp_r,
                                        opp_rd,
                                        1.0,
                                    ));
                                }
                                is_ranked_match = false;
                                state = AppState::Victory(PlayerSide::Left);
                                audio.play_score();
                            } else {
                                host_disconnected_unranked = true;
                            }
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

                    if host_disconnected_unranked {
                        net_host = None;
                        active_match_opponent = None;
                        state = AppState::MainMenu;
                    }

                    if state == AppState::Playing {
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
                                    if is_ranked_match {
                                        if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                                            let score = if winner == PlayerSide::Left { 1.0 } else { 0.0 };
                                            last_rating_update = Some(record_ranked_match_result(
                                                &mut config,
                                                &player1_name,
                                                opp_r,
                                                opp_rd,
                                                score,
                                            ));
                                        }
                                        is_ranked_match = false;
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
                }

                net_renderer.update(frame_dt);
                vfx.update(frame_dt);
                ball_trail.update(physics.ball.pos, physics.ball.vel.length());
            }

            AppState::Paused => {
                // Keep online network connections alive while paused
                if let Some(ref mut host) = net_host {
                    host.update(global_time);
                    if let Some((cname, ccountry, crating)) = host.client_hello.take() {
                        if active_match_opponent.is_none() {
                            active_match_opponent = Some((cname, crating, 100.0, ccountry));
                        }
                    }
                    for (sender, text) in host.new_chat_messages.drain(..) {
                        chat_log.push((sender, text, global_time));
                    }
                    if !host.is_client_connected && logic.winning_player.is_none() {
                        if is_ranked_match {
                            if let Some((_, opp_r, opp_rd, _)) = active_match_opponent {
                                last_rating_update = Some(record_ranked_match_result(
                                    &mut config,
                                    &player1_name,
                                    opp_r,
                                    opp_rd,
                                    1.0,
                                ));
                            }
                            is_ranked_match = false;
                            state = AppState::Victory(PlayerSide::Left);
                            audio.play_score();
                        } else {
                            net_host = None;
                            active_match_opponent = None;
                            state = AppState::MainMenu;
                        }
                    }
                } else if let Some(ref mut client) = net_client {
                    client.update();
                    let _ = client.take_events();
                    for (sender, text) in client.new_chat_messages.drain(..) {
                        chat_log.push((sender, text, global_time));
                    }
                    if !client.is_connected {
                        if is_ranked_match {
                            if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                                last_rating_update = Some(record_ranked_match_result(
                                    &mut config,
                                    &player1_name,
                                    opp_r,
                                    opp_rd,
                                    0.0,
                                ));
                            }
                            is_ranked_match = false;
                        }
                        active_match_opponent = None;
                        net_client = None;
                        state = AppState::MainMenu;
                    }
                }

                if state == AppState::Paused {
                    let is_online = net_host.is_some() || net_client.is_some();
                    if is_online && pause_selected == 1 {
                        pause_selected = 0;
                    }

                    if is_key_pressed(KeyCode::Escape) {
                        state = AppState::Playing;
                    }
                    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                        pause_selected = if is_online {
                            if pause_selected == 0 { 2 } else { 0 }
                        } else if pause_selected == 0 {
                            2
                        } else {
                            pause_selected - 1
                        };
                    }
                    if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                        pause_selected = if is_online {
                            if pause_selected == 0 { 2 } else { 0 }
                        } else {
                            (pause_selected + 1) % 3
                        };
                    }

                    let mut triggered_action: Option<usize> = None;
                    if is_mouse_button_pressed(MouseButton::Left) {
                        for (id, rect) in get_pause_button_rects(is_online) {
                            if rect.contains(virt_mouse) {
                                pause_selected = id;
                                triggered_action = Some(id);
                                break;
                            }
                        }
                    }
                    if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                        triggered_action = Some(pause_selected);
                    }

                    if let Some(action_id) = triggered_action {
                        match action_id {
                            0 => state = AppState::Playing,
                            1 if !is_online => {
                                logic.reset_match();
                                physics.reset_players();
                                physics.reset_ball(Some(PlayerSide::Left));
                                logic.on_serve();
                                bot_left.reset_rally();
                                bot_right.reset_rally();
                                last_point_scorer = None;
                                cheer_timer = 0.0;
                                state = AppState::Playing;
                                audio.play_whistle();
                            }
                            2 => {
                                // Abandonment penalty for ranked online match forfeit
                                if is_online && is_ranked_match {
                                    if let Some((_, opp_r, opp_rd, _)) = active_match_opponent.take() {
                                        last_rating_update = Some(record_ranked_match_result(
                                            &mut config,
                                            &player1_name,
                                            opp_r,
                                            opp_rd,
                                            0.0,
                                        ));
                                    }
                                    is_ranked_match = false;
                                }
                                active_match_opponent = None;
                                net_host = None;
                                net_client = None;
                                state = AppState::MainMenu;
                            }
                            _ => {}
                        }
                    }
                }
            }

            AppState::Victory(_) => {
                vfx.update(frame_dt);
                let allow_rematch = net_host.is_none()
                    && net_client.is_none()
                    && matches!(game_mode, GameMode::SinglePlayer | GameMode::TwoPlayerLocal | GameMode::BotVsBot);
                let (rematch_rect_opt, menu_rect) = get_victory_button_rects(last_rating_update.is_some(), allow_rematch);

                let mut do_rematch = false;
                let mut do_menu = false;

                if allow_rematch {
                    if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::R) {
                        do_rematch = true;
                    } else if is_key_pressed(KeyCode::Escape) {
                        do_menu = true;
                    }
                } else if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
                    do_menu = true;
                }

                if is_mouse_button_pressed(MouseButton::Left) {
                    if let Some(r_rect) = rematch_rect_opt {
                        if r_rect.contains(virt_mouse) {
                            do_rematch = true;
                        }
                    }
                    if menu_rect.contains(virt_mouse) {
                        do_menu = true;
                    }
                }

                if do_rematch {
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
                    last_rating_update = None;
                    last_point_scorer = None;
                    cheer_timer = 0.0;
                    state = AppState::Playing;
                    audio.play_whistle();
                } else if do_menu {
                    net_host = None;
                    net_client = None;
                    active_match_opponent = None;
                    is_ranked_match = false;
                    last_rating_update = None;
                    last_point_scorer = None;
                    cheer_timer = 0.0;
                    state = AppState::MainMenu;
                    audio.play_ball_hit(0.2);
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
                    config.volume,
                    audio.is_muted,
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
                    config.volume,
                    audio.is_muted,
                    fullscreen,
                    menu_selected,
                    Some(virt_mouse),
                    global_time,
                );
                render_exit_confirm(Some(virt_mouse), global_time);
            }
            AppState::Leaderboard { period_idx, ref entries, is_refreshing } => {
                render_leaderboard(period_idx, entries, &player1_name, is_refreshing, Some(virt_mouse), global_time);
            }
            AppState::PlayerSetup { is_two_player, active_field } => {
                render_player_setup(
                    &player1_name,
                    &config.country_code,
                    &player2_name,
                    is_two_player,
                    active_field,
                    &country_search_buf,
                    Some(virt_mouse),
                    global_time,
                );
            }
            AppState::ServerBrowser { selected_idx, ref servers, is_refreshing } => {
                render_server_browser(servers, selected_idx, is_refreshing, Some(virt_mouse), global_time);
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
                let is_connected = net_host.as_ref().is_some_and(|h| h.is_client_connected);
                let room_name = net_host.as_ref().map_or("Match", |h| &h.room_name);
                let host_local_ip = net_host.as_ref().map_or(local_ip.as_str(), |h| &h.local_ip);
                let public_ep = net_host.as_ref().and_then(|h| h.public_endpoint.as_deref());
                render_host_lobby(room_name, DEFAULT_PORT, host_local_ip, public_ep, is_connected, global_time);
            }
            AppState::JoinScreen => {
                let is_connecting = net_client.as_ref().is_some_and(|c| c.is_connecting);
                let err = net_client.as_ref().and_then(|c| c.error_message.as_deref());
                render_join_screen(&join_ip_input, is_connecting, err, global_time);
            }
            AppState::Playing => {
                let (left_title, right_title) = format_hud_titles(
                    &net_host,
                    &net_client,
                    &active_match_opponent,
                    game_mode,
                    &player1_name,
                    &player2_name,
                    &config,
                    difficulty,
                );

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
                let (left_title, right_title) = format_hud_titles(
                    &net_host,
                    &net_client,
                    &active_match_opponent,
                    game_mode,
                    &player1_name,
                    &player2_name,
                    &config,
                    difficulty,
                );
                render_hud(
                    &logic,
                    &left_title,
                    &right_title,
                    left_theme,
                    right_theme,
                    physics.ball.pos,
                    config.high_ball_camera,
                    global_time,
                    None,
                    physics.court_width,
                );
                let is_online = net_host.is_some() || net_client.is_some();
                render_pause_menu(pause_selected, is_online, Some(virt_mouse));
            }
            AppState::Victory(winner) => {
                let allow_rematch = net_host.is_none()
                    && net_client.is_none()
                    && matches!(game_mode, GameMode::SinglePlayer | GameMode::TwoPlayerLocal | GameMode::BotVsBot);
                let winner_name = match winner {
                    PlayerSide::Left => {
                        if net_client.is_some() {
                            active_match_opponent.as_ref().map(|(n, _, _, _)| n.as_str()).unwrap_or("Host")
                        } else if game_mode == GameMode::BotVsBot && net_host.is_none() {
                            "CPU 1"
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
                        } else if game_mode == GameMode::BotVsBot {
                            "CPU 2"
                        } else {
                            &player2_name
                        }
                    }
                };
                render_victory_screen(
                    winner,
                    winner_name,
                    &logic,
                    last_rating_update,
                    allow_rematch,
                    Some(virt_mouse),
                    global_time,
                );
            }
        }

        set_default_camera();
        next_frame().await;
    }
}
