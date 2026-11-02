// Blobby Volley 3 - 2026 Remastered Glassmorphism HUD, Menus, Profile Setup, Online Lobby & Chat UI
use macroquad::prelude::*;
use crate::physics::PlayerSide;
use crate::rules::{GameLogic, RuleMode};
use crate::render::blob::BlobTheme;
use crate::ai::AiDifficulty;
use crate::render::arenas::ArenaType;
use crate::constants::*;
use crate::net::{ServerEntry, LeaderboardEntry, CrossArenaOffer, ChallengeBroadcast};
use crate::config::{EloSearchRange, country_badge, country_display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    SinglePlayer,
    QuickMatch,
    TwoPlayerLocal,
    BotVsBot,
    HostOnline,
    JoinOnline,
    ServerBrowser,
}

impl GameMode {
    pub fn name(self) -> &'static str {
        match self {
            GameMode::SinglePlayer => "1P vs CPU",
            GameMode::QuickMatch => "Quick Match (Glicko-2 Ranked)",
            GameMode::TwoPlayerLocal => "2 Players (Local)",
            GameMode::BotVsBot => "CPU vs CPU",
            GameMode::HostOnline => "Host Online Match",
            GameMode::JoinOnline => "Direct Connect (IP)",
            GameMode::ServerBrowser => "Public Server Browser",
        }
    }

    pub fn next(self) -> Self {
        match self {
            GameMode::SinglePlayer => GameMode::QuickMatch,
            GameMode::QuickMatch => GameMode::TwoPlayerLocal,
            GameMode::TwoPlayerLocal => GameMode::BotVsBot,
            GameMode::BotVsBot => GameMode::HostOnline,
            GameMode::HostOnline => GameMode::JoinOnline,
            GameMode::JoinOnline => GameMode::ServerBrowser,
            GameMode::ServerBrowser => GameMode::SinglePlayer,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            GameMode::SinglePlayer => GameMode::ServerBrowser,
            GameMode::QuickMatch => GameMode::SinglePlayer,
            GameMode::TwoPlayerLocal => GameMode::QuickMatch,
            GameMode::BotVsBot => GameMode::TwoPlayerLocal,
            GameMode::HostOnline => GameMode::BotVsBot,
            GameMode::JoinOnline => GameMode::HostOnline,
            GameMode::ServerBrowser => GameMode::JoinOnline,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    StartMatch,
    GameMode,
    PlayerProfile,
    AiDifficulty,
    MatchmakingRange,
    Rules,
    PointsToWin,
    CourtSize,
    Arena,
    P1Color,
    P2Color,
    HighBallCamera,
    Fullscreen,
}

pub fn get_menu_items_for_mode(mode: GameMode) -> Vec<MenuItem> {
    match mode {
        GameMode::SinglePlayer => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::PlayerProfile,
            MenuItem::AiDifficulty,
            MenuItem::Rules,
            MenuItem::PointsToWin,
            MenuItem::CourtSize,
            MenuItem::Arena,
            MenuItem::P1Color,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
        GameMode::TwoPlayerLocal => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::PlayerProfile,
            MenuItem::Rules,
            MenuItem::PointsToWin,
            MenuItem::CourtSize,
            MenuItem::Arena,
            MenuItem::P1Color,
            MenuItem::P2Color,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
        GameMode::QuickMatch => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::PlayerProfile,
            MenuItem::MatchmakingRange,
            MenuItem::Arena,
            MenuItem::P1Color,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
        GameMode::BotVsBot => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::AiDifficulty,
            MenuItem::Rules,
            MenuItem::PointsToWin,
            MenuItem::CourtSize,
            MenuItem::Arena,
            MenuItem::P1Color,
            MenuItem::P2Color,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
        GameMode::HostOnline => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::PlayerProfile,
            MenuItem::Rules,
            MenuItem::PointsToWin,
            MenuItem::Arena,
            MenuItem::P1Color,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
        GameMode::JoinOnline | GameMode::ServerBrowser => vec![
            MenuItem::StartMatch,
            MenuItem::GameMode,
            MenuItem::PlayerProfile,
            MenuItem::HighBallCamera,
            MenuItem::Fullscreen,
        ],
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MenuRowLayout {
    pub index: usize,
    pub item: MenuItem,
    pub row_rect: Rect,
    pub left_arrow_rect: Option<Rect>,
    pub right_arrow_rect: Option<Rect>,
    pub slider_rect: Option<Rect>,
}

pub fn get_main_menu_layout(items: &[MenuItem]) -> Vec<MenuRowLayout> {
    let card_w = 560.0;
    let card_h = 430.0;
    let card_x = 24.0;
    let card_y = 115.0;

    let num = items.len().max(1);
    let row_h = (card_h - 26.0) / num as f32;
    let start_y = card_y + 14.0 + row_h * 0.5;

    let mut rows = Vec::with_capacity(num);
    for (i, &item) in items.iter().enumerate() {
        let y = start_y + i as f32 * row_h;
        let row_rect = Rect::new(card_x + 14.0, y - row_h * 0.44, card_w - 28.0, row_h * 0.88);

        if item == MenuItem::StartMatch {
            // Action button (START MATCH) - no arrows
            rows.push(MenuRowLayout {
                index: i,
                item,
                row_rect,
                left_arrow_rect: None,
                right_arrow_rect: None,
                slider_rect: None,
            });
        } else if item == MenuItem::CourtSize {
            // Schieberegler (Slider) row with left/right fine-tuning buttons and central slider track
            let btn_size = 24.0;
            let left_btn = Rect::new(card_x + card_w - 270.0, y - btn_size * 0.5, btn_size, btn_size);
            let right_btn = Rect::new(card_x + card_w - 32.0, y - btn_size * 0.5, btn_size, btn_size);
            let track_x = left_btn.x + btn_size + 6.0;
            let track_w = right_btn.x - track_x - 6.0;
            let track_h = 24.0;
            let slider_rect = Rect::new(track_x, y - track_h * 0.5, track_w, track_h);
            rows.push(MenuRowLayout {
                index: i,
                item,
                row_rect,
                left_arrow_rect: Some(left_btn),
                right_arrow_rect: Some(right_btn),
                slider_rect: Some(slider_rect),
            });
        } else {
            // Settings row with distinct left and right arrow buttons
            let btn_size = 24.0;
            let left_btn = Rect::new(card_x + card_w - 270.0, y - btn_size * 0.5, btn_size, btn_size);
            let right_btn = Rect::new(card_x + card_w - 32.0, y - btn_size * 0.5, btn_size, btn_size);
            rows.push(MenuRowLayout {
                index: i,
                item,
                row_rect,
                left_arrow_rect: Some(left_btn),
                right_arrow_rect: Some(right_btn),
                slider_rect: None,
            });
        }
    }
    rows
}

pub fn render_hud(
    logic: &GameLogic,
    left_name: &str,
    right_name: &str,
    left_theme: BlobTheme,
    right_theme: BlobTheme,
    ball_pos: Vec2,
    high_ball_camera: usize,
    time: f32,
    serve_countdown: Option<f32>,
    court_width: f32,
) {
    let board_w = 640.0;
    let board_h = 62.0;
    let board_x = (VIRTUAL_WIDTH - board_w) * 0.5;
    let board_y = 12.0;

    // Glassmorphism card backdrop with rounded aesthetic
    draw_rectangle(board_x, board_y, board_w, board_h, Color::new(0.06, 0.09, 0.16, 0.88));
    draw_rectangle_lines(board_x, board_y, board_w, board_h, 1.5, Color::new(0.25, 0.45, 0.8, 0.5));

    // Glow accent line on top of card
    draw_line(
        board_x + 10.0,
        board_y + 1.0,
        board_x + board_w - 10.0,
        board_y + 1.0,
        2.0,
        Color::new(0.2, 0.85, 1.0, 0.75),
    );

    // Center Score Pod: Dedicated slot x: 330..470 (width 140px)
    let pod_w = 140.0;
    let pod_h = board_h - 10.0;
    let pod_x = (VIRTUAL_WIDTH - pod_w) * 0.5;
    let pod_y = board_y + 5.0;

    draw_rectangle(pod_x, pod_y, pod_w, pod_h, Color::new(0.04, 0.06, 0.12, 0.95));
    draw_rectangle_lines(pod_x, pod_y, pod_w, pod_h, 1.0, Color::new(0.3, 0.6, 0.95, 0.4));

    // Match Mode Header inside Score Pod
    let mode_str = match logic.rule_mode {
        RuleMode::Modern => format!("RALLY TO {}", logic.score_to_win),
        RuleMode::Classic => format!("FIRST TO {} (SIDE-OUT)", logic.score_to_win),
    };
    let mode_dim = measure_text(&mode_str, None, 10, 1.0);
    draw_text(&mode_str, (VIRTUAL_WIDTH - mode_dim.width) * 0.5, pod_y + 13.0, 10.0, Color::new(0.4, 0.85, 1.0, 0.85));

    // LED Score Display inside Pod
    let score_str = format!("{}  :  {}", logic.scores[0], logic.scores[1]);
    let score_dim = measure_text(&score_str, None, 32, 1.0);
    draw_text(
        &score_str,
        (VIRTUAL_WIDTH - score_dim.width) * 0.5,
        pod_y + 42.0,
        32.0,
        Color::new(1.0, 0.95, 0.35, 1.0),
    );

    // Left Player Slot: x from board_x + 12 to pod_x - 10 (available width ~ 238px)
    let left_max_w = pod_x - board_x - 45.0;
    let (l_core, _, _) = left_theme.colors();
    let l_pip_x = board_x + 22.0;
    let l_pip_y = board_y + board_h * 0.5;
    draw_circle(l_pip_x, l_pip_y, 11.0, l_core);

    if logic.serving_player == PlayerSide::Left {
        let pulse = (time * 6.0).sin().abs() * 0.4 + 0.6;
        draw_circle(l_pip_x, l_pip_y, 15.0, Color::new(1.0, 0.85, 0.1, pulse * 0.7));
    }

    // Clip / truncate Left Name if too long so it NEVER collides with score pod
    let mut l_display = left_name.to_string();
    while measure_text(&l_display, None, 18, 1.0).width > left_max_w && l_display.len() > 4 {
        l_display.pop();
    }
    if l_display.len() < left_name.len() {
        l_display.push_str("..");
    }
    draw_text(&l_display, board_x + 40.0, board_y + 37.0, 18.0, WHITE);

    // Right Player Slot: x from pod_x + pod_w + 10 to board_x + board_w - 12
    let right_max_w = board_x + board_w - (pod_x + pod_w) - 45.0;
    let (r_core, _, _) = right_theme.colors();
    let r_pip_x = board_x + board_w - 22.0;
    let r_pip_y = board_y + board_h * 0.5;
    draw_circle(r_pip_x, r_pip_y, 11.0, r_core);

    if logic.serving_player == PlayerSide::Right {
        let pulse = (time * 6.0).sin().abs() * 0.4 + 0.6;
        draw_circle(r_pip_x, r_pip_y, 15.0, Color::new(1.0, 0.85, 0.1, pulse * 0.7));
    }

    let mut r_display = right_name.to_string();
    while measure_text(&r_display, None, 18, 1.0).width > right_max_w && r_display.len() > 4 {
        r_display.pop();
    }
    if r_display.len() < right_name.len() {
        r_display.push_str("..");
    }
    let r_w = measure_text(&r_display, None, 18, 1.0).width;
    draw_text(&r_display, r_pip_x - 18.0 - r_w, board_y + 37.0, 18.0, WHITE);

    // Floating Rally counter pill below HUD if rally > 1
    if logic.stats.current_rally_hits > 1 {
        let rally_str = format!("RALLY: {}", logic.stats.current_rally_hits);
        let rally_dim = measure_text(&rally_str, None, 13, 1.0);
        let rw = rally_dim.width + 16.0;
        let rx = (VIRTUAL_WIDTH - rw) * 0.5;
        let ry = board_y + board_h + 6.0;

        draw_rectangle(rx, ry, rw, 20.0, Color::new(0.08, 0.12, 0.22, 0.9));
        draw_rectangle_lines(rx, ry, rw, 20.0, 1.0, Color::new(0.2, 0.85, 1.0, 0.6));
        draw_text(&rally_str, rx + 8.0, ry + 14.5, 13.0, Color::new(0.4, 0.95, 1.0, 1.0));
    }

    // Online Serve Countdown Pill
    if let Some(cd) = serve_countdown {
        if cd > 0.0 {
            let cd_sec = cd.ceil() as i32;
            let cd_str = if cd_sec <= 1 {
                "SERVE NOW!".to_string()
            } else {
                format!("SERVE IN {}s", cd_sec)
            };
            let cd_dim = measure_text(&cd_str, None, 16, 1.0);
            let cw = cd_dim.width + 24.0;
            let cx = (VIRTUAL_WIDTH - cw) * 0.5;
            let cy = board_y + board_h + 30.0;

            let pulse = (cd * 8.0).sin().abs() * 0.25 + 0.75;
            draw_rectangle(cx, cy, cw, 26.0, Color::new(0.12, 0.05, 0.02, 0.92));
            draw_rectangle_lines(cx, cy, cw, 26.0, 1.5, Color::new(1.0, 0.7, 0.1, pulse));
            draw_text(&cd_str, cx + 12.0, cy + 18.0, 16.0, Color::new(1.0, 0.85, 0.2, pulse));
        }
    }

    // Off-Screen High Ball Indicator (only active in Indicator mode: 0)
    if ball_pos.y < 30.0 && high_ball_camera == 0 {
        let norm_x = (ball_pos.x / court_width) * VIRTUAL_WIDTH;
        let bx = norm_x.clamp(25.0, VIRTUAL_WIDTH - 25.0);
        let pulse = (time * 6.0).sin().abs() * 0.3 + 0.7;

        // Vertical laser projection line from top of screen down into the court
        draw_line(bx, 0.0, bx, 500.0, 1.5, Color::new(1.0, 0.85, 0.2, 0.22));

        // Top edge downward pointer chevron
        let chev_col = Color::new(1.0, 0.85, 0.15, pulse);
        draw_triangle(
            Vec2::new(bx - 12.0, 2.0),
            Vec2::new(bx + 12.0, 2.0),
            Vec2::new(bx, 15.0),
            chev_col,
        );

        // Altitude badge readout if above view
        let alt = (-ball_pos.y).max(0.0) as i32;
        let alt_str = format!("▲ {}m", alt / 10);
        let dim = measure_text(&alt_str, None, 12, 1.0);
        let bw = dim.width + 12.0;
        let bx_box = bx - bw * 0.5;
        draw_rectangle(bx_box, 16.0, bw, 16.0, Color::new(0.06, 0.08, 0.15, 0.92));
        draw_rectangle_lines(bx_box, 16.0, bw, 16.0, 1.0, Color::new(1.0, 0.8, 0.2, 0.85));
        draw_text(&alt_str, bx_box + 6.0, 28.0, 12.0, Color::new(1.0, 0.95, 0.4, 1.0));
    }
}

pub fn render_main_menu(
    game_mode: GameMode,
    items: &[MenuItem],
    player1_name: &str,
    country_code: &str,
    difficulty: AiDifficulty,
    elo_range: EloSearchRange,
    rule_mode: RuleMode,
    score_to_win: i32,
    court_width_factor: f32,
    aspect: f32,
    arena: ArenaType,
    left_theme: BlobTheme,
    right_theme: BlobTheme,
    high_ball_camera: usize,
    fullscreen: bool,
    selected_item: usize,
    virt_mouse: Option<Vec2>,
    time: f32,
) {
    let max_court_w = (600.0 * aspect).max(800.0);
    let current_court_width = (800.0 + (max_court_w - 800.0) * court_width_factor.clamp(0.0, 1.0)).round();
    let is_sizing_court = items.get(selected_item) == Some(&MenuItem::CourtSize);

    // Fullscreen backdrop dimming (softened when sizing court so visualizer in back is crystal clear)
    let ui_cam_w = VIRTUAL_HEIGHT * aspect;
    let ui_cam_x0 = (VIRTUAL_WIDTH - ui_cam_w) * 0.5;
    let dim_alpha = if is_sizing_court { 0.28 } else { 0.72 };
    draw_rectangle(ui_cam_x0, 0.0, ui_cam_w, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, dim_alpha));

    // Glowing 2026 Title
    let title = "BLOBBY VOLLEY 3";
    let title_dim = measure_text(title, None, 50, 1.0);
    let tx = (VIRTUAL_WIDTH - title_dim.width) * 0.5;
    let ty = 66.0;

    // Title shadow and glow
    draw_text(title, tx + 2.0, ty + 2.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.6));
    draw_text(title, tx, ty, 50.0, Color::new(0.15, 0.85, 1.0, 1.0));

    let subtitle = "REMASTERED - 2026 EDITION";
    let sub_dim = measure_text(subtitle, None, 16, 1.0);
    draw_text(
        subtitle,
        (VIRTUAL_WIDTH - sub_dim.width) * 0.5,
        ty + 22.0,
        16.0,
        Color::new(1.0, 0.85, 0.3, 0.9),
    );

    // Menu Card (frosted glass transparency when sizing court)
    let card_w = 560.0;
    let card_h = 430.0;
    let card_x = 24.0;
    let card_y = 115.0;

    let card_bg = if is_sizing_court {
        Color::new(0.06, 0.09, 0.16, 0.38)
    } else {
        Color::new(0.08, 0.1, 0.18, 0.92)
    };
    let card_border = if is_sizing_court {
        Color::new(0.2, 0.85, 1.0, 0.85)
    } else {
        Color::new(0.3, 0.5, 0.85, 0.5)
    };

    draw_rectangle(card_x, card_y, card_w, card_h, card_bg);
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.5, card_border);

    let layouts = get_main_menu_layout(items);

    let profile_display = if country_code.is_empty() {
        player1_name.to_string()
    } else {
        format!("{} {}", country_badge(country_code), player1_name)
    };

    for layout in &layouts {
        let i = layout.index;
        let is_selected = i == selected_item;
        let (label_opt, val) = match layout.item {
            MenuItem::StartMatch => {
                let action_text = match game_mode {
                    GameMode::QuickMatch => "FIND ONLINE MATCH  [ Press Space / Enter ]",
                    GameMode::HostOnline => "HOST ONLINE ROOM  [ Press Space / Enter ]",
                    GameMode::JoinOnline => "JOIN ONLINE ROOM  [ Press Space / Enter ]",
                    GameMode::ServerBrowser => "OPEN SERVER BROWSER  [ Press Space / Enter ]",
                    _ => "START MATCH  [ Press Space / Enter ]",
                };
                (None, action_text.to_string())
            }
            MenuItem::GameMode => (Some("Game Mode:"), game_mode.name().to_string()),
            MenuItem::PlayerProfile => (Some("Player Profile:"), profile_display.clone()),
            MenuItem::AiDifficulty => (Some("AI Difficulty:"), difficulty.name().to_string()),
            MenuItem::MatchmakingRange => (Some("Matchmaking Range:"), elo_range.name().to_string()),
            MenuItem::Rules => (Some("Rules:"), rule_mode.name().to_string()),
            MenuItem::PointsToWin => (Some("Points to Win:"), format!("{}", score_to_win)),
            MenuItem::CourtSize => {
                let lbl = if court_width_factor <= 0.01 {
                    "Court Width (Default):"
                } else if court_width_factor >= 0.99 {
                    "Court Width (Screen Borders):"
                } else {
                    "Court Width:"
                };
                (Some(lbl), format!("{:.0}px", current_court_width))
            }
            MenuItem::Arena => (Some("Arena:"), arena.name().to_string()),
            MenuItem::P1Color => (Some("P1 Color:"), left_theme.name().to_string()),
            MenuItem::P2Color => (Some("P2 Color:"), right_theme.name().to_string()),
            MenuItem::HighBallCamera => (Some("High Ball Cam:"), crate::config::HighBallCamera::from_index(high_ball_camera).name().to_string()),
            MenuItem::Fullscreen => (Some("Fullscreen:"), if fullscreen { "ENABLED".to_string() } else { "DISABLED".to_string() }),
        };

        // Draw row highlight if selected
        if is_selected {
            let pulse = (time * 5.0).sin().abs() * 0.15 + 0.85;
            draw_rectangle(
                layout.row_rect.x,
                layout.row_rect.y,
                layout.row_rect.w,
                layout.row_rect.h,
                Color::new(0.15, 0.45, 0.85, 0.35 * pulse),
            );
            draw_rectangle_lines(
                layout.row_rect.x,
                layout.row_rect.y,
                layout.row_rect.w,
                layout.row_rect.h,
                1.5,
                Color::new(0.2, 0.85, 1.0, 0.85),
            );
        }

        let text_y = layout.row_rect.y + layout.row_rect.h * 0.5 + 6.0;

        if let Some(label) = label_opt {
            // Setting Row: Label on left
            let label_col = if is_selected { WHITE } else { Color::new(0.8, 0.85, 0.92, 0.85) };
            draw_text(label, card_x + 30.0, text_y, 18.0, label_col);

            // Left arrow button `<`
            if let Some(l_rect) = layout.left_arrow_rect {
                let is_hovered = virt_mouse.map_or(false, |m| l_rect.contains(m));
                let bg_col = if is_hovered {
                    Color::new(0.2, 0.7, 1.0, 0.6)
                } else {
                    Color::new(0.12, 0.2, 0.32, 0.7)
                };
                let border_col = if is_hovered {
                    Color::new(0.4, 0.95, 1.0, 1.0)
                } else {
                    Color::new(0.3, 0.6, 0.9, 0.6)
                };

                draw_rectangle(l_rect.x, l_rect.y, l_rect.w, l_rect.h, bg_col);
                draw_rectangle_lines(l_rect.x, l_rect.y, l_rect.w, l_rect.h, 1.0, border_col);
                draw_text("<", l_rect.x + 8.0, l_rect.y + 19.0, 20.0, if is_hovered { WHITE } else { Color::new(0.3, 0.9, 1.0, 0.9) });
            }

            // Right arrow button `>`
            if let Some(r_rect) = layout.right_arrow_rect {
                let is_hovered = virt_mouse.map_or(false, |m| r_rect.contains(m));
                let bg_col = if is_hovered {
                    Color::new(0.2, 0.7, 1.0, 0.6)
                } else {
                    Color::new(0.12, 0.2, 0.32, 0.7)
                };
                let border_col = if is_hovered {
                    Color::new(0.4, 0.95, 1.0, 1.0)
                } else {
                    Color::new(0.3, 0.6, 0.9, 0.6)
                };

                draw_rectangle(r_rect.x, r_rect.y, r_rect.w, r_rect.h, bg_col);
                draw_rectangle_lines(r_rect.x, r_rect.y, r_rect.w, r_rect.h, 1.0, border_col);
                draw_text(">", r_rect.x + 8.0, r_rect.y + 19.0, 20.0, if is_hovered { WHITE } else { Color::new(0.3, 0.9, 1.0, 0.9) });
            }

            // Value representation between arrow buttons
            if let Some(s_rect) = layout.slider_rect {
                // Interactive Schieberegler (Slider)
                let bar_h = 6.0;
                let bar_y = s_rect.y + (s_rect.h - bar_h) * 0.5 + 4.0;
                let track_x = s_rect.x;
                let track_w = s_rect.w;

                // Value readout centered above slider track
                let val_str = if court_width_factor <= 0.01 {
                    "800px (Default)".to_string()
                } else if court_width_factor >= 0.99 {
                    format!("{:.0}px (Window Border)", current_court_width)
                } else {
                    format!("{:.0}px", current_court_width)
                };
                let val_dim = measure_text(&val_str, None, 13, 1.0);
                draw_text(
                    &val_str,
                    track_x + (track_w - val_dim.width) * 0.5,
                    s_rect.y + 4.0,
                    13.0,
                    if is_selected { Color::new(1.0, 0.95, 0.4, 1.0) } else { Color::new(0.85, 0.9, 1.0, 0.9) },
                );

                // Track Slot Background
                draw_rectangle(track_x, bar_y, track_w, bar_h, Color::new(0.1, 0.14, 0.22, 0.95));
                draw_rectangle_lines(track_x, bar_y, track_w, bar_h, 1.0, Color::new(0.25, 0.4, 0.6, 0.7));

                // Illuminated Active Progress Bar
                let fill_w = track_w * court_width_factor.clamp(0.0, 1.0);
                if fill_w > 0.0 {
                    draw_rectangle(track_x, bar_y, fill_w, bar_h, Color::new(0.2, 0.85, 1.0, 0.9));
                }

                // Slider Thumb Knob (Schieberegler handle)
                let thumb_x = track_x + fill_w;
                let thumb_y = bar_y + bar_h * 0.5;
                let is_track_hovered = virt_mouse.map_or(false, |m| s_rect.contains(m));
                let thumb_rad = if is_track_hovered { 8.5 } else { 7.0 };

                draw_circle(thumb_x, thumb_y, thumb_rad + 3.0, Color::new(0.2, 0.85, 1.0, 0.35));
                draw_circle(thumb_x, thumb_y, thumb_rad, Color::new(1.0, 0.95, 0.35, 1.0));
                draw_circle_lines(thumb_x, thumb_y, thumb_rad, 1.2, WHITE);
            } else if let (Some(l_rect), Some(r_rect)) = (layout.left_arrow_rect, layout.right_arrow_rect) {
                // Value text centered between the two arrow buttons for normal dropdown rows
                let val_dim = measure_text(&val, None, 18, 1.0);
                let center_x = (l_rect.x + l_rect.w + r_rect.x) * 0.5;
                let val_x = center_x - val_dim.width * 0.5;
                let val_col = if is_selected {
                    Color::new(1.0, 0.95, 0.4, 1.0)
                } else {
                    Color::new(0.85, 0.9, 1.0, 0.9)
                };
                if layout.item == MenuItem::PlayerProfile && !country_code.is_empty() {
                    draw_mini_flag(country_code, val_x - 24.0, text_y - 12.0, 18.0, 12.0);
                }
                draw_text(&val, val_x, text_y, 18.0, val_col);
            }
        } else {
            // Action button (START MATCH)
            let col = if is_selected {
                Color::new(1.0, 0.95, 0.3, 1.0)
            } else {
                Color::new(0.3, 0.9, 1.0, 1.0)
            };
            let dim = measure_text(&val, None, 21, 1.0);
            let tx = (VIRTUAL_WIDTH - dim.width) * 0.5;
            draw_text(&val, tx, text_y, 21.0, col);
        }
    }

    // Color Preview Showcase Pod on the right of the menu
    let pod_x = 600.0;
    let pod_y = card_y;
    let pod_w = 176.0;
    let pod_h = card_h;

    let pod_bg = if is_sizing_court {
        Color::new(0.06, 0.09, 0.16, 0.38)
    } else {
        Color::new(0.08, 0.1, 0.18, 0.92)
    };
    let pod_border = if is_sizing_court {
        Color::new(0.2, 0.85, 1.0, 0.85)
    } else {
        Color::new(0.3, 0.5, 0.85, 0.5)
    };

    draw_rectangle(pod_x, pod_y, pod_w, pod_h, pod_bg);
    draw_rectangle_lines(pod_x, pod_y, pod_w, pod_h, 1.5, pod_border);

    let mouse_target = virt_mouse.unwrap_or(Vec2::new(pod_x - 100.0, pod_y + 200.0));

    // Section 1: Player 1 Blob
    let is_p1_active = items.get(selected_item) == Some(&MenuItem::P1Color);
    let p1_label = format!("P1: {}", left_theme.name());
    let p1_dim = measure_text(&p1_label, None, 13, 1.0);
    draw_text(&p1_label, pod_x + (pod_w - p1_dim.width) * 0.5, pod_y + 28.0, 13.0, if is_p1_active { Color::new(1.0, 0.9, 0.3, 1.0) } else { Color::new(0.8, 0.85, 0.92, 0.85) });

    let p1_center = Vec2::new(pod_x + pod_w * 0.5, pod_y + 108.0);
    if is_p1_active {
        let pulse = (time * 5.0).sin().abs() * 0.2 + 0.8;
        draw_circle_lines(p1_center.x, p1_center.y - 10.0, 48.0, 1.5, Color::new(0.2, 0.85, 1.0, pulse * 0.7));
    }
    crate::render::blob::render_preview_blobby(p1_center, 1.18, PlayerSide::Left, left_theme, mouse_target, time);

    // Section Divider
    draw_line(pod_x + 20.0, pod_y + 212.0, pod_x + pod_w - 20.0, pod_y + 212.0, 1.0, Color::new(0.3, 0.5, 0.85, 0.35));

    // Section 2: Player 2 / CPU Blob
    let is_p2_active = items.get(selected_item) == Some(&MenuItem::P2Color);
    let p2_prefix = if game_mode == GameMode::TwoPlayerLocal { "P2" } else { "CPU" };
    let p2_label = format!("{}: {}", p2_prefix, right_theme.name());
    let p2_dim = measure_text(&p2_label, None, 13, 1.0);
    draw_text(&p2_label, pod_x + (pod_w - p2_dim.width) * 0.5, pod_y + 236.0, 13.0, if is_p2_active { Color::new(1.0, 0.9, 0.3, 1.0) } else { Color::new(0.8, 0.85, 0.92, 0.85) });

    let p2_center = Vec2::new(pod_x + pod_w * 0.5, pod_y + 316.0);
    if is_p2_active {
        let pulse = (time * 5.0).sin().abs() * 0.2 + 0.8;
        draw_circle_lines(p2_center.x, p2_center.y - 10.0, 48.0, 1.5, Color::new(0.2, 0.85, 1.0, pulse * 0.7));
    }
    crate::render::blob::render_preview_blobby(p2_center, 1.18, PlayerSide::Right, right_theme, mouse_target, time + 1.5);

    // Footer Controls Guide
    let guide = "Controls: [Left/Right] Cycle | [Enter] Confirm & Start | [L] Leaderboards | [ESC] Quit Game | [F11] Fullscreen";
    let guide_dim = measure_text(guide, None, 13, 1.0);
    draw_text(
        guide,
        (VIRTUAL_WIDTH - guide_dim.width) * 0.5,
        VIRTUAL_HEIGHT - 18.0,
        13.0,
        Color::new(0.7, 0.75, 0.85, 0.8),
    );
}

pub fn draw_mini_flag(code: &str, x: f32, y: f32, w: f32, h: f32) {
    let c = code.trim().to_ascii_uppercase();
    draw_rectangle(x, y, w, h, Color::new(0.08, 0.12, 0.2, 1.0));

    match c.as_str() {
        "DE" => { // Germany: Black, Red, Gold horizontal
            let sh = h / 3.0;
            draw_rectangle(x, y, w, sh, Color::new(0.1, 0.1, 0.1, 1.0));
            draw_rectangle(x, y + sh, w, sh, Color::new(0.85, 0.1, 0.1, 1.0));
            draw_rectangle(x, y + sh * 2.0, w, h - sh * 2.0, Color::new(1.0, 0.8, 0.1, 1.0));
        }
        "FR" => { // France: Blue, White, Red vertical
            let sw = w / 3.0;
            draw_rectangle(x, y, sw, h, Color::new(0.0, 0.33, 0.65, 1.0));
            draw_rectangle(x + sw, y, sw, h, WHITE);
            draw_rectangle(x + sw * 2.0, y, w - sw * 2.0, h, Color::new(0.9, 0.15, 0.2, 1.0));
        }
        "IT" => { // Italy: Green, White, Red vertical
            let sw = w / 3.0;
            draw_rectangle(x, y, sw, h, Color::new(0.0, 0.58, 0.27, 1.0));
            draw_rectangle(x + sw, y, sw, h, WHITE);
            draw_rectangle(x + sw * 2.0, y, w - sw * 2.0, h, Color::new(0.85, 0.18, 0.2, 1.0));
        }
        "ES" => { // Spain: Red, Yellow (double), Red horizontal
            draw_rectangle(x, y, w, h * 0.25, Color::new(0.8, 0.1, 0.15, 1.0));
            draw_rectangle(x, y + h * 0.25, w, h * 0.5, Color::new(1.0, 0.78, 0.1, 1.0));
            draw_rectangle(x, y + h * 0.75, w, h * 0.25, Color::new(0.8, 0.1, 0.15, 1.0));
        }
        "US" => { // USA: Red/White stripes + Blue canton
            let sh = h / 7.0;
            for s in 0..7 {
                let col = if s % 2 == 0 { Color::new(0.8, 0.15, 0.2, 1.0) } else { WHITE };
                draw_rectangle(x, y + s as f32 * sh, w, sh + 0.5, col);
            }
            draw_rectangle(x, y, w * 0.42, h * 0.58, Color::new(0.0, 0.18, 0.45, 1.0));
        }
        "GB" | "UK" => { // UK: Union Jack
            draw_rectangle(x, y, w, h, Color::new(0.0, 0.15, 0.45, 1.0));
            draw_line(x, y, x + w, y + h, 2.0, WHITE);
            draw_line(x, y + h, x + w, y, 2.0, WHITE);
            draw_line(x + w * 0.5, y, x + w * 0.5, y + h, 3.0, WHITE);
            draw_line(x, y + h * 0.5, x + w, y + h * 0.5, 3.0, WHITE);
            draw_line(x + w * 0.5, y, x + w * 0.5, y + h, 1.5, Color::new(0.85, 0.1, 0.15, 1.0));
            draw_line(x, y + h * 0.5, x + w, y + h * 0.5, 1.5, Color::new(0.85, 0.1, 0.15, 1.0));
        }
        "NL" => { // Netherlands: Red, White, Blue horizontal
            let sh = h / 3.0;
            draw_rectangle(x, y, w, sh, Color::new(0.8, 0.15, 0.2, 1.0));
            draw_rectangle(x, y + sh, w, sh, WHITE);
            draw_rectangle(x, y + sh * 2.0, w, h - sh * 2.0, Color::new(0.1, 0.25, 0.6, 1.0));
        }
        "BE" => { // Belgium: Black, Yellow, Red vertical
            let sw = w / 3.0;
            draw_rectangle(x, y, sw, h, Color::new(0.1, 0.1, 0.1, 1.0));
            draw_rectangle(x + sw, y, sw, h, Color::new(0.95, 0.85, 0.15, 1.0));
            draw_rectangle(x + sw * 2.0, y, w - sw * 2.0, h, Color::new(0.9, 0.15, 0.2, 1.0));
        }
        "PL" => { // Poland: White, Red horizontal
            draw_rectangle(x, y, w, h * 0.5, WHITE);
            draw_rectangle(x, y + h * 0.5, w, h * 0.5, Color::new(0.85, 0.15, 0.25, 1.0));
        }
        "UA" => { // Ukraine: Blue, Yellow horizontal
            draw_rectangle(x, y, w, h * 0.5, Color::new(0.0, 0.4, 0.8, 1.0));
            draw_rectangle(x, y + h * 0.5, w, h * 0.5, Color::new(1.0, 0.85, 0.1, 1.0));
        }
        "SE" => { // Sweden: Blue with yellow cross
            draw_rectangle(x, y, w, h, Color::new(0.0, 0.35, 0.65, 1.0));
            draw_line(x + w * 0.35, y, x + w * 0.35, y + h, 2.5, Color::new(1.0, 0.8, 0.1, 1.0));
            draw_line(x, y + h * 0.5, x + w, y + h * 0.5, 2.5, Color::new(1.0, 0.8, 0.1, 1.0));
        }
        "JP" => { // Japan: White with red sun circle
            draw_rectangle(x, y, w, h, WHITE);
            draw_circle(x + w * 0.5, y + h * 0.5, h * 0.3, Color::new(0.8, 0.1, 0.2, 1.0));
        }
        "BR" => { // Brazil: Green with yellow diamond & blue circle
            draw_rectangle(x, y, w, h, Color::new(0.0, 0.58, 0.25, 1.0));
            draw_triangle(Vec2::new(x + w * 0.5, y + 1.0), Vec2::new(x + 1.5, y + h * 0.5), Vec2::new(x + w * 0.5, y + h - 1.0), Color::new(1.0, 0.85, 0.1, 1.0));
            draw_triangle(Vec2::new(x + w * 0.5, y + 1.0), Vec2::new(x + w - 1.5, y + h * 0.5), Vec2::new(x + w * 0.5, y + h - 1.0), Color::new(1.0, 0.85, 0.1, 1.0));
            draw_circle(x + w * 0.5, y + h * 0.5, h * 0.22, Color::new(0.1, 0.2, 0.55, 1.0));
        }
        "CH" => { // Switzerland: Red with white cross
            draw_rectangle(x, y, w, h, Color::new(0.85, 0.1, 0.15, 1.0));
            draw_rectangle(x + w * 0.5 - 1.5, y + h * 0.2, 3.0, h * 0.6, WHITE);
            draw_rectangle(x + w * 0.2, y + h * 0.5 - 1.5, w * 0.6, 3.0, WHITE);
        }
        "AT" => { // Austria: Red, White, Red horizontal
            let sh = h / 3.0;
            draw_rectangle(x, y, w, sh, Color::new(0.85, 0.15, 0.2, 1.0));
            draw_rectangle(x, y + sh, w, sh, WHITE);
            draw_rectangle(x, y + sh * 2.0, w, h - sh * 2.0, Color::new(0.85, 0.15, 0.2, 1.0));
        }
        "CA" => { // Canada: Red, White, Red vertical
            draw_rectangle(x, y, w * 0.25, h, Color::new(0.85, 0.1, 0.15, 1.0));
            draw_rectangle(x + w * 0.25, y, w * 0.5, h, WHITE);
            draw_rectangle(x + w * 0.75, y, w * 0.25, h, Color::new(0.85, 0.1, 0.15, 1.0));
            draw_circle(x + w * 0.5, y + h * 0.5, 2.5, Color::new(0.85, 0.1, 0.15, 1.0));
        }
        _ => { // Global / other: stylized globe or badge code
            draw_rectangle(x, y, w, h, Color::new(0.12, 0.16, 0.24, 1.0));
            if c.is_empty() {
                draw_circle_lines(x + w * 0.5, y + h * 0.5, h * 0.35, 1.2, Color::new(0.2, 0.8, 1.0, 0.9));
                draw_line(x + 2.0, y + h * 0.5, x + w - 2.0, y + h * 0.5, 1.0, Color::new(0.2, 0.8, 1.0, 0.9));
            } else {
                let fs = (h * 0.7).max(8.0);
                let dim = measure_text(&c, None, fs as u16, 1.0);
                draw_text(&c, x + (w - dim.width) * 0.5, y + h * 0.75, fs, Color::new(0.3, 0.9, 1.0, 1.0));
            }
        }
    }
    // Clean flag border
    draw_rectangle_lines(x, y, w, h, 1.0, Color::new(0.3, 0.45, 0.65, 0.7));
}

pub fn render_player_setup(
    p1_name: &str,
    country_code: &str,
    p2_name: &str,
    is_two_player: bool,
    active_field: usize,
    search_query: &str,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.85));

    let card_w = 560.0;
    let card_h = if is_two_player { 390.0 } else { 330.0 };
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.95));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.8));

    let title = "PLAYER PROFILE & IDENTITY";
    let title_dim = measure_text(title, None, 30, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 42.0, 30.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let sub = "Display names and country flags are saved for local & online play";
    let sub_dim = measure_text(sub, None, 14, 1.0);
    draw_text(sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 68.0, 14.0, Color::new(0.75, 0.8, 0.9, 0.8));

    let cursor = if (time * 3.5) as i32 % 2 == 0 { "_" } else { "" };

    // Field 0: Player 1 Name
    let f0_y = card_y + 110.0;
    draw_text("Player 1 Name:", card_x + 40.0, f0_y - 8.0, 17.0, WHITE);
    let box0_rect = Rect::new(card_x + 40.0, f0_y, card_w - 80.0, 36.0);
    draw_rectangle(box0_rect.x, box0_rect.y, box0_rect.w, box0_rect.h, Color::new(0.05, 0.08, 0.14, 0.95));
    let f0_border = if active_field == 0 { Color::new(0.2, 0.9, 1.0, 1.0) } else { Color::new(0.3, 0.45, 0.65, 0.6) };
    draw_rectangle_lines(box0_rect.x, box0_rect.y, box0_rect.w, box0_rect.h, 1.5, f0_border);
    let f0_text = if active_field == 0 { format!("{}{}", p1_name, cursor) } else { p1_name.to_string() };
    draw_text(&f0_text, box0_rect.x + 12.0, box0_rect.y + 25.0, 20.0, Color::new(1.0, 0.95, 0.35, 1.0));

    let mut next_y = f0_y + 68.0;

    // Field 1: Player 2 Name (only if 2-player mode)
    if is_two_player {
        let f1_y = next_y;
        draw_text("Player 2 Name:", card_x + 40.0, f1_y - 8.0, 17.0, WHITE);
        let box1_rect = Rect::new(card_x + 40.0, f1_y, card_w - 80.0, 36.0);
        draw_rectangle(box1_rect.x, box1_rect.y, box1_rect.w, box1_rect.h, Color::new(0.05, 0.08, 0.14, 0.95));
        let f1_border = if active_field == 1 { Color::new(0.2, 0.9, 1.0, 1.0) } else { Color::new(0.3, 0.45, 0.65, 0.6) };
        draw_rectangle_lines(box1_rect.x, box1_rect.y, box1_rect.w, box1_rect.h, 1.5, f1_border);
        let f1_text = if active_field == 1 { format!("{}{}", p2_name, cursor) } else { p2_name.to_string() };
        draw_text(&f1_text, box1_rect.x + 12.0, box1_rect.y + 25.0, 20.0, Color::new(1.0, 0.95, 0.35, 1.0));
        next_y += 68.0;
    }

    // Country / Flag Field
    let country_field_idx = if is_two_player { 2 } else { 1 };
    let fc_y = next_y;
    draw_text("Country / Flag Badge (Optional):", card_x + 40.0, fc_y - 8.0, 17.0, WHITE);

    // Live search query badge if active
    if active_field == country_field_idx && !search_query.is_empty() {
        let q_str = format!("Jump to: \"{}\"", search_query);
        let q_dim = measure_text(&q_str, None, 14, 1.0);
        draw_text(&q_str, card_x + card_w - 40.0 - q_dim.width, fc_y - 8.0, 14.0, Color::new(1.0, 0.9, 0.35, 1.0));
    }

    let boxc_rect = Rect::new(card_x + 40.0, fc_y, card_w - 80.0, 36.0);
    draw_rectangle(boxc_rect.x, boxc_rect.y, boxc_rect.w, boxc_rect.h, Color::new(0.05, 0.08, 0.14, 0.95));
    let fc_border = if active_field == country_field_idx { Color::new(0.2, 0.9, 1.0, 1.0) } else { Color::new(0.3, 0.45, 0.65, 0.6) };
    draw_rectangle_lines(boxc_rect.x, boxc_rect.y, boxc_rect.w, boxc_rect.h, 1.5, fc_border);

    // Mini Flag on the left of the field
    draw_mini_flag(country_code, boxc_rect.x + 10.0, boxc_rect.y + 7.0, 32.0, 22.0);

    let display_str = format!("<  {}  >", country_display(country_code));
    let display_dim = measure_text(&display_str, None, 18, 1.0);
    draw_text(
        &display_str,
        boxc_rect.x + (boxc_rect.w - display_dim.width) * 0.5,
        boxc_rect.y + 25.0,
        18.0,
        Color::new(0.3, 0.9, 1.0, 1.0),
    );

    // Confirm Prompt
    let btn_y = card_y + card_h - 32.0;
    let hint = "Type letters to jump (e.g. 'GER') | Left / Right: Cycle | Tab: Next | Enter: Confirm";
    let hint_dim = measure_text(hint, None, 13, 1.0);
    draw_text(hint, (VIRTUAL_WIDTH - hint_dim.width) * 0.5, btn_y, 13.0, Color::new(0.3, 0.95, 1.0, 0.9));
}

pub fn render_server_browser(
    servers: &[ServerEntry],
    selected_idx: usize,
    is_refreshing: bool,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.88));

    let card_w = 680.0;
    let card_h = 440.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.95));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.75));

    let title = "PUBLIC DEDICATED SERVERS & MATCHMAKING";
    let title_dim = measure_text(title, None, 28, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 42.0, 28.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let subtitle = "Official Blobby Volley 2 Network & Remastered Community Rooms";
    let sub_dim = measure_text(subtitle, None, 14, 1.0);
    draw_text(subtitle, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 66.0, 14.0, Color::new(0.8, 0.85, 0.95, 0.8));

    // Table Header
    let table_y = card_y + 100.0;
    draw_rectangle(card_x + 20.0, table_y - 20.0, card_w - 40.0, 26.0, Color::new(0.12, 0.16, 0.26, 0.9));
    draw_text("SERVER NAME", card_x + 35.0, table_y - 2.0, 15.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("ENDPOINT", card_x + 360.0, table_y - 2.0, 15.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("STATUS / TYPE", card_x + 530.0, table_y - 2.0, 15.0, Color::new(0.4, 0.9, 1.0, 1.0));

    let row_h = 34.0;
    let list_start_y = table_y + 18.0;

    if servers.is_empty() {
        let msg = if is_refreshing { "Refreshing master server list..." } else { "No servers found. Press R to refresh." };
        let msg_dim = measure_text(msg, None, 18, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, list_start_y + 60.0, 18.0, Color::new(1.0, 0.85, 0.3, 1.0));
    } else {
        for (i, server) in servers.iter().enumerate().take(7) {
            let y = list_start_y + i as f32 * row_h;
            let is_sel = i == selected_idx;

            if is_sel {
                let pulse = (time * 5.0).sin().abs() * 0.15 + 0.85;
                draw_rectangle(card_x + 20.0, y - 16.0, card_w - 40.0, 30.0, Color::new(0.18, 0.45, 0.85, 0.4 * pulse));
                draw_rectangle_lines(card_x + 20.0, y - 16.0, card_w - 40.0, 30.0, 1.5, Color::new(0.2, 0.85, 1.0, 0.9));
            }

            let col = if is_sel { Color::new(1.0, 0.95, 0.4, 1.0) } else { WHITE };
            draw_text(&server.name, card_x + 35.0, y + 5.0, 16.0, col);

            let endpoint = format!("{}:{}", server.host, server.port);
            draw_text(&endpoint, card_x + 360.0, y + 5.0, 15.0, Color::new(0.85, 0.9, 1.0, 0.85));

            let status = server.ping_ms.map_or_else(|| server.server_type.clone(), |p| format!("{} ({}ms)", server.server_type, p));
            draw_text(&status, card_x + 530.0, y + 5.0, 14.0, Color::new(0.3, 1.0, 0.5, 0.9));
        }
    }

    // Notice about zero-cost matchmaking
    let note = "Tip: Remastered uses zero-cost direct P2P/LAN rooms. Host a match from menu anytime!";
    let note_dim = measure_text(note, None, 13, 1.0);
    draw_text(note, (VIRTUAL_WIDTH - note_dim.width) * 0.5, card_y + card_h - 48.0, 13.0, Color::new(0.8, 0.85, 0.95, 0.7));

    // Footer actions
    let actions = "[Enter / Click] Connect | [R] Refresh | [Esc] Return to Menu";
    let act_dim = measure_text(actions, None, 15, 1.0);
    draw_text(actions, (VIRTUAL_WIDTH - act_dim.width) * 0.5, card_y + card_h - 20.0, 15.0, Color::new(0.3, 0.9, 1.0, 0.95));
}

pub fn render_host_lobby(
    room_name: &str,
    port: u16,
    local_ip: &str,
    public_endpoint: Option<&str>,
    is_connected: bool,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.88));

    let card_w = 600.0;
    let card_h = 360.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.95));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.75));

    let title = "HOSTING ZERO-COST P2P MATCH";
    let title_dim = measure_text(title, None, 28, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 44.0, 28.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let sub = format!("Room: {}  (Port {})", room_name, port);
    let sub_dim = measure_text(&sub, None, 16, 1.0);
    draw_text(&sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 72.0, 16.0, Color::new(1.0, 0.9, 0.3, 0.9));

    // Section 1: Internet / Remote Direct Join (STUN)
    let y1 = card_y + 115.0;
    draw_rectangle(card_x + 30.0, y1 - 15.0, card_w - 60.0, 48.0, Color::new(0.05, 0.08, 0.14, 0.9));
    draw_rectangle_lines(card_x + 30.0, y1 - 15.0, card_w - 60.0, 48.0, 1.0, Color::new(0.2, 0.6, 0.9, 0.5));
    draw_text("Public Internet Endpoint (STUN Verified):", card_x + 44.0, y1 + 5.0, 14.0, Color::new(0.7, 0.8, 0.95, 0.85));

    if let Some(ep) = public_endpoint {
        draw_text(ep, card_x + 44.0, y1 + 25.0, 18.0, Color::new(0.3, 1.0, 0.5, 1.0));
        draw_text("[ Share with friends across internet ]", card_x + 240.0, y1 + 25.0, 14.0, Color::new(0.8, 0.85, 0.95, 0.7));
    } else {
        draw_text("Resolving STUN public endpoint...", card_x + 44.0, y1 + 25.0, 16.0, Color::new(1.0, 0.8, 0.3, 0.8));
    }

    // Section 2: Local Wi-Fi / LAN Auto-Discovery
    let y2 = card_y + 180.0;
    draw_rectangle(card_x + 30.0, y2 - 15.0, card_w - 60.0, 48.0, Color::new(0.05, 0.08, 0.14, 0.9));
    draw_rectangle_lines(card_x + 30.0, y2 - 15.0, card_w - 60.0, 48.0, 1.0, Color::new(0.2, 0.6, 0.9, 0.5));
    draw_text("Local Network (LAN Broadcast Active on Port 1235):", card_x + 44.0, y2 + 5.0, 14.0, Color::new(0.7, 0.8, 0.95, 0.85));
    let lan_info = format!("{}:{}", local_ip, port);
    draw_text(&lan_info, card_x + 44.0, y2 + 25.0, 18.0, Color::new(0.2, 0.9, 1.0, 1.0));
    draw_text("[ Auto-appears in opponent's Server Browser ]", card_x + 240.0, y2 + 25.0, 14.0, Color::new(0.8, 0.85, 0.95, 0.7));

    // Section 3: Connection Status
    let pulse = (time * 4.0).sin().abs() * 0.3 + 0.7;
    let y3 = card_y + 270.0;
    if is_connected {
        let msg = "OPPONENT CONNECTED! LAUNCHING MATCH...";
        let msg_dim = measure_text(msg, None, 20, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, y3, 20.0, Color::new(0.2, 1.0, 0.4, 1.0));
    } else {
        let msg = "Waiting for player to connect (via Server Browser or Direct IP)...";
        let msg_dim = measure_text(msg, None, 16, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, y3, 16.0, Color::new(1.0, 0.85, 0.3, pulse));
    }

    let cancel = "Press Escape to Cancel and Return to Menu";
    let cancel_dim = measure_text(cancel, None, 15, 1.0);
    draw_text(cancel, (VIRTUAL_WIDTH - cancel_dim.width) * 0.5, card_y + card_h - 20.0, 15.0, Color::new(0.7, 0.75, 0.85, 0.7));
}

pub fn render_join_screen(ip_input: &str, is_connecting: bool, error: Option<&str>, time: f32) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.85));

    let card_w = 520.0;
    let card_h = 320.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.8, 1.0, 0.7));

    let title = "DIRECT CONNECT TO SERVER";
    let title_dim = measure_text(title, None, 30, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 48.0, 30.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let label = "Enter Host Address (IP:Port):";
    draw_text(label, card_x + 40.0, card_y + 105.0, 20.0, WHITE);

    // Text Input Box
    let input_box_w = card_w - 80.0;
    let input_box_h = 44.0;
    let input_box_x = card_x + 40.0;
    let input_box_y = card_y + 125.0;

    draw_rectangle(input_box_x, input_box_y, input_box_w, input_box_h, Color::new(0.05, 0.08, 0.14, 0.95));
    draw_rectangle_lines(input_box_x, input_box_y, input_box_w, input_box_h, 2.0, Color::new(0.3, 0.85, 1.0, 0.85));

    let cursor = if (time * 3.0) as i32 % 2 == 0 { "_" } else { "" };
    let display_text = format!("{}{}", ip_input, cursor);
    draw_text(&display_text, input_box_x + 14.0, input_box_y + 30.0, 24.0, Color::new(1.0, 0.95, 0.3, 1.0));

    // Status or Error
    if is_connecting {
        let msg = "Connecting to server...";
        let msg_dim = measure_text(msg, None, 20, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, card_y + 215.0, 20.0, Color::new(1.0, 0.85, 0.2, 1.0));
    } else if let Some(err) = error {
        let err_dim = measure_text(err, None, 18, 1.0);
        draw_text(err, (VIRTUAL_WIDTH - err_dim.width) * 0.5, card_y + 215.0, 18.0, Color::new(1.0, 0.35, 0.35, 1.0));
    } else {
        let hint = "Press Enter to Connect";
        let hint_dim = measure_text(hint, None, 18, 1.0);
        draw_text(hint, (VIRTUAL_WIDTH - hint_dim.width) * 0.5, card_y + 215.0, 18.0, Color::new(0.4, 0.9, 1.0, 0.85));
    }

    let cancel = "Press Escape to Cancel";
    let cancel_dim = measure_text(cancel, None, 16, 1.0);
    draw_text(cancel, (VIRTUAL_WIDTH - cancel_dim.width) * 0.5, card_y + card_h - 25.0, 16.0, Color::new(0.7, 0.75, 0.85, 0.7));
}

pub fn render_chat_overlay(
    chat_log: &[(String, String, f32)], // (sender, text, timestamp)
    is_typing: bool,
    input_text: &str,
    time: f32,
) {
    let start_x = 24.0;
    let mut curr_y = 100.0;

    // Display recent chat messages (fade out after 8 seconds unless typing)
    for (sender, text, ts) in chat_log.iter().rev().take(6) {
        let age = time - ts;
        if age < 8.0 || is_typing {
            let alpha = if is_typing { 1.0 } else { (1.0 - (age - 6.0) / 2.0).clamp(0.0, 1.0) };
            let msg = format!("{}: {}", sender, text);

            let dim = measure_text(&msg, None, 16, 1.0);
            draw_rectangle(start_x - 6.0, curr_y - 14.0, dim.width + 12.0, 22.0, Color::new(0.05, 0.08, 0.15, 0.75 * alpha));
            draw_text(&msg, start_x, curr_y + 2.0, 16.0, Color::new(1.0, 1.0, 1.0, alpha));
            curr_y += 26.0;
        }
    }

    // If currently typing: show input bar at bottom
    if is_typing {
        let bar_w = 440.0;
        let bar_h = 36.0;
        let bar_x = 24.0;
        let bar_y = VIRTUAL_HEIGHT - 60.0;

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.06, 0.1, 0.18, 0.92));
        draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.5, Color::new(0.2, 0.85, 1.0, 0.9));

        let cursor = if (time * 3.0) as i32 % 2 == 0 { "_" } else { "" };
        let text_disp = format!("Chat: {}{}", input_text, cursor);
        draw_text(&text_disp, bar_x + 10.0, bar_y + 24.0, 18.0, Color::new(1.0, 0.95, 0.3, 1.0));
    }
}

pub fn render_pause_menu(selected_item: usize) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.65));

    let card_w = 340.0;
    let card_h = 240.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.16, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.5, Color::new(0.3, 0.6, 0.9, 0.6));

    let title = "PAUSED";
    let title_dim = measure_text(title, None, 36, 1.0);
    draw_text(
        title,
        (VIRTUAL_WIDTH - title_dim.width) * 0.5,
        card_y + 48.0,
        36.0,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );

    let items = ["RESUME MATCH", "RESTART SET", "MAIN MENU"];
    let start_y = card_y + 105.0;
    let row_h = 42.0;

    for (i, item) in items.iter().enumerate() {
        let is_sel = i == selected_item;
        let y = start_y + i as f32 * row_h;

        if is_sel {
            draw_rectangle(
                card_x + 20.0,
                y - 25.0,
                card_w - 40.0,
                34.0,
                Color::new(0.18, 0.45, 0.85, 0.5),
            );
            draw_rectangle_lines(
                card_x + 20.0,
                y - 25.0,
                card_w - 40.0,
                34.0,
                1.5,
                Color::new(0.2, 0.9, 1.0, 0.9),
            );
        }

        let dim = measure_text(item, None, 20, 1.0);
        let col = if is_sel { WHITE } else { Color::new(0.75, 0.8, 0.88, 0.8) };
        draw_text(item, (VIRTUAL_WIDTH - dim.width) * 0.5, y - 2.0, 20.0, col);
    }
}

pub fn render_matchmaking_queue(
    player_name: &str,
    country_code: &str,
    my_rating: f64,
    tolerance: f64,
    elo_range: EloSearchRange,
    arena: ArenaType,
    elapsed_sec: f32,
    rank_title: &str,
    active_challenges: &[ChallengeBroadcast],
    cross_offer: Option<&CrossArenaOffer>,
    time: f32,
) {
    // Backdrop dimming
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.88));

    let card_w = 660.0;
    let card_h = 490.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    // Glassmorphism card
    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.95));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.8));

    // Top glow line
    draw_line(
        card_x + 10.0,
        card_y + 1.0,
        card_x + card_w - 10.0,
        card_y + 1.0,
        2.0,
        Color::new(0.3, 0.9, 1.0, 0.9),
    );

    // Title & Subtitle
    let title = "RANKED QUICK MATCH";
    let title_dim = measure_text(title, None, 30, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 40.0, 30.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let sub = "Automated Glicko-2 Matchmaking & Live Challenge Radar";
    let sub_dim = measure_text(sub, None, 14, 1.0);
    draw_text(sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 62.0, 14.0, Color::new(1.0, 0.85, 0.35, 0.9));

    // Animated Radar Scanner Circle
    let radar_center_x = VIRTUAL_WIDTH * 0.5;
    let radar_center_y = card_y + 125.0;
    let radar_radius = 42.0;

    // Concentric radar rings
    draw_circle_lines(radar_center_x, radar_center_y, radar_radius, 1.5, Color::new(0.2, 0.6, 0.9, 0.4));
    draw_circle_lines(radar_center_x, radar_center_y, radar_radius * 0.65, 1.0, Color::new(0.2, 0.6, 0.9, 0.3));
    draw_circle_lines(radar_center_x, radar_center_y, radar_radius * 0.3, 1.0, Color::new(0.2, 0.6, 0.9, 0.25));

    // Expanding pulse waves
    let wave1 = (time * 1.5).fract();
    let wave2 = ((time * 1.5) + 0.5).fract();
    draw_circle_lines(radar_center_x, radar_center_y, radar_radius * wave1, 1.5, Color::new(0.3, 0.9, 1.0, (1.0 - wave1) * 0.8));
    draw_circle_lines(radar_center_x, radar_center_y, radar_radius * wave2, 1.5, Color::new(0.3, 0.9, 1.0, (1.0 - wave2) * 0.8));

    // Rotating scanner beam
    let angle = time * 3.5;
    let sweep_x = radar_center_x + angle.cos() * radar_radius;
    let sweep_y = radar_center_y + angle.sin() * radar_radius;
    draw_line(radar_center_x, radar_center_y, sweep_x, sweep_y, 2.0, Color::new(0.4, 1.0, 0.7, 0.9));
    draw_circle(radar_center_x, radar_center_y, 4.0, Color::new(0.4, 1.0, 0.7, 1.0));

    // Details Box
    let box_y = card_y + 180.0;
    let box_w = card_w - 50.0;
    let box_x = card_x + 25.0;
    draw_rectangle(box_x, box_y, box_w, 132.0, Color::new(0.05, 0.08, 0.15, 0.85));
    draw_rectangle_lines(box_x, box_y, box_w, 132.0, 1.0, Color::new(0.3, 0.5, 0.75, 0.5));

    let mins = (elapsed_sec / 60.0).floor() as u32;
    let secs = (elapsed_sec % 60.0).floor() as u32;

    let line1_label = "Player Profile:";
    let line1_val = format!("{} {} (Rating: {:.0} - {})", country_badge(country_code), player_name, my_rating, rank_title);

    let line2_label = "Search Range:";
    let line2_val = if elo_range == EloSearchRange::Unrestricted {
        "Unrestricted (Any Elo)  [ E ] Cycle".to_string()
    } else {
        format!("{:.0} - {:.0} (±{:.0} Rating, +1/3 per 5s)  [ E ] Cycle", (my_rating - tolerance).max(400.0), my_rating + tolerance, tolerance)
    };

    let line3_label = "Court / Arena:";
    let line3_val = format!("{}  [ A ] Cycle Court", arena.name());

    let line4_label = "Status:";
    let line4_val = format!("Seeking Match... ({:02}:{:02}) | Broadcasting Challenge", mins, secs);

    let draw_row = |lbl: &str, val: &str, y: f32| {
        draw_text(lbl, box_x + 18.0, y, 15.0, Color::new(0.7, 0.8, 0.9, 0.9));
        draw_text(val, box_x + 140.0, y, 15.0, Color::new(1.0, 0.95, 0.4, 1.0));
    };

    draw_row(line1_label, &line1_val, box_y + 26.0);
    draw_row(line2_label, &line2_val, box_y + 54.0);
    draw_row(line3_label, &line3_val, box_y + 82.0);
    draw_row(line4_label, &line4_val, box_y + 110.0);

    // Active Online Challenges Bar
    let chal_y = card_y + 322.0;
    let chal_w = box_w;
    let chal_x = box_x;
    let chal_h = 108.0;
    draw_rectangle(chal_x, chal_y, chal_w, chal_h, Color::new(0.04, 0.07, 0.13, 0.88));
    draw_rectangle_lines(chal_x, chal_y, chal_w, chal_h, 1.0, Color::new(0.25, 0.45, 0.7, 0.4));

    let chal_title = format!("Open Online Challenges ({} waiting in pool):", active_challenges.len());
    draw_text(&chal_title, chal_x + 16.0, chal_y + 22.0, 14.0, Color::new(0.3, 0.9, 1.0, 0.9));

    if active_challenges.is_empty() {
        let empty_msg = "No other players currently waiting. Your challenge is broadcast to all active players!";
        draw_text(empty_msg, chal_x + 16.0, chal_y + 58.0, 13.0, Color::new(0.7, 0.75, 0.85, 0.75));
    } else {
        for (idx, ch) in active_challenges.iter().take(2).enumerate() {
            let row_y = chal_y + 48.0 + idx as f32 * 26.0;
            let c_info = format!(
                "• {} {} (Rating: {:.0})  -  Preferred Court: {}",
                country_badge(&ch.country),
                ch.name,
                ch.rating,
                ch.arena_name
            );
            draw_text(&c_info, chal_x + 16.0, row_y, 14.0, Color::new(1.0, 0.9, 0.45, 0.95));
        }
    }

    // Cancel prompt & Key Shortcuts
    let hint_str = format!("Press [ ESC ] to Cancel Search | [ E ] Range: {} | [ A ] Cycle Court", elo_range.name());
    let c_dim = measure_text(&hint_str, None, 14, 1.0);
    let pulse_c = (time * 4.0).sin().abs() * 0.2 + 0.8;
    draw_text(
        &hint_str,
        (VIRTUAL_WIDTH - c_dim.width) * 0.5,
        card_y + card_h - 18.0,
        14.0,
        Color::new(0.2, 0.9, 1.0, pulse_c),
    );

    // Cross-Arena Challenge Modal Popup (if offer received)
    if let Some(offer) = cross_offer {
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.02, 0.03, 0.08, 0.82));

        let m_w = 580.0;
        let m_h = 280.0;
        let m_x = (VIRTUAL_WIDTH - m_w) * 0.5;
        let m_y = (VIRTUAL_HEIGHT - m_h) * 0.5;

        draw_rectangle(m_x, m_y, m_w, m_h, Color::new(0.08, 0.12, 0.22, 0.98));
        draw_rectangle_lines(m_x, m_y, m_w, m_h, 2.5, Color::new(1.0, 0.8, 0.2, 0.95));

        draw_line(m_x + 10.0, m_y + 1.0, m_x + m_w - 10.0, m_y + 1.0, 2.0, Color::new(1.0, 0.9, 0.3, 1.0));

        let m_title = "CROSS-ARENA CHALLENGE DETECTED!";
        let mt_dim = measure_text(m_title, None, 25, 1.0);
        draw_text(m_title, (VIRTUAL_WIDTH - mt_dim.width) * 0.5, m_y + 40.0, 25.0, Color::new(1.0, 0.8, 0.2, 1.0));

        let m_sub = "A suitable skill opponent was found, but is waiting in another court:";
        let ms_dim = measure_text(m_sub, None, 14, 1.0);
        draw_text(m_sub, (VIRTUAL_WIDTH - ms_dim.width) * 0.5, m_y + 66.0, 14.0, Color::new(0.85, 0.9, 0.98, 0.85));

        let box_offer_y = m_y + 84.0;
        let box_offer_w = m_w - 40.0;
        let box_offer_x = m_x + 20.0;
        draw_rectangle(box_offer_x, box_offer_y, box_offer_w, 88.0, Color::new(0.04, 0.07, 0.14, 0.9));
        draw_rectangle_lines(box_offer_x, box_offer_y, box_offer_w, 88.0, 1.0, Color::new(0.3, 0.5, 0.7, 0.4));

        let opp_str = format!("Opponent: {} {}  (Rating: {:.0})", country_badge(&offer.opponent_country), offer.opponent_name, offer.opponent_rating);
        draw_text(&opp_str, box_offer_x + 16.0, box_offer_y + 25.0, 16.0, WHITE);

        let opp_arena_str = format!("Opponent's Court:  {}", ArenaType::from_index(offer.opponent_arena).name());
        draw_text(&opp_arena_str, box_offer_x + 16.0, box_offer_y + 50.0, 15.0, Color::new(1.0, 0.85, 0.35, 1.0));

        let my_arena_str = format!("Your Court:        {}", ArenaType::from_index(offer.my_arena).name());
        draw_text(&my_arena_str, box_offer_x + 16.0, box_offer_y + 74.0, 15.0, Color::new(0.3, 0.9, 1.0, 1.0));

        // Action options
        let opt1 = "[ Y ]      Accept Opponent's Court & Play";
        let opt2 = "[ Space ]  Propose Coin-Flip Consensus Court (50/50)";
        let opt3 = "[ N / Esc ] Decline Offer & Continue Searching";

        draw_text(opt1, m_x + 40.0, m_y + 200.0, 16.0, Color::new(0.3, 1.0, 0.5, 1.0));
        draw_text(opt2, m_x + 40.0, m_y + 225.0, 15.0, Color::new(0.3, 0.85, 1.0, 0.95));
        draw_text(opt3, m_x + 40.0, m_y + 250.0, 15.0, Color::new(1.0, 0.45, 0.45, 0.9));
    }
}

pub fn render_victory_screen(
    _winner: PlayerSide,
    winner_name: &str,
    logic: &GameLogic,
    rating_update: Option<(f64, f64, &'static str)>,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.65));

    let has_rating = rating_update.is_some();
    let card_w = 540.0;
    let card_h = if has_rating { 390.0 } else { 330.0 };
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(1.0, 0.85, 0.2, 0.8));

    let winner_text = format!("{} WINS!", winner_name.to_uppercase());

    let pulse = (time * 4.0).sin().abs() * 0.15 + 0.85;
    let title_dim = measure_text(&winner_text, None, 38, 1.0);
    draw_text(
        &winner_text,
        (VIRTUAL_WIDTH - title_dim.width) * 0.5,
        card_y + 52.0,
        38.0,
        Color::new(1.0, 0.9 * pulse, 0.2, 1.0),
    );

    let score_str = format!("FINAL SCORE: {} - {}", logic.scores[0], logic.scores[1]);
    let score_dim = measure_text(&score_str, None, 25, 1.0);
    draw_text(
        &score_str,
        (VIRTUAL_WIDTH - score_dim.width) * 0.5,
        card_y + 94.0,
        25.0,
        WHITE,
    );

    let mut start_y = card_y + 140.0;

    // Rating update banner if applicable
    if let Some((old_r, new_r, rank)) = rating_update {
        let delta = new_r - old_r;
        let sign = if delta >= 0.0 { "+" } else { "" };
        let col = if delta >= 0.0 {
            Color::new(0.3, 1.0, 0.45, 1.0)
        } else {
            Color::new(1.0, 0.4, 0.4, 1.0)
        };

        let banner_w = card_w - 60.0;
        let banner_x = card_x + 30.0;
        draw_rectangle(banner_x, start_y - 22.0, banner_w, 36.0, Color::new(0.06, 0.12, 0.22, 0.9));
        draw_rectangle_lines(banner_x, start_y - 22.0, banner_w, 36.0, 1.0, Color::new(0.3, 0.7, 1.0, 0.6));

        let r_text = format!("GLICKO-2: {:.0} -> {:.0} ({}{:.0}) - {}", old_r, new_r, sign, delta, rank);
        let r_dim = measure_text(&r_text, None, 17, 1.0);
        draw_text(&r_text, (VIRTUAL_WIDTH - r_dim.width) * 0.5, start_y + 2.0, 17.0, col);

        start_y += 50.0;
    }

    // Stats breakdown
    let mins = (logic.stats.match_time_seconds / 60.0).floor() as u32;
    let secs = (logic.stats.match_time_seconds % 60.0).floor() as u32;
    let stat1 = format!("Match Duration:    {:02}:{:02}", mins, secs);
    let stat2 = format!("Total Rallies:     {}", logic.stats.total_rallies);
    let stat3 = format!("Longest Rally:     {} hits", logic.stats.max_rally_hits);

    draw_text(&stat1, card_x + 60.0, start_y, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));
    draw_text(&stat2, card_x + 60.0, start_y + 28.0, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));
    draw_text(&stat3, card_x + 60.0, start_y + 56.0, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));

    let prompt = "Press Space or Enter to Continue";
    let prompt_dim = measure_text(prompt, None, 18, 1.0);
    draw_text(
        prompt,
        (VIRTUAL_WIDTH - prompt_dim.width) * 0.5,
        card_y + card_h - 22.0,
        18.0,
        Color::new(0.2, 0.9, 1.0, pulse),
    );
}

pub fn render_exit_confirm(time: f32) {
    // Backdrop dark overlay
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.02, 0.03, 0.07, 0.8));

    let card_w = 480.0;
    let card_h = 240.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    // Glassmorphism card
    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.96));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(1.0, 0.45, 0.2, 0.85));

    // Top glow line
    draw_line(
        card_x + 10.0,
        card_y + 1.0,
        card_x + card_w - 10.0,
        card_y + 1.0,
        2.0,
        Color::new(1.0, 0.55, 0.2, 0.9),
    );

    // Title & prompt
    let title = "QUIT BLOBBY VOLLEY 3?";
    let title_dim = measure_text(title, None, 28, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 52.0, 28.0, Color::new(1.0, 0.65, 0.2, 1.0));

    let sub = "Are you sure you want to exit to desktop?";
    let sub_dim = measure_text(sub, None, 16, 1.0);
    draw_text(sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 85.0, 16.0, Color::new(0.85, 0.88, 0.95, 0.85));

    // Action buttons
    let pulse = (time * 4.0).sin().abs() * 0.15 + 0.85;

    // Button 1: Quit [ Enter ]
    let btn1_rect = Rect::new(card_x + 40.0, card_y + 125.0, 185.0, 44.0);
    draw_rectangle(btn1_rect.x, btn1_rect.y, btn1_rect.w, btn1_rect.h, Color::new(0.65, 0.15, 0.15, 0.85));
    draw_rectangle_lines(btn1_rect.x, btn1_rect.y, btn1_rect.w, btn1_rect.h, 1.5, Color::new(1.0, 0.35, 0.35, pulse));
    let t1 = "YES, QUIT [ Enter ]";
    let t1_dim = measure_text(t1, None, 15, 1.0);
    draw_text(t1, btn1_rect.x + (btn1_rect.w - t1_dim.width) * 0.5, btn1_rect.y + 27.0, 15.0, WHITE);

    // Button 2: Resume [ Esc ]
    let btn2_rect = Rect::new(card_x + 255.0, card_y + 125.0, 185.0, 44.0);
    draw_rectangle(btn2_rect.x, btn2_rect.y, btn2_rect.w, btn2_rect.h, Color::new(0.12, 0.28, 0.45, 0.85));
    draw_rectangle_lines(btn2_rect.x, btn2_rect.y, btn2_rect.w, btn2_rect.h, 1.5, Color::new(0.3, 0.85, 1.0, 0.85));
    let t2 = "CANCEL [ ESC ]";
    let t2_dim = measure_text(t2, None, 15, 1.0);
    draw_text(t2, btn2_rect.x + (btn2_rect.w - t2_dim.width) * 0.5, btn2_rect.y + 27.0, 15.0, Color::new(0.4, 0.95, 1.0, 1.0));

    // Helpful hint
    let hint = "Tip: Progress & Glicko ratings are always saved automatically";
    let hint_dim = measure_text(hint, None, 13, 1.0);
    draw_text(hint, (VIRTUAL_WIDTH - hint_dim.width) * 0.5, card_y + 205.0, 13.0, Color::new(0.6, 0.7, 0.8, 0.75));
}

pub fn render_leaderboard(
    period_idx: usize, // 0 = Week, 1 = Month, 2 = Year
    entries: &[LeaderboardEntry],
    my_name: &str,
    is_refreshing: bool,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.92));

    let card_w = 720.0;
    let card_h = 490.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    // Card frame
    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.96));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.8));

    // Header Glow Line
    draw_line(card_x + 10.0, card_y + 1.0, card_x + card_w - 10.0, card_y + 1.0, 2.0, Color::new(0.3, 0.9, 1.0, 0.95));

    // Title
    let title = "GLOBAL COMPETITIVE LEADERBOARDS";
    let title_dim = measure_text(title, None, 28, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 38.0, 28.0, Color::new(0.3, 0.9, 1.0, 1.0));

    // Subtitle
    let sub = "Live Glicko-2 Elo Rankings | Automatic Inactivity Drop-off System";
    let sub_dim = measure_text(sub, None, 13, 1.0);
    draw_text(sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 58.0, 13.0, Color::new(0.8, 0.85, 0.95, 0.8));

    // Period Tabs: [ 1. THIS WEEK ] [ 2. THIS MONTH ] [ 3. THIS YEAR ]
    let tab_w = 170.0;
    let tab_h = 28.0;
    let tab_spacing = 20.0;
    let start_tab_x = (VIRTUAL_WIDTH - (tab_w * 3.0 + tab_spacing * 2.0)) * 0.5;
    let tab_y = card_y + 74.0;

    let periods = [
        ("1. THIS WEEK (7d)", "Active within 7 days"),
        ("2. THIS MONTH (30d)", "Active within 30 days"),
        ("3. THIS YEAR (365d)", "Active within 1 year"),
    ];

    for (i, (tab_title, _)) in periods.iter().enumerate() {
        let is_sel = i == period_idx;
        let tx = start_tab_x + i as f32 * (tab_w + tab_spacing);

        let bg_col = if is_sel {
            Color::new(0.18, 0.55, 0.9, 0.75)
        } else {
            Color::new(0.08, 0.12, 0.22, 0.8)
        };
        let border_col = if is_sel {
            Color::new(0.3, 0.95, 1.0, 1.0)
        } else {
            Color::new(0.25, 0.35, 0.5, 0.5)
        };

        draw_rectangle(tx, tab_y, tab_w, tab_h, bg_col);
        draw_rectangle_lines(tx, tab_y, tab_w, tab_h, 1.5, border_col);

        let dim = measure_text(tab_title, None, 14, 1.0);
        let txt_col = if is_sel { WHITE } else { Color::new(0.7, 0.75, 0.85, 0.8) };
        draw_text(tab_title, tx + (tab_w - dim.width) * 0.5, tab_y + 19.0, 14.0, txt_col);
    }

    // Table Header
    let table_y = card_y + 120.0;
    draw_rectangle(card_x + 18.0, table_y - 12.0, card_w - 36.0, 26.0, Color::new(0.12, 0.16, 0.26, 0.9));
    draw_text("RANK", card_x + 30.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("PLAYER NAME", card_x + 95.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("DIVISION", card_x + 280.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("RATING (ELO)", card_x + 420.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("WIN RATE", card_x + 540.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));
    draw_text("LAST ACTIVE", card_x + 630.0, table_y + 6.0, 14.0, Color::new(0.4, 0.9, 1.0, 1.0));

    // Rows
    let row_h = 31.0;
    let list_start_y = table_y + 24.0;

    if is_refreshing {
        let msg = "Refreshing global rankings...";
        let dim = measure_text(msg, None, 18, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - dim.width) * 0.5, list_start_y + 100.0, 18.0, Color::new(0.3, 0.9, 1.0, 1.0));
    } else if entries.is_empty() {
        let msg = "No active players in this period. Press R to refresh.";
        let dim = measure_text(msg, None, 16, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - dim.width) * 0.5, list_start_y + 100.0, 16.0, Color::new(0.7, 0.75, 0.85, 0.8));
    } else {
        for (i, entry) in entries.iter().take(9).enumerate() {
            let ry = list_start_y + i as f32 * row_h;
            let is_me = entry.name.contains(my_name) || entry.name.contains("(You)");

            if is_me {
                let pulse = (time * 4.0).sin().abs() * 0.15 + 0.85;
                draw_rectangle(card_x + 18.0, ry - 7.0, card_w - 36.0, 27.0, Color::new(0.15, 0.45, 0.85, 0.4 * pulse));
                draw_rectangle_lines(card_x + 18.0, ry - 7.0, card_w - 36.0, 27.0, 1.5, Color::new(0.2, 0.9, 1.0, 0.9));
            } else if i % 2 == 1 {
                draw_rectangle(card_x + 18.0, ry - 7.0, card_w - 36.0, 27.0, Color::new(0.05, 0.08, 0.14, 0.5));
            }

            // Rank Medal / Number
            let rank_str = format!("#{}", entry.rank);
            let rank_col = match entry.rank {
                1 => Color::new(1.0, 0.85, 0.2, 1.0), // Gold
                2 => Color::new(0.85, 0.9, 0.98, 1.0), // Silver
                3 => Color::new(0.88, 0.55, 0.25, 1.0), // Bronze
                _ => Color::new(0.7, 0.75, 0.85, 0.85),
            };
            draw_text(&rank_str, card_x + 30.0, ry + 12.0, 15.0, rank_col);

            // Player Name
            let name_col = if is_me { Color::new(1.0, 0.95, 0.35, 1.0) } else { WHITE };
            draw_text(&entry.name, card_x + 95.0, ry + 12.0, 15.0, name_col);

            // Division Title
            let tier_col = match entry.rank_title.as_str() {
                "Grandmaster" => Color::new(1.0, 0.3, 0.3, 1.0),
                "Diamond" => Color::new(0.4, 0.9, 1.0, 1.0),
                "Platinum I" | "Platinum II" => Color::new(0.2, 1.0, 0.7, 1.0),
                "Gold I" | "Gold II" => Color::new(1.0, 0.85, 0.25, 1.0),
                "Silver I" | "Silver II" => Color::new(0.85, 0.88, 0.95, 1.0),
                _ => Color::new(0.8, 0.55, 0.3, 1.0),
            };
            draw_text(&entry.rank_title, card_x + 280.0, ry + 12.0, 14.0, tier_col);

            // Rating
            let r_str = format!("{:.0}", entry.rating);
            draw_text(&r_str, card_x + 420.0, ry + 12.0, 15.0, Color::new(1.0, 0.95, 0.45, 1.0));

            // Win Rate
            let wr_str = format!("{}% ({}W)", entry.win_rate, entry.matches_won);
            draw_text(&wr_str, card_x + 540.0, ry + 12.0, 14.0, Color::new(0.75, 0.85, 0.95, 0.9));

            // Last Active
            let active_str = if entry.last_active_days == 0 {
                "Today".to_string()
            } else if entry.last_active_days == 1 {
                "1d ago".to_string()
            } else {
                format!("{}d ago", entry.last_active_days)
            };
            draw_text(&active_str, card_x + 630.0, ry + 12.0, 14.0, Color::new(0.65, 0.75, 0.85, 0.75));
        }
    }

    // Inactivity notice banner
    let notice_y = card_y + card_h - 60.0;
    draw_rectangle(card_x + 25.0, notice_y, card_w - 50.0, 24.0, Color::new(0.05, 0.08, 0.15, 0.8));
    draw_rectangle_lines(card_x + 25.0, notice_y, card_w - 50.0, 24.0, 1.0, Color::new(0.2, 0.5, 0.8, 0.4));
    let notice = "Inactivity Rules: Players drop off after 7 days (Weekly), 30 days (Monthly), 90 days (All-Time)";
    let n_dim = measure_text(notice, None, 13, 1.0);
    draw_text(notice, (VIRTUAL_WIDTH - n_dim.width) * 0.5, notice_y + 16.0, 13.0, Color::new(0.75, 0.85, 0.95, 0.85));

    // Footer prompt
    let footer = "[ Left / Right / 1-3 ] Switch Period | [ R ] Refresh | [ ESC ] Return to Main Menu";
    let f_dim = measure_text(footer, None, 14, 1.0);
    draw_text(footer, (VIRTUAL_WIDTH - f_dim.width) * 0.5, card_y + card_h - 16.0, 14.0, Color::new(0.3, 0.9, 1.0, 0.9));
}
