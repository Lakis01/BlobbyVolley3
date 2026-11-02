// Blobby Volley 3 - 2026 Remastered Glassmorphism HUD, Menus, Profile Setup, Online Lobby & Chat UI
use macroquad::prelude::*;
use crate::physics::PlayerSide;
use crate::rules::{GameLogic, RuleMode};
use crate::render::blob::BlobTheme;
use crate::ai::AiDifficulty;
use crate::render::arenas::ArenaType;
use crate::constants::*;
use crate::net::ServerEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    SinglePlayer,
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
            GameMode::TwoPlayerLocal => "2 Players (Local)",
            GameMode::BotVsBot => "CPU vs CPU",
            GameMode::HostOnline => "Host Online Match",
            GameMode::JoinOnline => "Direct Connect (IP)",
            GameMode::ServerBrowser => "Public Server Browser",
        }
    }

    pub fn next(self) -> Self {
        match self {
            GameMode::SinglePlayer => GameMode::TwoPlayerLocal,
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
            GameMode::TwoPlayerLocal => GameMode::SinglePlayer,
            GameMode::BotVsBot => GameMode::TwoPlayerLocal,
            GameMode::HostOnline => GameMode::BotVsBot,
            GameMode::JoinOnline => GameMode::HostOnline,
            GameMode::ServerBrowser => GameMode::JoinOnline,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MenuRowLayout {
    pub index: usize,
    pub row_rect: Rect,
    pub left_arrow_rect: Option<Rect>,
    pub right_arrow_rect: Option<Rect>,
}

pub fn get_main_menu_layout() -> Vec<MenuRowLayout> {
    let card_w = 660.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = 120.0;
    let start_y = card_y + 36.0;
    let row_h = 37.0;

    let mut rows = Vec::new();
    for i in 0..10 {
        let y = start_y + i as f32 * row_h;
        let row_rect = Rect::new(card_x + 14.0, y - 18.0, card_w - 28.0, 32.0);

        if i == 0 {
            // Action button (START MATCH) - no arrows
            rows.push(MenuRowLayout {
                index: i,
                row_rect,
                left_arrow_rect: None,
                right_arrow_rect: None,
            });
        } else {
            // Settings row with distinct left and right arrow buttons
            let left_btn = Rect::new(card_x + card_w - 275.0, y - 16.0, 26.0, 26.0);
            let right_btn = Rect::new(card_x + card_w - 40.0, y - 16.0, 26.0, 26.0);
            rows.push(MenuRowLayout {
                index: i,
                row_rect,
                left_arrow_rect: Some(left_btn),
                right_arrow_rect: Some(right_btn),
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
    time: f32,
) {
    let board_w = 480.0;
    let board_h = 74.0;
    let board_x = (VIRTUAL_WIDTH - board_w) * 0.5;
    let board_y = 16.0;

    // Glassmorphism card backdrop
    draw_rectangle(board_x, board_y, board_w, board_h, Color::new(0.08, 0.1, 0.16, 0.85));
    draw_rectangle_lines(board_x, board_y, board_w, board_h, 1.5, Color::new(0.4, 0.6, 0.9, 0.45));

    // Glow accent line on top of card
    draw_line(
        board_x + 10.0,
        board_y + 1.0,
        board_x + board_w - 10.0,
        board_y + 1.0,
        2.0,
        Color::new(0.2, 0.85, 1.0, 0.75),
    );

    // Left player info
    let (l_core, _, _) = left_theme.colors();
    draw_circle(board_x + 28.0, board_y + 37.0, 12.0, l_core);
    draw_text(left_name, board_x + 48.0, board_y + 44.0, 22.0, WHITE);

    // Left serving indicator
    if logic.serving_player == PlayerSide::Left {
        let pulse = (time * 6.0).sin().abs() * 0.4 + 0.6;
        draw_circle(board_x + 28.0, board_y + 37.0, 16.0, Color::new(1.0, 0.85, 0.1, pulse * 0.7));
    }

    // Right player info
    let (r_core, _, _) = right_theme.colors();
    draw_circle(board_x + board_w - 28.0, board_y + 37.0, 12.0, r_core);
    let r_text_w = measure_text(right_name, None, 22, 1.0).width;
    draw_text(right_name, board_x + board_w - 48.0 - r_text_w, board_y + 44.0, 22.0, WHITE);

    // Right serving indicator
    if logic.serving_player == PlayerSide::Right {
        let pulse = (time * 6.0).sin().abs() * 0.4 + 0.6;
        draw_circle(board_x + board_w - 28.0, board_y + 37.0, 16.0, Color::new(1.0, 0.85, 0.1, pulse * 0.7));
    }

    // Score display in center
    let score_str = format!("{}  :  {}", logic.scores[0], logic.scores[1]);
    let score_dim = measure_text(&score_str, None, 38, 1.0);
    draw_text(
        &score_str,
        (VIRTUAL_WIDTH - score_dim.width) * 0.5,
        board_y + 48.0,
        38.0,
        Color::new(1.0, 0.95, 0.4, 1.0),
    );

    // Subtitle info: First to N points & Rule Mode (using clean ASCII hyphen)
    let mode_str = match logic.rule_mode {
        RuleMode::Modern => "RALLY POINTS",
        RuleMode::Classic => "CLASSIC SIDE-OUT",
    };
    let sub_str = format!("FIRST TO {}  -  {}", logic.score_to_win, mode_str);
    let sub_dim = measure_text(&sub_str, None, 13, 1.0);
    draw_text(
        &sub_str,
        (VIRTUAL_WIDTH - sub_dim.width) * 0.5,
        board_y + 66.0,
        13.0,
        Color::new(0.8, 0.85, 0.95, 0.75),
    );

    // Rally counter pill if rally > 1
    if logic.stats.current_rally_hits > 1 {
        let rally_str = format!("RALLY: {}", logic.stats.current_rally_hits);
        let rally_dim = measure_text(&rally_str, None, 14, 1.0);
        let rw = rally_dim.width + 18.0;
        let rx = (VIRTUAL_WIDTH - rw) * 0.5;
        let ry = board_y + board_h + 8.0;

        draw_rectangle(rx, ry, rw, 22.0, Color::new(0.1, 0.15, 0.25, 0.85));
        draw_rectangle_lines(rx, ry, rw, 22.0, 1.0, Color::new(0.2, 0.8, 1.0, 0.6));
        draw_text(&rally_str, rx + 9.0, ry + 16.0, 14.0, Color::new(0.4, 0.95, 1.0, 1.0));
    }
}

pub fn render_main_menu(
    game_mode: GameMode,
    player1_name: &str,
    difficulty: AiDifficulty,
    rule_mode: RuleMode,
    score_to_win: i32,
    arena: ArenaType,
    left_theme: BlobTheme,
    right_theme: BlobTheme,
    fullscreen: bool,
    selected_item: usize,
    virt_mouse: Option<Vec2>,
    time: f32,
) {
    // Backdrop dimming
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.72));

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

    // Menu Card
    let card_w = 660.0;
    let card_h = 425.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = 120.0;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.5, Color::new(0.3, 0.5, 0.85, 0.5));

    let layouts = get_main_menu_layout();

    // Data for each row: (Label, Value)
    let rows_data: [(Option<&'static str>, String); 10] = [
        (None, "START MATCH  [ Press Space / Enter ]".to_string()),
        (Some("Game Mode:"), game_mode.name().to_string()),
        (Some("Player Profile:"), player1_name.to_string()),
        (Some("AI Difficulty:"), difficulty.name().to_string()),
        (Some("Rules:"), rule_mode.name().to_string()),
        (Some("Points to Win:"), format!("{}", score_to_win)),
        (Some("Arena:"), arena.name().to_string()),
        (Some("P1 Color:"), left_theme.name().to_string()),
        (Some("P2 Color:"), right_theme.name().to_string()),
        (Some("Fullscreen:"), if fullscreen { "ENABLED".to_string() } else { "DISABLED".to_string() }),
    ];

    for layout in &layouts {
        let i = layout.index;
        let is_selected = i == selected_item;
        let (label_opt, ref val) = rows_data[i];

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

        let text_y = layout.row_rect.y + 22.0;

        if let Some(label) = label_opt {
            // Setting Row: Label on left
            let label_col = if is_selected { WHITE } else { Color::new(0.8, 0.85, 0.92, 0.85) };
            draw_text(label, card_x + 30.0, text_y, 19.0, label_col);

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

            // Value text centered between the two arrow buttons
            if let (Some(l_rect), Some(r_rect)) = (layout.left_arrow_rect, layout.right_arrow_rect) {
                let val_dim = measure_text(val, None, 18, 1.0);
                let center_x = (l_rect.x + l_rect.w + r_rect.x) * 0.5;
                let val_x = center_x - val_dim.width * 0.5;
                let val_col = if is_selected {
                    Color::new(1.0, 0.95, 0.4, 1.0)
                } else {
                    Color::new(0.85, 0.9, 1.0, 0.9)
                };
                draw_text(val, val_x, text_y, 18.0, val_col);
            }
        } else {
            // Action button (START MATCH)
            let col = if is_selected {
                Color::new(1.0, 0.95, 0.3, 1.0)
            } else {
                Color::new(0.3, 0.9, 1.0, 1.0)
            };
            let dim = measure_text(val, None, 21, 1.0);
            let tx = (VIRTUAL_WIDTH - dim.width) * 0.5;
            draw_text(val, tx, text_y, 21.0, col);
        }
    }

    // Footer Controls Guide
    let guide = "Controls: [Left/Right] or Click [< / >] to cycle | [Enter/Click Row] to Confirm & Start | [F11] Fullscreen";
    let guide_dim = measure_text(guide, None, 13, 1.0);
    draw_text(
        guide,
        (VIRTUAL_WIDTH - guide_dim.width) * 0.5,
        VIRTUAL_HEIGHT - 18.0,
        13.0,
        Color::new(0.7, 0.75, 0.85, 0.8),
    );
}

pub fn render_player_setup(
    p1_name: &str,
    p2_name: &str,
    is_two_player: bool,
    active_field: usize,
    time: f32,
) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.85));

    let card_w = 560.0;
    let card_h = if is_two_player { 340.0 } else { 290.0 };
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.95));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.85, 1.0, 0.8));

    let title = "PLAYER PROFILE & IDENTITY";
    let title_dim = measure_text(title, None, 30, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 44.0, 30.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let sub = "Display names are saved across sessions for local and online play";
    let sub_dim = measure_text(sub, None, 14, 1.0);
    draw_text(sub, (VIRTUAL_WIDTH - sub_dim.width) * 0.5, card_y + 70.0, 14.0, Color::new(0.75, 0.8, 0.9, 0.8));

    let cursor = if (time * 3.5) as i32 % 2 == 0 { "_" } else { "" };

    // Field 0: Player 1 Name
    let f0_y = card_y + 115.0;
    draw_text("Player 1 Name:", card_x + 40.0, f0_y - 8.0, 18.0, WHITE);
    let box0_rect = Rect::new(card_x + 40.0, f0_y, card_w - 80.0, 38.0);
    draw_rectangle(box0_rect.x, box0_rect.y, box0_rect.w, box0_rect.h, Color::new(0.05, 0.08, 0.14, 0.95));
    let f0_border = if active_field == 0 { Color::new(0.2, 0.9, 1.0, 1.0) } else { Color::new(0.3, 0.45, 0.65, 0.6) };
    draw_rectangle_lines(box0_rect.x, box0_rect.y, box0_rect.w, box0_rect.h, 1.5, f0_border);
    let f0_text = if active_field == 0 { format!("{}{}", p1_name, cursor) } else { p1_name.to_string() };
    draw_text(&f0_text, box0_rect.x + 12.0, box0_rect.y + 26.0, 20.0, Color::new(1.0, 0.95, 0.35, 1.0));

    // Field 1: Player 2 Name (only if 2-player mode)
    if is_two_player {
        let f1_y = card_y + 190.0;
        draw_text("Player 2 Name:", card_x + 40.0, f1_y - 8.0, 18.0, WHITE);
        let box1_rect = Rect::new(card_x + 40.0, f1_y, card_w - 80.0, 38.0);
        draw_rectangle(box1_rect.x, box1_rect.y, box1_rect.w, box1_rect.h, Color::new(0.05, 0.08, 0.14, 0.95));
        let f1_border = if active_field == 1 { Color::new(0.2, 0.9, 1.0, 1.0) } else { Color::new(0.3, 0.45, 0.65, 0.6) };
        draw_rectangle_lines(box1_rect.x, box1_rect.y, box1_rect.w, box1_rect.h, 1.5, f1_border);
        let f1_text = if active_field == 1 { format!("{}{}", p2_name, cursor) } else { p2_name.to_string() };
        draw_text(&f1_text, box1_rect.x + 12.0, box1_rect.y + 26.0, 20.0, Color::new(1.0, 0.95, 0.35, 1.0));
    }

    // Confirm Prompt
    let btn_y = card_y + card_h - 40.0;
    let hint = "Press Enter / Tab to Confirm & Proceed | Esc to Cancel";
    let hint_dim = measure_text(hint, None, 15, 1.0);
    draw_text(hint, (VIRTUAL_WIDTH - hint_dim.width) * 0.5, btn_y, 15.0, Color::new(0.3, 0.95, 1.0, 0.9));
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

pub fn render_victory_screen(_winner: PlayerSide, winner_name: &str, logic: &GameLogic, time: f32) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.65));

    let card_w = 520.0;
    let card_h = 330.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(1.0, 0.85, 0.2, 0.8));

    let winner_text = format!("{} WINS!", winner_name.to_uppercase());

    let pulse = (time * 4.0).sin().abs() * 0.15 + 0.85;
    let title_dim = measure_text(&winner_text, None, 40, 1.0);
    draw_text(
        &winner_text,
        (VIRTUAL_WIDTH - title_dim.width) * 0.5,
        card_y + 58.0,
        40.0,
        Color::new(1.0, 0.9 * pulse, 0.2, 1.0),
    );

    let score_str = format!("FINAL SCORE: {} - {}", logic.scores[0], logic.scores[1]);
    let score_dim = measure_text(&score_str, None, 26, 1.0);
    draw_text(
        &score_str,
        (VIRTUAL_WIDTH - score_dim.width) * 0.5,
        card_y + 105.0,
        26.0,
        WHITE,
    );

    // Stats breakdown
    let mins = (logic.stats.match_time_seconds / 60.0).floor() as u32;
    let secs = (logic.stats.match_time_seconds % 60.0).floor() as u32;
    let stat1 = format!("Match Duration:    {:02}:{:02}", mins, secs);
    let stat2 = format!("Total Rallies:     {}", logic.stats.total_rallies);
    let stat3 = format!("Longest Rally:     {} hits", logic.stats.max_rally_hits);

    let start_y = card_y + 155.0;
    draw_text(&stat1, card_x + 60.0, start_y, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));
    draw_text(&stat2, card_x + 60.0, start_y + 30.0, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));
    draw_text(&stat3, card_x + 60.0, start_y + 60.0, 18.0, Color::new(0.8, 0.85, 0.95, 0.9));

    let prompt = "Press Space or Enter to Continue";
    let prompt_dim = measure_text(prompt, None, 18, 1.0);
    draw_text(
        prompt,
        (VIRTUAL_WIDTH - prompt_dim.width) * 0.5,
        card_y + card_h - 24.0,
        18.0,
        Color::new(0.2, 0.9, 1.0, pulse),
    );
}
