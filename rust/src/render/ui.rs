// Blobby Volley 3 - 2026 Remastered Glassmorphism HUD, Menus, Online Lobby & Chat UI
use macroquad::prelude::*;
use crate::physics::PlayerSide;
use crate::rules::{GameLogic, RuleMode};
use crate::render::blob::BlobTheme;
use crate::ai::AiDifficulty;
use crate::render::arenas::ArenaType;
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    SinglePlayer,
    TwoPlayerLocal,
    BotVsBot,
    HostOnline,
    JoinOnline,
}

impl GameMode {
    pub fn name(self) -> &'static str {
        match self {
            GameMode::SinglePlayer => "1P vs CPU",
            GameMode::TwoPlayerLocal => "2 Players (Local)",
            GameMode::BotVsBot => "CPU vs CPU",
            GameMode::HostOnline => "Host Online Match",
            GameMode::JoinOnline => "Join Online Match",
        }
    }

    pub fn next(self) -> Self {
        match self {
            GameMode::SinglePlayer => GameMode::TwoPlayerLocal,
            GameMode::TwoPlayerLocal => GameMode::BotVsBot,
            GameMode::BotVsBot => GameMode::HostOnline,
            GameMode::HostOnline => GameMode::JoinOnline,
            GameMode::JoinOnline => GameMode::SinglePlayer,
        }
    }
}

pub fn render_hud(
    logic: &GameLogic,
    left_name: &str,
    right_name: &str,
    left_theme: BlobTheme,
    right_theme: BlobTheme,
    time: f32,
) {
    let board_w = 460.0;
    let board_h = 74.0;
    let board_x = (VIRTUAL_WIDTH - board_w) * 0.5;
    let board_y = 16.0;

    // Glassmorphism card backdrop
    draw_rectangle(board_x, board_y, board_w, board_h, Color::new(0.08, 0.1, 0.16, 0.82));
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

    // Subtitle info: First to N points & Rule Mode
    let mode_str = match logic.rule_mode {
        RuleMode::Modern => "RALLY POINTS",
        RuleMode::Classic => "CLASSIC SIDE-OUT",
    };
    let sub_str = format!("FIRST TO {}  •  {}", logic.score_to_win, mode_str);
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
    difficulty: AiDifficulty,
    rule_mode: RuleMode,
    score_to_win: i32,
    arena: ArenaType,
    left_theme: BlobTheme,
    right_theme: BlobTheme,
    selected_item: usize,
    time: f32,
) {
    // Backdrop dimming
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.7));

    // Glowing 2026 Title
    let title = "BLOBBY VOLLEY 3";
    let title_dim = measure_text(title, None, 56, 1.0);
    let tx = (VIRTUAL_WIDTH - title_dim.width) * 0.5;
    let ty = 120.0;

    // Title shadow and glow
    draw_text(title, tx + 2.0, ty + 3.0, 56.0, Color::new(0.0, 0.0, 0.0, 0.5));
    draw_text(title, tx, ty, 56.0, Color::new(0.15, 0.85, 1.0, 1.0));

    let subtitle = "REMASTERED • 2026 EDITION";
    let sub_dim = measure_text(subtitle, None, 18, 1.0);
    draw_text(
        subtitle,
        (VIRTUAL_WIDTH - sub_dim.width) * 0.5,
        ty + 28.0,
        18.0,
        Color::new(1.0, 0.85, 0.3, 0.9),
    );

    // Menu Card
    let card_w = 560.0;
    let card_h = 360.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = 175.0;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.88));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.5, Color::new(0.3, 0.5, 0.85, 0.5));

    // Menu Items
    let items = [
        format!("START MATCH  [ Press Space / Enter ]"),
        format!("Game Mode:       <  {}  >", game_mode.name()),
        format!("AI Difficulty:   <  {}  >", difficulty.name()),
        format!("Rules:           <  {}  >", match rule_mode {
            RuleMode::Modern => "Modern (Rally Points)",
            RuleMode::Classic => "Classic (Side-Out)",
        }),
        format!("Points to Win:   <  {}  >", score_to_win),
        format!("Arena:           <  {}  >", arena.name()),
        format!("P1 Color:        <  {}  >", left_theme.name()),
        format!("P2 Color:        <  {}  >", right_theme.name()),
    ];

    let start_y = card_y + 42.0;
    let row_h = 38.0;

    for (i, text) in items.iter().enumerate() {
        let is_selected = i == selected_item;
        let y = start_y + i as f32 * row_h;

        if is_selected {
            let pulse = (time * 5.0).sin().abs() * 0.2 + 0.8;
            draw_rectangle(
                card_x + 16.0,
                y - 24.0,
                card_w - 32.0,
                32.0,
                Color::new(0.15, 0.45, 0.85, 0.4 * pulse),
            );
            draw_rectangle_lines(
                card_x + 16.0,
                y - 24.0,
                card_w - 32.0,
                32.0,
                1.5,
                Color::new(0.2, 0.85, 1.0, 0.85),
            );
        }

        let col = if i == 0 {
            if is_selected {
                Color::new(1.0, 0.95, 0.3, 1.0)
            } else {
                Color::new(0.3, 0.9, 1.0, 1.0)
            }
        } else if is_selected {
            WHITE
        } else {
            Color::new(0.8, 0.85, 0.92, 0.85)
        };

        draw_text(text, card_x + 36.0, y - 2.0, 20.0, col);
    }

    // Footer Controls Guide
    let guide = "Controls: P1 [W/A/D or Gamepad] | P2 [Arrow Keys] | Online Chat [T] | Options [Left/Right]";
    let guide_dim = measure_text(guide, None, 14, 1.0);
    draw_text(
        guide,
        (VIRTUAL_WIDTH - guide_dim.width) * 0.5,
        VIRTUAL_HEIGHT - 24.0,
        14.0,
        Color::new(0.7, 0.75, 0.85, 0.75),
    );
}

pub fn render_host_lobby(port: u16, local_ip: &str, is_connected: bool, time: f32) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.85));

    let card_w = 520.0;
    let card_h = 300.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.8, 1.0, 0.7));

    let title = "HOSTING ONLINE MATCH";
    let title_dim = measure_text(title, None, 32, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 48.0, 32.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let info_ip = format!("Your Local LAN IP:  {}:{}", local_ip, port);
    let info_port = format!("Blobby Volley Port: {}", port);
    draw_text(&info_ip, card_x + 40.0, card_y + 110.0, 20.0, WHITE);
    draw_text(&info_port, card_x + 40.0, card_y + 145.0, 20.0, Color::new(0.8, 0.85, 0.95, 0.8));

    let pulse = (time * 4.0).sin().abs() * 0.3 + 0.7;
    if is_connected {
        let msg = "OPPONENT CONNECTED! STARTING...";
        let msg_dim = measure_text(msg, None, 22, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, card_y + 210.0, 22.0, Color::new(0.2, 1.0, 0.4, 1.0));
    } else {
        let msg = "Waiting for incoming connection...";
        let msg_dim = measure_text(msg, None, 20, 1.0);
        draw_text(msg, (VIRTUAL_WIDTH - msg_dim.width) * 0.5, card_y + 210.0, 20.0, Color::new(1.0, 0.85, 0.2, pulse));
    }

    let cancel = "Press Escape to Cancel and Return to Menu";
    let cancel_dim = measure_text(cancel, None, 16, 1.0);
    draw_text(cancel, (VIRTUAL_WIDTH - cancel_dim.width) * 0.5, card_y + card_h - 25.0, 16.0, Color::new(0.7, 0.75, 0.85, 0.7));
}

pub fn render_join_screen(ip_input: &str, is_connecting: bool, error: Option<&str>, time: f32) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.04, 0.05, 0.1, 0.85));

    let card_w = 520.0;
    let card_h = 320.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.2, 0.8, 1.0, 0.7));

    let title = "JOIN ONLINE MATCH";
    let title_dim = measure_text(title, None, 32, 1.0);
    draw_text(title, (VIRTUAL_WIDTH - title_dim.width) * 0.5, card_y + 48.0, 32.0, Color::new(0.3, 0.9, 1.0, 1.0));

    let label = "Enter Server Address (IP:Port):";
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

pub fn render_victory_screen(winner: PlayerSide, logic: &GameLogic, time: f32) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.65));

    let card_w = 480.0;
    let card_h = 320.0;
    let card_x = (VIRTUAL_WIDTH - card_w) * 0.5;
    let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5;

    draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.1, 0.18, 0.92));
    draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(1.0, 0.85, 0.2, 0.8));

    let winner_text = match winner {
        PlayerSide::Left => "PLAYER 1 WINS!",
        PlayerSide::Right => "PLAYER 2 WINS!",
    };

    let pulse = (time * 4.0).sin().abs() * 0.15 + 0.85;
    let title_dim = measure_text(winner_text, None, 44, 1.0);
    draw_text(
        winner_text,
        (VIRTUAL_WIDTH - title_dim.width) * 0.5,
        card_y + 58.0,
        44.0,
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
