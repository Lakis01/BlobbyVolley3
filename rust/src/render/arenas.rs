// Blobby Volley 3 - 2026 Remastered Arenas & Full-Screen Visual Overhaul
// Eliminates pillarboxing / black borders by dynamically expanding scenic horizons across all aspect ratios (16:10, 16:9, 21:9)
// while perfectly maintaining 100% authentic physics court dynamics (0..800x600).
use macroquad::prelude::*;
use glam::Vec2;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::constants::*;
use crate::physics::PlayerSide;
use crate::render::blob::BlobTheme;

#[derive(Debug, Clone, Copy, Default)]
pub struct CheerState {
    pub last_scorer: Option<PlayerSide>,
    pub cheer_timer: f32,
    pub match_winner: Option<PlayerSide>,
    pub left_theme: BlobTheme,
    pub right_theme: BlobTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaType {
    SunsetBeach,
    GrandStadium,
    CyberNeon,
    OlympicBeach,
    ClassicRemaster,
    AlpinePeaks,
    MagmaCaldera,
    OrbitalStation,
}

impl ArenaType {
    pub fn name(self) -> &'static str {
        match self {
            ArenaType::SunsetBeach => "Copacabana Sunset",
            ArenaType::GrandStadium => "Blobby Grand Arena (Stadium)",
            ArenaType::CyberNeon => "Neo Cyber Stadium",
            ArenaType::OlympicBeach => "Olympic Beach Stadium",
            ArenaType::ClassicRemaster => "Classic Beach HD",
            ArenaType::AlpinePeaks => "Alpine Peaks (Snow)",
            ArenaType::MagmaCaldera => "Magma Caldera (Volcano)",
            ArenaType::OrbitalStation => "Orbital Station (Space)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ArenaType::SunsetBeach => ArenaType::GrandStadium,
            ArenaType::GrandStadium => ArenaType::CyberNeon,
            ArenaType::CyberNeon => ArenaType::OlympicBeach,
            ArenaType::OlympicBeach => ArenaType::ClassicRemaster,
            ArenaType::ClassicRemaster => ArenaType::AlpinePeaks,
            ArenaType::AlpinePeaks => ArenaType::MagmaCaldera,
            ArenaType::MagmaCaldera => ArenaType::OrbitalStation,
            ArenaType::OrbitalStation => ArenaType::SunsetBeach,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ArenaType::SunsetBeach => ArenaType::OrbitalStation,
            ArenaType::GrandStadium => ArenaType::SunsetBeach,
            ArenaType::CyberNeon => ArenaType::GrandStadium,
            ArenaType::OlympicBeach => ArenaType::CyberNeon,
            ArenaType::ClassicRemaster => ArenaType::OlympicBeach,
            ArenaType::AlpinePeaks => ArenaType::ClassicRemaster,
            ArenaType::MagmaCaldera => ArenaType::AlpinePeaks,
            ArenaType::OrbitalStation => ArenaType::MagmaCaldera,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => ArenaType::SunsetBeach,
            1 => ArenaType::GrandStadium,
            2 => ArenaType::CyberNeon,
            3 => ArenaType::OlympicBeach,
            4 => ArenaType::ClassicRemaster,
            5 => ArenaType::AlpinePeaks,
            6 => ArenaType::MagmaCaldera,
            _ => ArenaType::OrbitalStation,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            ArenaType::SunsetBeach => 0,
            ArenaType::GrandStadium => 1,
            ArenaType::CyberNeon => 2,
            ArenaType::OlympicBeach => 3,
            ArenaType::ClassicRemaster => 4,
            ArenaType::AlpinePeaks => 5,
            ArenaType::MagmaCaldera => 6,
            ArenaType::OrbitalStation => 7,
        }
    }
}

pub fn render_arena(arena: ArenaType, time: f32, view_x0: f32, view_w: f32, ball_pos: Vec2, cheer: CheerState, court_width: f32) {
    // Calculate full dynamic screen bounds to seamlessly fill all displays (e.g. 16:10, 16:9, 21:9 ultrawide)
    let x_left = view_x0.min(-250.0);
    let x_right = (view_x0 + view_w).max(court_width + 250.0);
    let full_w = x_right - x_left;

    match arena {
        ArenaType::SunsetBeach => render_sunset_beach(time, x_left, x_right, full_w, court_width),
        ArenaType::GrandStadium => render_grand_stadium(time, x_left, x_right, full_w, ball_pos, cheer, court_width),
        ArenaType::CyberNeon => render_cyber_neon(time, x_left, x_right, full_w, ball_pos, cheer, court_width),
        ArenaType::OlympicBeach => render_olympic_beach(time, x_left, x_right, full_w, ball_pos, cheer, court_width),
        ArenaType::ClassicRemaster => render_classic_remaster(time, x_left, x_right, full_w, court_width),
        ArenaType::AlpinePeaks => render_alpine_peaks(time, x_left, x_right, full_w),
        ArenaType::MagmaCaldera => render_magma_caldera(time, x_left, x_right, full_w),
        ArenaType::OrbitalStation => render_orbital_station(time, x_left, x_right, full_w, court_width),
    }

    // Render tournament court boundary guidelines and floodlight towers
    render_court_boundaries(time, arena, court_width);

    // Render in-game perimeter advertising boards across active court and side extensions
    render_perimeter_boards(time, arena, x_left, x_right, court_width);
}

// =========================================================================
// 1. COPACABANA SUNSET BEACH (Modern Tropical Sunset with Parallax Vista)
// =========================================================================
fn render_sunset_beach(time: f32, x_left: f32, x_right: f32, full_w: f32, court_width: f32) {
    let sky_top = -500.0;
    let sky_bot = 330.0;
    let sky_h = sky_bot - sky_top;
    let bands = 24;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;

        let r = (0.08 + t * 0.90).min(1.0);
        let g = (0.03 + t * 0.50).min(1.0);
        let b = (0.35 - t * 0.25).max(0.0);
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Distant mountain ridge silhouettes (Sugarloaf & coastal peaks)
    let mountain_pts = [
        (x_left + 40.0, 260.0, 320.0),
        (court_width * 0.225, 240.0, 260.0),
        (court_width * 0.475, 270.0, 290.0),
        (court_width * 0.775, 250.0, 310.0),
        (x_right - 60.0, 260.0, 340.0),
    ];
    for (mx, my, mw) in mountain_pts {
        draw_triangle(
            Vec2::new(mx - mw * 0.5, 330.0),
            Vec2::new(mx, my),
            Vec2::new(mx + mw * 0.5, 330.0),
            Color::new(0.28, 0.14, 0.36, 0.95),
        );
    }

    // Setting sun with multi-layered atmospheric radial glow
    let sun_x = court_width * 0.5;
    let sun_y = 230.0;
    draw_circle(sun_x, sun_y, 110.0, Color::new(1.0, 0.75, 0.2, 0.08));
    draw_circle(sun_x, sun_y, 80.0, Color::new(1.0, 0.82, 0.3, 0.18));
    draw_circle(sun_x, sun_y, 55.0, Color::new(1.0, 0.92, 0.5, 0.45));
    draw_circle(sun_x, sun_y, 38.0, Color::new(1.0, 0.98, 0.85, 0.98));

    // Clouds drifting across full width
    let cloud_span = full_w + 200.0;
    render_stylized_cloud(x_left + ((time * 6.0 + 80.0) % cloud_span) - 100.0, 85.0, 1.25, 0.32);
    render_stylized_cloud(x_left + ((time * 4.5 + 460.0) % cloud_span) - 100.0, 125.0, 0.95, 0.26);
    render_stylized_cloud(x_left + ((time * 5.2 + 820.0) % cloud_span) - 100.0, 65.0, 0.75, 0.22);

    // Deep ocean extending full width
    let ocean_top = 310.0;
    let ocean_h = 100.0;
    draw_rectangle(x_left, ocean_top, full_w, ocean_h, Color::new(0.08, 0.22, 0.45, 1.0));

    // Sun reflection light trail shimmering on water
    let shimmer_w = 160.0;
    draw_rectangle(
        sun_x - shimmer_w * 0.5,
        ocean_top,
        shimmer_w,
        ocean_h,
        Color::new(1.0, 0.7, 0.25, 0.24),
    );

    // Dynamic wave ripples across full screen
    for wave in 0..4 {
        let wave_y = ocean_top + 14.0 * wave as f32 + 10.0;
        let speed = (wave as f32 + 1.0) * 1.5;
        let mut prev_x = x_left;
        let mut prev_y = wave_y;

        let mut x = x_left;
        while x <= x_right {
            let offset_y = ((x * 0.03 + time * speed).sin()) * 3.5;
            let curr_y = wave_y + offset_y;
            draw_line(prev_x, prev_y, x, curr_y, 2.0, Color::new(0.65, 0.88, 0.95, 0.4));
            prev_x = x;
            prev_y = curr_y;
            x += 16.0;
        }
    }

    // Swaying palm trees placed along outer dunes
    let sway_left = (time * 1.2).sin() * 8.0;
    let sway_right = ((time * 1.4) + 1.2).sin() * 9.0;
    render_palm_tree(x_left + 80.0, GROUND_PLANE_HEIGHT_MAX, 1.2, sway_left, true);
    render_palm_tree(95.0, GROUND_PLANE_HEIGHT_MAX, 1.05, sway_left * 0.9, true);
    render_palm_tree(court_width - 95.0, GROUND_PLANE_HEIGHT_MAX, 1.1, sway_right * 0.9, false);
    render_palm_tree(x_right - 80.0, GROUND_PLANE_HEIGHT_MAX, 1.25, sway_right, false);

    // Sand dunes extending across entire display
    let sand_top = GROUND_PLANE_HEIGHT_MAX - 75.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top + 50.0;
    draw_rectangle(x_left, sand_top, full_w, sand_h, Color::new(0.92, 0.76, 0.48, 1.0));
    draw_rectangle(x_left, sand_top, full_w, 18.0, Color::new(0.82, 0.64, 0.38, 0.5));

    // Court sand highlight inside official play boundary (0..court_width)
    draw_rectangle(0.0, sand_top + 10.0, court_width, sand_h, Color::new(0.95, 0.80, 0.52, 0.35));
}

// =========================================================================
// 2. BLOBBY GRAND ARENA (Premier World Championship Stadium with Spectators)
// =========================================================================
fn render_grand_stadium(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState, court_width: f32) {
    // 1. Indoor Stadium Enclosed Ceiling Dome extending into upper sky
    let ceil_top = -500.0;
    let ceil_bot = 175.0;
    let ceil_h = ceil_bot - ceil_top;
    for i in 0..16 {
        let t = i as f32 / 16.0;
        let y = ceil_top + t * ceil_h;
        let h = ceil_h / 16.0 + 1.5;
        let r = 0.03 + t * 0.09;
        let g = 0.04 + t * 0.11;
        let b = 0.08 + t * 0.19;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Steel Lattice Roof Truss Arches extending upward
    let lower_arch_y = 65.0;
    for &arch_y in &[-350.0, -180.0, -20.0, lower_arch_y] {
        draw_line(x_left, arch_y, x_right, arch_y, 3.0, Color::new(0.25, 0.32, 0.45, 0.85));
        draw_line(x_left, arch_y + 18.0, x_right, arch_y + 18.0, 2.0, Color::new(0.18, 0.24, 0.35, 0.8));
        let mut tx = x_left + 20.0;
        while tx < x_right {
            draw_line(tx, arch_y, tx + 25.0, arch_y + 18.0, 1.5, Color::new(0.25, 0.32, 0.45, 0.7));
            draw_line(tx + 25.0, arch_y + 18.0, tx + 50.0, arch_y, 1.5, Color::new(0.25, 0.32, 0.45, 0.7));
            tx += 50.0;
        }
    }

    // Suspended Arena Sound System Speakers
    draw_rectangle(court_width * 0.225, lower_arch_y + 18.0, 32.0, 18.0, Color::new(0.08, 0.1, 0.15, 0.95));
    draw_rectangle(court_width * 0.7375, lower_arch_y + 18.0, 32.0, 18.0, Color::new(0.08, 0.1, 0.15, 0.95));

    // 2. Upper Balcony Tier Bleachers (y = 165, 192, 219)
    let upper_rows = [165.0, 192.0, 219.0];
    let upper_scales = [0.48, 0.53, 0.58];
    render_stadium_crowd(time, x_left, x_right, &upper_rows, &upper_scales, CrowdStyle::GrandColiseum, ball_pos, cheer, court_width);

    // Balcony Tier Concrete Fascia & LED Ribbon Board
    let fascia_y = 236.0;
    let fascia_h = 24.0;
    draw_rectangle(x_left, fascia_y, full_w, fascia_h, Color::new(0.1, 0.12, 0.18, 0.98));
    draw_line(x_left, fascia_y, x_right, fascia_y, 2.0, Color::new(0.3, 0.4, 0.55, 0.9));
    draw_line(x_left, fascia_y + fascia_h, x_right, fascia_y + fascia_h, 2.0, Color::new(0.3, 0.4, 0.55, 0.9));

    // Scrolling Ribbon Ticker
    let ribbon_text = " ★ BLOBBY GRAND ARENA ★ 2026 WORLD VOLLEY TOUR ★ SPIKE TO WIN ★ ";
    let mut rx = x_left + ((time * 30.0) % 350.0) - 350.0;
    while rx < x_right {
        draw_text(ribbon_text, rx, fascia_y + 16.0, 12.0, Color::new(0.25, 0.95, 1.0, 0.95));
        rx += 380.0;
    }

    // 4. Lower Main Grandstand Bleachers (y = 272, 302, 332, 362, 392)
    let lower_rows = [272.0, 302.0, 332.0, 362.0, 392.0];
    let lower_scales = [0.65, 0.72, 0.79, 0.86, 0.93];
    render_stadium_crowd(time, x_left, x_right, &lower_rows, &lower_scales, CrowdStyle::GrandColiseum, ball_pos, cheer, court_width);

    // 5. Hardwood Parquet Championship Court Floor
    let court_top = GROUND_PLANE_HEIGHT_MAX - 75.0; // 425.0
    let court_h = VIRTUAL_HEIGHT - court_top + 150.0;
    draw_rectangle(x_left, court_top, full_w, court_h, Color::new(0.84, 0.65, 0.42, 1.0));
    draw_rectangle(0.0, court_top, court_width, court_h, Color::new(0.89, 0.70, 0.46, 0.6));

    // Parquet horizontal wood plank grooves
    for p in 0..14 {
        let py = court_top + p as f32 * 18.0;
        let shade = if p % 2 == 0 { Color::new(0.78, 0.58, 0.36, 0.6) } else { Color::new(0.92, 0.74, 0.49, 0.3) };
        draw_line(x_left, py, x_right, py, 1.2, shade);
    }
}

// =========================================================================
// 3. NEO TOKYO CYBER STADIUM (Synthwave / Cyberpunk Full-Screen Grid with Cyber Crowd)
// =========================================================================
fn render_cyber_neon(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState, court_width: f32) {
    let sky_top = -500.0;
    let sky_bot = 240.0;
    let sky_h = sky_bot - sky_top;
    let bands = 20;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;
        let r = 0.02 + t * 0.38;
        let g = 0.01 + t * 0.05;
        let b = 0.08 + t * 0.50;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Giant Wireframe Retro Sun centered at net
    let sun_x = court_width * 0.5;
    let sun_y = 150.0;
    let sun_radius = 75.0;
    draw_circle(sun_x, sun_y, sun_radius + 20.0, Color::new(1.0, 0.15, 0.65, 0.12));
    draw_circle(sun_x, sun_y, sun_radius, Color::new(1.0, 0.25, 0.55, 0.95));

    // Sun horizontal scanlines
    for bar in 0..6 {
        let bar_y = sun_y + 10.0 + bar as f32 * 10.0;
        let bar_h = 3.0 + bar as f32 * 0.7;
        let dist_y = bar_y - sun_y;
        if dist_y < sun_radius {
            let half_w = (sun_radius * sun_radius - dist_y * dist_y).sqrt();
            draw_rectangle(sun_x - half_w, bar_y, half_w * 2.0, bar_h, Color::new(0.12, 0.04, 0.24, 1.0));
        }
    }

    // Extended cyberpunk skyline across entire horizon
    let mut bldgs = vec![
        (x_left + 30.0, 150.0, 65.0),
        (x_left + 110.0, 120.0, 75.0),
        (20.0, 160.0, 50.0),
        (65.0, 130.0, 65.0),
        (125.0, 180.0, 55.0),
        (175.0, 140.0, 70.0),
        (260.0, 170.0, 60.0),
        (550.0, 130.0, 65.0),
        (610.0, 170.0, 55.0),
        (660.0, 120.0, 75.0),
        (730.0, 165.0, 60.0),
        (x_right - 180.0, 130.0, 70.0),
        (x_right - 100.0, 160.0, 75.0),
    ];
    for (bx, by, bw) in bldgs.drain(..) {
        draw_rectangle(bx, by, bw, 240.0 - by, Color::new(0.08, 0.07, 0.18, 0.95));
        draw_rectangle_lines(bx, by, bw, 240.0 - by, 1.0, Color::new(0.2, 0.85, 1.0, 0.3));

        // Blinking red aviation warning lights on top of tallest spires
        if by < 145.0 {
            let blink = (time * 3.0 + bx).sin().abs() > 0.3;
            if blink {
                draw_circle(bx + bw * 0.5, by - 4.0, 2.5, Color::new(1.0, 0.2, 0.2, 1.0));
            }
        }
    }

    // Neon Cyber Stadium Bleachers packed with Cyber-Blob Spectators!
    let cyber_rows = [245.0, 282.0, 319.0, 356.0, 392.0];
    let cyber_scales = [0.63, 0.71, 0.79, 0.87, 0.94];
    render_stadium_crowd(time, x_left, x_right, &cyber_rows, &cyber_scales, CrowdStyle::Cyberpunk, ball_pos, cheer, court_width);

    // 3D Perspective Cyber Grid Floor
    let grid_top = GROUND_PLANE_HEIGHT_MAX - 75.0; // 425.0
    draw_rectangle(x_left, grid_top, full_w, VIRTUAL_HEIGHT - grid_top + 50.0, Color::new(0.05, 0.04, 0.12, 1.0));

    let horiz_lines = 10;
    for i in 1..=horiz_lines {
        let t = (i as f32 / horiz_lines as f32).powf(1.8);
        let y = grid_top + t * (VIRTUAL_HEIGHT - grid_top + 50.0);
        let alpha = 0.2 + t * 0.55;
        draw_line(x_left, y, x_right, y, 1.5, Color::new(0.15, 0.85, 1.0, alpha));
    }

    let v_rays = 24;
    let net_center_x = court_width * 0.5;
    for i in 0..=v_rays {
        let bottom_x = x_left + i as f32 * (full_w / v_rays as f32);
        let vanish_x = net_center_x + (bottom_x - net_center_x) * 0.2;
        draw_line(vanish_x, grid_top, bottom_x, VIRTUAL_HEIGHT + 50.0, 1.2, Color::new(1.0, 0.15, 0.7, 0.35));
    }
}

// =========================================================================
// 4. OLYMPIC BEACH STADIUM (Open-Air Seaside Tournament Grandstands)
// =========================================================================
fn render_olympic_beach(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState, court_width: f32) {
    // 1. Sunny Coastal Azure Sky extending into upper sky
    let sky_top = -500.0;
    let sky_bot = 240.0;
    let sky_h = sky_bot - sky_top;
    let bands = 16;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;
        let r = 0.12 + t * 0.43;
        let g = 0.40 + t * 0.50;
        let b = 0.95 + t * 0.05;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Golden Sun
    draw_circle(210.0, 75.0, 48.0, Color::new(1.0, 0.95, 0.75, 0.2));
    draw_circle(210.0, 75.0, 32.0, Color::new(1.0, 0.96, 0.82, 1.0));

    // Drifting clouds
    let cloud_span = full_w + 200.0;
    render_stylized_cloud(x_left + ((time * 3.0 + 80.0) % cloud_span) - 100.0, 60.0, 1.1, 0.85);
    render_stylized_cloud(x_left + ((time * 2.5 + 460.0) % cloud_span) - 100.0, 95.0, 1.3, 0.8);

    // Ocean Horizon & White Sailboats
    let ocean_y = 195.0;
    draw_rectangle(x_left, ocean_y, full_w, 45.0, Color::new(0.12, 0.52, 0.80, 0.95));
    // Gentle sea wave shimmer
    for wave in 0..2 {
        let wy = ocean_y + 8.0 + wave as f32 * 14.0;
        let mut wx = x_left;
        while wx <= x_right {
            let offset_y = (time * 2.5 + wx * 0.04).sin() * 2.5;
            draw_line(wx, wy + offset_y, wx + 18.0, wy + offset_y, 1.2, Color::new(1.0, 1.0, 1.0, 0.35));
            wx += 36.0;
        }
    }

    // Flanking Tropical Palms on the edges
    let sway_left = (time * 1.5).sin() * 6.0;
    let sway_right = (time * 1.5 + 1.2).sin() * 6.0;
    render_palm_tree(x_left + 75.0, GROUND_PLANE_HEIGHT_MAX, 1.15, sway_left, true);
    render_palm_tree(x_right - 75.0, GROUND_PLANE_HEIGHT_MAX, 1.2, sway_right, false);

    // 2. Tournament Sun-Canopy / Striped Awning above bleachers
    let canopy_y = 205.0;
    let canopy_h = 24.0;
    let stripe_w = 20.0;
    let mut cx = x_left;
    let mut stripe_idx = 0;
    while cx < x_right {
        let col = if stripe_idx % 2 == 0 {
            Color::new(0.15, 0.45, 0.85, 0.95) // Olympic Blue
        } else {
            Color::new(0.96, 0.96, 0.98, 0.95) // Crisp White
        };
        draw_rectangle(cx, canopy_y, stripe_w, canopy_h, col);
        cx += stripe_w;
        stripe_idx += 1;
    }
    draw_line(x_left, canopy_y, x_right, canopy_y, 2.0, Color::new(0.2, 0.3, 0.45, 0.9));
    draw_line(x_left, canopy_y + canopy_h, x_right, canopy_y + canopy_h, 2.0, Color::new(0.2, 0.3, 0.45, 0.9));

    // International Tournament Pennant Flags on canopy poles
    let mut flag_x = x_left + 35.0;
    let mut f_idx = 0;
    while flag_x < x_right {
        let pole_top = canopy_y - 18.0;
        draw_line(flag_x, canopy_y, flag_x, pole_top, 1.5, Color::new(0.6, 0.65, 0.75, 0.9));
        let flag_col = match f_idx % 5 {
            0 => Color::new(0.95, 0.25, 0.25, 1.0),
            1 => Color::new(1.0, 0.82, 0.2, 1.0),
            2 => Color::new(0.2, 0.85, 0.45, 1.0),
            3 => Color::new(0.18, 0.65, 1.0, 1.0),
            _ => Color::new(0.85, 0.35, 0.95, 1.0),
        };
        let flap = (time * 7.0 + flag_x * 0.1).sin() * 2.5;
        draw_triangle(
            Vec2::new(flag_x, pole_top),
            Vec2::new(flag_x + 14.0, pole_top + 4.0 + flap),
            Vec2::new(flag_x, pole_top + 9.0),
            flag_col,
        );
        flag_x += 80.0;
        f_idx += 1;
    }

    // 3. Bleacher Rows filled with Beach Blob Spectators
    let beach_rows = [248.0, 284.0, 320.0, 356.0, 392.0];
    let beach_scales = [0.64, 0.71, 0.78, 0.85, 0.92];
    render_stadium_crowd(time, x_left, x_right, &beach_rows, &beach_scales, CrowdStyle::OlympicBeach, ball_pos, cheer, court_width);

    // 4. Golden Sand Court
    let sand_top = GROUND_PLANE_HEIGHT_MAX - 75.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top + 50.0;
    draw_rectangle(x_left, sand_top, full_w, sand_h, Color::new(0.92, 0.77, 0.49, 1.0));
    draw_rectangle(x_left, sand_top, full_w, 16.0, Color::new(0.82, 0.65, 0.38, 0.5));
    draw_rectangle(0.0, sand_top + 8.0, court_width, sand_h, Color::new(0.96, 0.82, 0.54, 0.35));
}

// =========================================================================
// 3. CLASSIC BEACH HD (Nostalgic Sunny Sky & Crystal Ocean)
// =========================================================================
fn render_classic_remaster(time: f32, x_left: f32, x_right: f32, full_w: f32, court_width: f32) {
    let sky_top = -500.0;
    let sky_bot = 360.0;
    let sky_h = sky_bot - sky_top;
    let bands = 18;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;
        let r = 0.15 + t * 0.55;
        let g = 0.40 + t * 0.50;
        let b = 0.95 + t * 0.05;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Brilliant golden summer sun
    let sun_x = court_width * 0.85;
    draw_circle(sun_x, 85.0, 52.0, Color::new(1.0, 0.98, 0.8, 0.18));
    draw_circle(sun_x, 85.0, 34.0, Color::new(1.0, 0.98, 0.85, 1.0));

    // Fluffy procedural cartoon clouds
    let cloud_span = full_w + 200.0;
    render_stylized_cloud(x_left + ((time * 3.5 + 100.0) % cloud_span) - 100.0, 75.0, 1.15, 0.88);
    render_stylized_cloud(x_left + ((time * 2.8 + 480.0) % cloud_span) - 100.0, 110.0, 1.35, 0.82);
    render_stylized_cloud(x_left + ((time * 3.2 + 820.0) % cloud_span) - 100.0, 60.0, 0.9, 0.78);

    // Tropical ocean
    let ocean_y = 330.0;
    draw_rectangle(x_left, ocean_y, full_w, 90.0, Color::new(0.12, 0.55, 0.82, 1.0));

    for wave in 0..3 {
        let wy = ocean_y + wave as f32 * 18.0 + 8.0;
        let mut x = x_left;
        while x <= x_right {
            let dy = ((x * 0.04 + time * 2.0).sin()) * 2.5;
            draw_circle(x, wy + dy, 3.2, Color::new(0.9, 0.96, 1.0, 0.65));
            x += 24.0;
        }
    }

    // Pristine tropical sand court
    let sand_y = 420.0;
    draw_rectangle(x_left, sand_y, full_w, VIRTUAL_HEIGHT - sand_y + 50.0, Color::new(0.95, 0.84, 0.55, 1.0));
    draw_rectangle(x_left, sand_y, full_w, 14.0, Color::new(0.85, 0.72, 0.42, 0.45));
}

// =========================================================================
// 4. ALPINE PEAKS (Winter Glacier with Animated Aurora Borealis)
// =========================================================================
fn render_alpine_peaks(time: f32, x_left: f32, x_right: f32, full_w: f32) {
    let sky_top = -500.0;
    let sky_bot = 360.0;
    let sky_h = sky_bot - sky_top;
    let bands = 20;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;
        let r = 0.04 + t * 0.38;
        let g = 0.08 + t * 0.50;
        let b = 0.22 + t * 0.65;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Dynamic Aurora Borealis ribbons rippling across the Arctic sky
    for ribbon in 0..3 {
        let base_y = 70.0 + ribbon as f32 * 45.0;
        let speed = (ribbon as f32 + 1.0) * 0.7;
        let mut prev_x = x_left;
        let mut prev_y = base_y;

        let mut x = x_left;
        while x <= x_right {
            let offset_y = (x * 0.015 + time * speed).sin() * 18.0 + (x * 0.03 - time * 0.5).cos() * 8.0;
            let curr_y = base_y + offset_y;
            let alpha = 0.18 + ((x * 0.01 + time).sin().abs()) * 0.15;
            let col = if ribbon % 2 == 0 {
                Color::new(0.15, 0.95, 0.65, alpha)
            } else {
                Color::new(0.2, 0.75, 1.0, alpha)
            };
            draw_line(prev_x, prev_y, x, curr_y, 8.0, col);
            prev_x = x;
            prev_y = curr_y;
            x += 20.0;
        }
    }

    // Distant mountain ranges with snow caps across full width
    let mut peaks = vec![
        (x_left + 60.0, 160.0, 240.0),
        (x_left + 240.0, 130.0, 260.0),
        (80.0, 160.0, 180.0),
        (260.0, 130.0, 220.0),
        (450.0, 150.0, 200.0),
        (620.0, 120.0, 240.0),
        (760.0, 170.0, 190.0),
        (x_right - 180.0, 140.0, 240.0),
        (x_right - 50.0, 160.0, 220.0),
    ];

    for (px, py, pw) in peaks.drain(..) {
        draw_triangle(
            Vec2::new(px - pw * 0.5, 360.0),
            Vec2::new(px, py),
            Vec2::new(px + pw * 0.5, 360.0),
            Color::new(0.32, 0.42, 0.55, 1.0),
        );
        // Snow cap
        draw_triangle(
            Vec2::new(px - pw * 0.2, py + (360.0 - py) * 0.35),
            Vec2::new(px, py),
            Vec2::new(px + pw * 0.2, py + (360.0 - py) * 0.35),
            Color::new(0.95, 0.98, 1.0, 0.95),
        );
    }

    // Snowy pine trees along the foothills across full screen
    let mut tx = x_left + 15.0;
    let mut i = 0;
    while tx <= x_right {
        let ty = 340.0 + (i as f32 * 1.7).sin() * 12.0;
        draw_triangle(Vec2::new(tx - 14.0, ty + 30.0), Vec2::new(tx, ty), Vec2::new(tx + 14.0, ty + 30.0), Color::new(0.12, 0.32, 0.22, 1.0));
        draw_triangle(Vec2::new(tx - 10.0, ty + 16.0), Vec2::new(tx, ty - 6.0), Vec2::new(tx + 10.0, ty + 16.0), Color::new(0.16, 0.42, 0.28, 1.0));
        draw_triangle(Vec2::new(tx - 5.0, ty + 4.0), Vec2::new(tx, ty - 6.0), Vec2::new(tx + 5.0, ty + 4.0), Color::new(0.92, 0.96, 1.0, 0.9));
        tx += 50.0;
        i += 1;
    }

    // Frosted Snow Sand Court
    let snow_y = 415.0;
    draw_rectangle(x_left, snow_y, full_w, VIRTUAL_HEIGHT - snow_y + 50.0, Color::new(0.88, 0.94, 0.98, 1.0));
    draw_rectangle(x_left, snow_y, full_w, 15.0, Color::new(0.75, 0.85, 0.95, 0.5));

    // Falling procedural snowflakes across full screen
    for flake in 0..38 {
        let seed = flake as f32 * 31.7;
        let fx = x_left + ((seed * 17.3 + time * 24.0) % (full_w + 60.0)) - 30.0;
        let fy = (seed * 29.1 + time * 45.0) % 520.0;
        let sway = (time * 2.0 + seed).sin() * 6.0;
        let size = 1.8 + (seed % 2.5);
        draw_circle(fx + sway, fy, size, Color::new(1.0, 1.0, 1.0, 0.75));
    }
}

// =========================================================================
// 5. MAGMA CALDERA (Volcanic Magma Arena with Glowing Fissures)
// =========================================================================
fn render_magma_caldera(time: f32, x_left: f32, x_right: f32, full_w: f32) {
    let sky_top = -500.0;
    let sky_bot = 360.0;
    let sky_h = sky_bot - sky_top;
    let bands = 20;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = sky_top + t * sky_h;
        let h = sky_h / bands as f32 + 1.5;
        let r = 0.08 + t * 0.78;
        let g = 0.01 + t * 0.20;
        let b = 0.02 + t * 0.06;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Jagged basalt peaks & smoking caldera rim
    let mut spires = vec![
        (x_left + 80.0, 160.0, 150.0),
        (x_left + 220.0, 140.0, 160.0),
        (60.0, 180.0, 120.0),
        (200.0, 140.0, 160.0),
        (350.0, 190.0, 140.0),
        (480.0, 150.0, 170.0),
        (640.0, 170.0, 150.0),
        (750.0, 130.0, 160.0),
        (x_right - 180.0, 150.0, 170.0),
        (x_right - 40.0, 170.0, 140.0),
    ];

    for (sx, sy, sw) in spires.drain(..) {
        draw_triangle(
            Vec2::new(sx - sw * 0.5, 360.0),
            Vec2::new(sx, sy),
            Vec2::new(sx + sw * 0.5, 360.0),
            Color::new(0.14, 0.08, 0.09, 1.0),
        );
        // Magma fissure lines
        draw_line(sx, sy + 30.0, sx + 10.0, 360.0, 2.0, Color::new(1.0, 0.35, 0.05, 0.6));
    }

    // Molten lava pool under court across full screen
    let lava_y = 350.0;
    let pulse = (time * 3.5).sin().abs() * 0.2 + 0.8;
    draw_rectangle(x_left, lava_y, full_w, 70.0, Color::new(0.85 * pulse, 0.22 * pulse, 0.02, 1.0));

    // Lava crust islands
    let mut cx = x_left + 10.0;
    let mut c = 0;
    while cx <= x_right {
        let cy = lava_y + 12.0 + (c as f32 * 1.5).sin() * 8.0;
        draw_rectangle(cx, cy, 70.0, 16.0, Color::new(0.2, 0.1, 0.08, 0.95));
        draw_line(cx, cy, cx + 70.0, cy, 1.5, Color::new(1.0, 0.55, 0.1, 0.7));
        cx += 105.0;
        c += 1;
    }

    // Dark Basalt Tile Court
    let court_y = 420.0;
    draw_rectangle(x_left, court_y, full_w, VIRTUAL_HEIGHT - court_y + 50.0, Color::new(0.16, 0.14, 0.16, 1.0));
    draw_rectangle(x_left, court_y, full_w, 14.0, Color::new(0.28, 0.18, 0.12, 0.7));

    // Rising burning embers across full screen
    for ember in 0..30 {
        let seed = ember as f32 * 23.9;
        let ex = x_left + ((seed * 19.3 + time * 18.0) % (full_w + 40.0)) - 20.0;
        let ey = 500.0 - ((seed * 27.1 + time * 65.0) % 380.0);
        let sway = (time * 3.0 + seed).sin() * 10.0;
        let col = if ember % 2 == 0 {
            Color::new(1.0, 0.85, 0.2, 0.85)
        } else {
            Color::new(1.0, 0.35, 0.05, 0.8)
        };
        draw_circle(ex + sway, ey, 2.2, col);
    }
}

// =========================================================================
// 6. ORBITAL STATION (Deep Space Zero-G Deck with Blue Earth Horizon)
// =========================================================================
fn render_orbital_station(time: f32, x_left: f32, x_right: f32, full_w: f32, court_width: f32) {
    // Deep cosmic space extending into upper sky
    draw_rectangle(x_left, -500.0, full_w, 920.0, Color::new(0.02, 0.02, 0.06, 1.0));

    // Cosmic nebula cloud
    draw_circle(court_width * 0.3125, 150.0, 170.0, Color::new(0.45, 0.1, 0.65, 0.14));
    draw_circle(court_width * 0.725, 180.0, 150.0, Color::new(0.1, 0.4, 0.75, 0.16));

    // Starfield across full screen
    for s in 0..85 {
        let seed = s as f32 * 17.13;
        let sx = x_left + (seed * 83.1) % full_w;
        let sy = -500.0 + (seed * 47.7) % 830.0;
        let twinkle = ((time * 3.0 + seed).sin() * 0.4 + 0.6).clamp(0.1, 1.0);
        let sz = if s % 5 == 0 { 2.2 } else { 1.2 };
        draw_circle(sx, sy, sz, Color::new(0.9, 0.95, 1.0, twinkle));
    }

    // Curved Earth limb in the background
    let earth_cx = court_width * 0.5;
    let earth_cy = 980.0;
    let earth_r = 750.0;
    draw_circle(earth_cx, earth_cy, earth_r + 28.0, Color::new(0.15, 0.6, 1.0, 0.15));
    draw_circle(earth_cx, earth_cy, earth_r + 12.0, Color::new(0.3, 0.8, 1.0, 0.35));
    draw_circle(earth_cx, earth_cy, earth_r, Color::new(0.08, 0.35, 0.65, 1.0));

    // Station solar array & truss across full width
    draw_rectangle(x_left, 310.0, full_w, 14.0, Color::new(0.2, 0.22, 0.3, 1.0));
    draw_rectangle_lines(x_left, 310.0, full_w, 14.0, 1.5, Color::new(0.4, 0.6, 0.9, 0.6));

    let mut px = x_left + 10.0;
    while px <= x_right {
        draw_rectangle(px, 326.0, 56.0, 20.0, Color::new(0.08, 0.25, 0.55, 0.9));
        draw_rectangle_lines(px, 326.0, 56.0, 20.0, 1.0, Color::new(0.3, 0.8, 1.0, 0.5));
        px += 68.0;
    }

    // High-Tech Zero-G Court Platform
    let deck_y = 410.0;
    draw_rectangle(x_left, deck_y, full_w, VIRTUAL_HEIGHT - deck_y + 50.0, Color::new(0.1, 0.12, 0.18, 1.0));
    draw_rectangle(x_left, deck_y, full_w, 12.0, Color::new(0.15, 0.2, 0.3, 0.8));

    let mut lx = x_left;
    while lx <= x_right {
        draw_line(lx, deck_y, lx, VIRTUAL_HEIGHT + 50.0, 1.0, Color::new(0.25, 0.35, 0.5, 0.35));
        lx += 80.0;
    }
}

// =========================================================================
// TOURNAMENT COURT BOUNDARY GUIDELINES & FLOODLIGHT TOWERS
// =========================================================================
fn render_court_boundaries(time: f32, arena: ArenaType, court_width: f32) {
    let (line_col, glow_col) = match arena {
        ArenaType::SunsetBeach | ArenaType::ClassicRemaster | ArenaType::OlympicBeach => (
            Color::new(1.0, 1.0, 1.0, 0.92),
            Color::new(0.3, 0.85, 1.0, 0.5),
        ),
        ArenaType::GrandStadium => (
            Color::new(1.0, 0.96, 0.75, 0.95),
            Color::new(0.2, 0.85, 1.0, 0.65),
        ),
        ArenaType::CyberNeon => (
            Color::new(0.2, 0.95, 1.0, 0.95),
            Color::new(1.0, 0.15, 0.75, 0.6),
        ),
        ArenaType::AlpinePeaks => (
            Color::new(0.3, 0.85, 1.0, 0.92),
            Color::new(0.6, 0.95, 1.0, 0.5),
        ),
        ArenaType::MagmaCaldera => (
            Color::new(1.0, 0.45, 0.1, 0.95),
            Color::new(1.0, 0.2, 0.05, 0.6),
        ),
        ArenaType::OrbitalStation => (
            Color::new(0.1, 0.85, 1.0, 0.95),
            Color::new(0.3, 0.7, 1.0, 0.6),
        ),
    };

    let left_plane = 0.0;
    let right_plane = court_width;

    // 1. Official Ground Baseline Tape (0..court_width at y = 500)
    draw_line(left_plane, GROUND_PLANE_HEIGHT_MAX, right_plane, GROUND_PLANE_HEIGHT_MAX, 3.5, line_col);

    // 2. High-Tech Vertical Court Barrier Posts at x = 0.0 and x = right_plane
    // Perfectly aligns with physical wall bounce planes, extending up into the sky so players have crisp visual feedback on high balls
    let top_bounce_y = -500.0;
    let pulse = (time * 4.0).sin().abs() * 0.2 + 0.8;
    draw_line(left_plane, top_bounce_y, left_plane, GROUND_PLANE_HEIGHT_MAX, 2.5, Color::new(glow_col.r, glow_col.g, glow_col.b, glow_col.a * pulse));
    draw_line(right_plane, top_bounce_y, right_plane, GROUND_PLANE_HEIGHT_MAX, 2.5, Color::new(glow_col.r, glow_col.g, glow_col.b, glow_col.a * pulse));

    // Outer subtle glow halo
    draw_line(left_plane - 1.5, top_bounce_y, left_plane - 1.5, GROUND_PLANE_HEIGHT_MAX, 5.0, Color::new(glow_col.r, glow_col.g, glow_col.b, 0.15));
    draw_line(right_plane + 1.5, top_bounce_y, right_plane + 1.5, GROUND_PLANE_HEIGHT_MAX, 5.0, Color::new(glow_col.r, glow_col.g, glow_col.b, 0.15));

    // Height indicator tick hashes along vertical bounce bars
    let mut hash_y = top_bounce_y + 25.0;
    while hash_y < GROUND_PLANE_HEIGHT_MAX {
        let is_major = (hash_y as i32).abs() % 100 < 10;
        let h_len = if is_major { 10.0 } else { 5.0 };
        let h_alpha = if is_major { 0.45 } else { 0.25 };
        draw_line(left_plane, hash_y, left_plane + h_len, hash_y, 1.2, Color::new(glow_col.r, glow_col.g, glow_col.b, h_alpha));
        draw_line(right_plane - h_len, hash_y, right_plane, hash_y, 1.2, Color::new(glow_col.r, glow_col.g, glow_col.b, h_alpha));
        hash_y += 50.0;
    }

    // 3. Modern Tournament Floodlight Stanchions on Court Flanks (only for stadiums/arenas)
    let has_floodlights = matches!(arena, ArenaType::GrandStadium | ArenaType::OlympicBeach | ArenaType::CyberNeon);
    if has_floodlights {
        let draw_floodlight_tower = |tx: f32, is_left: bool| {
            // Metallic vertical mast
            draw_rectangle(tx - 3.0, 40.0, 6.0, GROUND_PLANE_HEIGHT_MAX - 40.0, Color::new(0.15, 0.18, 0.25, 0.95));
            draw_rectangle_lines(tx - 3.0, 40.0, 6.0, GROUND_PLANE_HEIGHT_MAX - 40.0, 1.0, Color::new(0.3, 0.4, 0.55, 0.8));

            // Top luminaire head
            let head_x = tx - 14.0;
            draw_rectangle(head_x, 32.0, 28.0, 12.0, Color::new(0.08, 0.1, 0.15, 1.0));
            draw_rectangle_lines(head_x, 32.0, 28.0, 12.0, 1.0, Color::new(0.4, 0.6, 0.85, 0.9));

            // Glowing LED light bar
            draw_rectangle(head_x + 2.0, 36.0, 24.0, 4.0, Color::new(1.0, 0.98, 0.9, 0.95));
            draw_circle(tx, 38.0, 18.0, Color::new(1.0, 0.95, 0.8, 0.15));

            // Downward illumination cone
            let target_x = if is_left { court_width * 0.25 } else { court_width * 0.75 };
            draw_triangle(
                Vec2::new(tx - 6.0, 44.0),
                Vec2::new(tx + 6.0, 44.0),
                Vec2::new(target_x, GROUND_PLANE_HEIGHT_MAX),
                Color::new(1.0, 0.95, 0.8, 0.04),
            );
        };

        draw_floodlight_tower(left_plane - 14.0, true);
        draw_floodlight_tower(right_plane + 14.0, false);
    }
}

// =========================================================================
// OPEN-SOURCE PERIMETER ADVERTISING BOARDS (Full-Screen Stadium Barrier)
// =========================================================================
thread_local! {
    static SPONSOR_LOGOS: RefCell<Option<HashMap<&'static str, Texture2D>>> = RefCell::new(None);
}

fn with_sponsor_logos<F: FnOnce(&HashMap<&'static str, Texture2D>)>(f: F) {
    SPONSOR_LOGOS.with(|cell| {
        let mut borrow = cell.borrow_mut();
        if borrow.is_none() {
            let embedded: [(&'static str, &[u8]); 13] = [
                ("PhotoCraft", include_bytes!("../../assets/sponsors/photocraft.png")),
                ("PdfCraft", include_bytes!("../../assets/sponsors/pdfcraft.png")),
                ("FilmCraft", include_bytes!("../../assets/sponsors/filmcraft.png")),
                ("WordCraft", include_bytes!("../../assets/sponsors/wordcraft.png")),
                ("LightCraft", include_bytes!("../../assets/sponsors/lightcraft.png")),
                ("VectorCraft", include_bytes!("../../assets/sponsors/vectorcraft.png")),
                ("EffectCraft", include_bytes!("../../assets/sponsors/effectcraft.png")),
                ("CADCraft", include_bytes!("../../assets/sponsors/cadcraft.png")),
                ("GridCraft", include_bytes!("../../assets/sponsors/gridcraft.png")),
                ("DesignCraft", include_bytes!("../../assets/sponsors/designcraft.png")),
                ("FigCraft", include_bytes!("../../assets/sponsors/figcraft.png")),
                ("DeckCraft", include_bytes!("../../assets/sponsors/deckcraft.png")),
                ("SoundCraft", include_bytes!("../../assets/sponsors/soundcraft.png")),
            ];
            let mut map = HashMap::with_capacity(embedded.len());
            for (name, bytes) in embedded {
                let tex = Texture2D::from_file_with_format(bytes, Some(ImageFormat::Png));
                tex.set_filter(FilterMode::Linear);
                map.insert(name, tex);
            }
            *borrow = Some(map);
        }
        if let Some(ref map) = *borrow {
            f(map);
        }
    });
}

fn render_perimeter_boards(time: f32, _arena: ArenaType, x_left: f32, x_right: f32, court_width: f32) {
    let board_y = 398.0;
    let board_h = 24.0;
    let board_start_x = 35.0;
    let board_end_x = court_width - 35.0;
    let board_w = board_end_x - board_start_x;

    // 1. Continuous side barriers extending to the display flanks
    if x_left < board_start_x - 10.0 {
        let left_w = (board_start_x - 8.0) - x_left;
        draw_rectangle(x_left, board_y + 2.0, left_w, board_h - 4.0, Color::new(0.08, 0.1, 0.15, 0.85));
        draw_line(x_left, board_y + 2.0, board_start_x - 8.0, board_y + 2.0, 1.5, Color::new(0.3, 0.5, 0.7, 0.6));
    }
    if x_right > board_end_x + 10.0 {
        let right_w = x_right - (board_end_x + 8.0);
        draw_rectangle(board_end_x + 8.0, board_y + 2.0, right_w, board_h - 4.0, Color::new(0.08, 0.1, 0.15, 0.85));
        draw_line(board_end_x + 8.0, board_y + 2.0, x_right, board_y + 2.0, 1.5, Color::new(0.3, 0.5, 0.7, 0.6));
    }

    // 2. Dark metallic chassis & support stanchions for main active court boards
    draw_rectangle(board_start_x - 4.0, board_y - 2.0, board_w + 8.0, board_h + 4.0, Color::new(0.06, 0.07, 0.1, 0.95));
    draw_rectangle_lines(board_start_x - 4.0, board_y - 2.0, board_w + 8.0, board_h + 4.0, 1.5, Color::new(0.3, 0.35, 0.45, 0.7));

    // Top illuminated LED rail
    draw_line(board_start_x - 4.0, board_y - 2.0, board_end_x + 4.0, board_y - 2.0, 2.0, Color::new(0.6, 0.85, 1.0, 0.85));

    // Open-Source Craft suite & FOSS projects roster
    struct AdBanner {
        name: &'static str,
        accent: Color,
        tagline: &'static str,
    }

    let banners = [
        // 13 Craft apps strictly sorted by GitHub stars
        AdBanner { name: "PhotoCraft", accent: Color::new(0.28, 0.72, 1.0, 1.0), tagline: "open photo editor" },
        AdBanner { name: "PdfCraft", accent: Color::new(0.95, 0.35, 0.32, 1.0), tagline: "open PDF reader" },
        AdBanner { name: "FilmCraft", accent: Color::new(0.65, 0.45, 0.98, 1.0), tagline: "video editor" },
        AdBanner { name: "WordCraft", accent: Color::new(0.25, 0.58, 0.96, 1.0), tagline: "word processor" },
        AdBanner { name: "LightCraft", accent: Color::new(0.96, 0.68, 0.22, 1.0), tagline: "raw photo studio" },
        AdBanner { name: "VectorCraft", accent: Color::new(0.98, 0.52, 0.18, 1.0), tagline: "vector graphics" },
        AdBanner { name: "EffectCraft", accent: Color::new(0.75, 0.38, 0.96, 1.0), tagline: "motion & VFX" },
        AdBanner { name: "CADCraft", accent: Color::new(0.22, 0.82, 0.68, 1.0), tagline: "2D & 3D CAD" },
        AdBanner { name: "GridCraft", accent: Color::new(0.22, 0.82, 0.45, 1.0), tagline: "open spreadsheet" },
        AdBanner { name: "DesignCraft", accent: Color::new(0.92, 0.32, 0.65, 1.0), tagline: "page layout" },
        AdBanner { name: "FigCraft", accent: Color::new(0.55, 0.52, 1.0, 1.0), tagline: "UI prototyping" },
        AdBanner { name: "DeckCraft", accent: Color::new(0.96, 0.44, 0.24, 1.0), tagline: "presentations" },
        AdBanner { name: "SoundCraft", accent: Color::new(0.18, 0.84, 0.88, 1.0), tagline: "open-source DAW" },
        // FOSS distros & projects
        AdBanner { name: "arch linux", accent: Color::new(0.09, 0.58, 0.82, 1.0), tagline: "keep it simple" },
        AdBanner { name: "debian", accent: Color::new(0.84, 0.04, 0.33, 1.0), tagline: "universal os" },
        AdBanner { name: "fedora", accent: Color::new(0.2, 0.45, 0.85, 1.0), tagline: "innovate" },
        AdBanner { name: "CachyOS", accent: Color::new(0.0, 0.85, 0.45, 1.0), tagline: "speed & kernel" },
        AdBanner { name: "KDE Plasma", accent: Color::new(0.11, 0.6, 0.95, 1.0), tagline: "simple & powerful" },
        AdBanner { name: "Rust", accent: Color::new(0.95, 0.45, 0.15, 1.0), tagline: "empowering everyone" },
        AdBanner { name: "GODOT", accent: Color::new(0.28, 0.55, 0.82, 1.0), tagline: "free game engine" },
        AdBanner { name: "blender", accent: Color::new(0.92, 0.52, 0.08, 1.0), tagline: "open 3D creation" },
        AdBanner { name: "FFmpeg", accent: Color::new(0.15, 0.75, 0.25, 1.0), tagline: "fast multimedia" },
        AdBanner { name: "mpv", accent: Color::new(0.48, 0.18, 0.65, 1.0), tagline: "video player" },
        AdBanner { name: "git", accent: Color::new(0.94, 0.32, 0.2, 1.0), tagline: "source control" },
        AdBanner { name: "OpenCode", accent: Color::new(0.18, 0.68, 0.98, 1.0), tagline: "open development" },
        AdBanner { name: "Navidrome", accent: Color::new(0.0, 0.78, 0.85, 1.0), tagline: "music server" },
        AdBanner { name: "NEOVIM", accent: Color::new(0.35, 0.72, 0.28, 1.0), tagline: "hyper extensible" },
    ];

    let panel_w = 146.0;
    let num_panels = banners.len();
    let scroll_speed = 18.0; // pixels per second
    let total_carousel_w = num_panels as f32 * panel_w;
    let offset_x = (time * scroll_speed) % total_carousel_w;

    // Render continuous banner ribbon within board bounds
    with_sponsor_logos(|logos| {
        for i in 0..(num_panels * 2) {
            let banner_idx = i % num_panels;
            let px = board_start_x + (i as f32 * panel_w) - offset_x;

            if px + panel_w < board_start_x || px > board_end_x {
                continue;
            }

            let ad = &banners[banner_idx];
            let clamped_w = panel_w.min(board_end_x - px);

            // Panel plate
            draw_rectangle(px, board_y, clamped_w, board_h, Color::new(0.08, 0.1, 0.15, 0.9));
            draw_rectangle_lines(px, board_y, clamped_w, board_h, 1.0, Color::new(0.2, 0.25, 0.35, 0.5));

            // Left accent indicator tag
            draw_rectangle(px + 2.0, board_y + 3.0, 3.0, board_h - 6.0, ad.accent);

            // Embedded Craft PNG Logo or Handcrafted Vector Project Logo
            if let Some(tex) = logos.get(ad.name) {
                if px + 25.0 < board_end_x {
                    draw_texture_ex(
                        tex,
                        px + 7.0,
                        board_y + 3.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(18.0, 18.0)),
                            ..Default::default()
                        },
                    );
                }
            } else if px + 24.0 < board_end_x {
                draw_project_logo(ad.name, px + 8.0, board_y + 4.0, ad.accent);
            }

            // Name and mini tagline
            if px + 28.0 < board_end_x - 15.0 {
                draw_text(ad.name, px + 28.0, board_y + 13.0, 11.0, WHITE);
                draw_text(ad.tagline, px + 28.0, board_y + 21.0, 7.5, ad.accent);
            }
        }
    });

    // Glass sheen gloss overlay across top half of boards
    draw_rectangle(board_start_x, board_y, board_w, 6.0, Color::new(1.0, 1.0, 1.0, 0.08));
}

fn draw_project_logo(name: &str, x: f32, y: f32, accent: Color) {
    match name {
        "arch linux" => {
            // Stylized Arch cyan triangle peak with bottom cutout
            draw_triangle(
                Vec2::new(x + 1.0, y + 14.0),
                Vec2::new(x + 7.5, y + 1.5),
                Vec2::new(x + 14.0, y + 14.0),
                accent,
            );
            draw_triangle(
                Vec2::new(x + 4.5, y + 14.0),
                Vec2::new(x + 7.5, y + 8.5),
                Vec2::new(x + 10.5, y + 14.0),
                Color::new(0.08, 0.1, 0.15, 1.0),
            );
        }
        "debian" => {
            // Iconic Debian red spiral
            draw_circle_lines(x + 7.5, y + 7.5, 5.5, 1.6, accent);
            draw_circle(x + 7.5, y + 5.5, 2.2, accent);
            draw_line(x + 7.5, y + 5.5, x + 10.5, y + 10.5, 1.6, accent);
        }
        "fedora" => {
            // Fedora blue circle with white infinity emblem
            draw_circle(x + 7.5, y + 7.5, 6.5, accent);
            draw_circle_lines(x + 6.0, y + 7.5, 2.0, 1.2, WHITE);
            draw_circle_lines(x + 9.0, y + 7.5, 2.0, 1.2, WHITE);
            draw_line(x + 5.0, y + 7.5, x + 10.0, y + 7.5, 1.2, WHITE);
        }
        "CachyOS" => {
            // Speed chevron / green energy bolt
            draw_triangle(
                Vec2::new(x + 2.0, y + 13.0),
                Vec2::new(x + 9.0, y + 2.0),
                Vec2::new(x + 14.0, y + 7.0),
                accent,
            );
            draw_triangle(
                Vec2::new(x + 4.5, y + 13.0),
                Vec2::new(x + 8.5, y + 7.0),
                Vec2::new(x + 13.0, y + 13.0),
                Color::new(0.08, 0.1, 0.15, 1.0),
            );
        }
        "KDE Plasma" => {
            // Blue gear with white spokes
            draw_circle(x + 7.5, y + 7.5, 6.0, accent);
            draw_circle(x + 7.5, y + 7.5, 2.4, Color::new(0.08, 0.1, 0.15, 1.0));
            draw_line(x + 7.5, y + 2.5, x + 7.5, y + 12.5, 1.3, WHITE);
            draw_line(x + 2.5, y + 7.5, x + 12.5, y + 7.5, 1.3, WHITE);
        }
        "Rust" => {
            // Orange cog / gear with 'R'
            draw_circle_lines(x + 7.5, y + 7.5, 5.5, 1.8, accent);
            draw_circle(x + 7.5, y + 7.5, 3.2, Color::new(0.08, 0.1, 0.15, 1.0));
            draw_text("R", x + 5.2, y + 10.5, 9.0, accent);
        }
        "GODOT" => {
            // Godot robot head with antenna & eyes
            draw_rectangle(x + 2.5, y + 4.5, 10.0, 8.0, accent);
            draw_line(x + 3.5, y + 2.5, x + 3.5, y + 4.5, 1.2, accent);
            draw_line(x + 11.5, y + 2.5, x + 11.5, y + 4.5, 1.2, accent);
            draw_circle(x + 5.2, y + 8.0, 1.2, WHITE);
            draw_circle(x + 9.8, y + 8.0, 1.2, WHITE);
            draw_line(x + 5.5, y + 10.5, x + 9.5, y + 10.5, 1.0, WHITE);
        }
        "blender" => {
            // Orange circle with blue center & radiating arms
            draw_circle(x + 7.5, y + 8.5, 5.0, accent);
            draw_circle(x + 7.5, y + 8.5, 2.4, Color::new(0.2, 0.6, 0.9, 1.0));
            draw_circle(x + 7.5, y + 8.5, 1.0, WHITE);
            draw_line(x + 7.5, y + 2.0, x + 7.5, y + 4.5, 1.3, accent);
            draw_line(x + 3.5, y + 3.5, x + 5.5, y + 5.5, 1.3, accent);
            draw_line(x + 11.5, y + 3.5, x + 9.5, y + 5.5, 1.3, accent);
        }
        "FFmpeg" => {
            // Multi-colored blocks
            draw_rectangle(x + 2.5, y + 3.5, 4.0, 8.5, accent);
            draw_rectangle(x + 7.5, y + 6.5, 4.5, 5.5, Color::new(0.85, 0.2, 0.2, 1.0));
            draw_rectangle(x + 7.5, y + 2.5, 4.5, 3.2, Color::new(0.2, 0.6, 0.9, 1.0));
        }
        "mpv" => {
            // Purple circle with white play triangle
            draw_circle(x + 7.5, y + 7.5, 6.2, accent);
            draw_triangle(
                Vec2::new(x + 6.0, y + 4.5),
                Vec2::new(x + 11.0, y + 7.5),
                Vec2::new(x + 6.0, y + 10.5),
                WHITE,
            );
        }
        "git" => {
            // Red diamond with commit branch nodes
            draw_line(x + 7.5, y + 2.5, x + 12.5, y + 7.5, 1.5, accent);
            draw_line(x + 12.5, y + 7.5, x + 7.5, y + 12.5, 1.5, accent);
            draw_line(x + 7.5, y + 12.5, x + 2.5, y + 7.5, 1.5, accent);
            draw_line(x + 2.5, y + 7.5, x + 7.5, y + 2.5, 1.5, accent);
            draw_circle(x + 5.5, y + 7.5, 1.2, WHITE);
            draw_circle(x + 9.5, y + 7.5, 1.2, WHITE);
            draw_line(x + 5.5, y + 7.5, x + 9.5, y + 7.5, 1.0, WHITE);
        }
        "OpenCode" => {
            // Cyan code tags: < / >
            draw_text("<", x + 1.5, y + 11.0, 10.0, accent);
            draw_text("/", x + 6.0, y + 11.0, 9.0, WHITE);
            draw_text(">", x + 9.5, y + 11.0, 10.0, accent);
        }
        "Navidrome" => {
            // Soundwave bars in teal
            draw_rectangle(x + 2.5, y + 7.0, 2.0, 6.0, accent);
            draw_rectangle(x + 5.5, y + 4.0, 2.0, 9.0, accent);
            draw_rectangle(x + 8.5, y + 2.0, 2.0, 11.0, accent);
            draw_rectangle(x + 11.5, y + 5.0, 2.0, 8.0, accent);
        }
        "NEOVIM" => {
            // Green/Cyan angular 'N'
            draw_line(x + 3.5, y + 12.5, x + 3.5, y + 2.5, 2.0, accent);
            draw_line(x + 3.5, y + 2.5, x + 11.5, y + 12.5, 2.0, Color::new(0.2, 0.6, 0.9, 1.0));
            draw_line(x + 11.5, y + 12.5, x + 11.5, y + 2.5, 2.0, accent);
        }
        _ => {
            draw_circle(x + 7.5, y + 7.5, 5.0, accent);
        }
    }
}

// Helpers
fn render_stylized_cloud(x: f32, y: f32, scale: f32, alpha: f32) {
    let cloud_color = Color::new(1.0, 1.0, 1.0, alpha);
    draw_circle(x, y, 22.0 * scale, cloud_color);
    draw_circle(x + 20.0 * scale, y - 6.0 * scale, 28.0 * scale, cloud_color);
    draw_circle(x + 44.0 * scale, y, 24.0 * scale, cloud_color);
    draw_circle(x + 22.0 * scale, y + 8.0 * scale, 20.0 * scale, cloud_color);
}

fn render_palm_tree(trunk_x: f32, ground_y: f32, scale: f32, sway: f32, tilt_right: bool) {
    let top_x = trunk_x + if tilt_right { 35.0 } else { -35.0 } + sway;
    let top_y = ground_y - 180.0 * scale;

    let segments = 8;
    let mut prev_p = Vec2::new(trunk_x, ground_y);
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let curr_x = trunk_x + (top_x - trunk_x) * t.powf(1.4);
        let curr_y = ground_y + (top_y - ground_y) * t;
        let curr_p = Vec2::new(curr_x, curr_y);
        let w = (16.0 - t * 8.0) * scale;

        draw_line(prev_p.x, prev_p.y, curr_p.x, curr_p.y, w, Color::new(0.48, 0.32, 0.18, 1.0));
        prev_p = curr_p;
    }

    let frond_angles: [f32; 7] = [-1.4, -0.9, -0.4, 0.1, 0.6, 1.1, 1.6];
    for angle in frond_angles {
        let frond_len = 65.0 * scale;
        let end_x = top_x + angle.cos() * frond_len + sway * 0.8;
        let end_y = top_y + angle.sin() * frond_len + 12.0;

        draw_line(top_x, top_y, end_x, end_y, 4.0 * scale, Color::new(0.18, 0.58, 0.22, 0.95));
        draw_line(top_x, top_y - 2.0, end_x, end_y, 2.0 * scale, Color::new(0.35, 0.78, 0.32, 0.85));
    }
}

// =========================================================================
// PROCEDURAL STADIUM SPECTATOR CROWD ENGINE
// =========================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrowdStyle {
    GrandColiseum,
    OlympicBeach,
    Cyberpunk,
}

fn render_stadium_crowd(
    time: f32,
    x_left: f32,
    x_right: f32,
    row_y_coords: &[f32],
    row_scales: &[f32],
    style: CrowdStyle,
    ball_pos: Vec2,
    cheer: CheerState,
    court_width: f32,
) {
    let full_w = x_right - x_left;
    let num_rows = row_y_coords.len();

    for (r, (&row_y, &scale)) in row_y_coords.iter().zip(row_scales.iter()).enumerate() {
        // 1. Bleacher Step Platform / Bench
        match style {
            CrowdStyle::GrandColiseum => {
                let bench_h = 24.0 * scale;
                draw_rectangle(x_left, row_y - 2.0 * scale, full_w, bench_h, Color::new(0.12, 0.14, 0.20, 0.95));
                draw_rectangle(x_left, row_y - 2.0 * scale, full_w, 3.0 * scale, Color::new(0.22, 0.26, 0.36, 0.9));
                draw_line(x_left, row_y - 2.0 * scale, x_right, row_y - 2.0 * scale, 1.2, Color::new(0.4, 0.5, 0.65, 0.8));
                let mut post_x = x_left + 40.0;
                while post_x < x_right {
                    draw_line(post_x, row_y - 12.0 * scale, post_x, row_y + 4.0 * scale, 1.5, Color::new(0.3, 0.38, 0.5, 0.7));
                    post_x += 120.0 * scale;
                }
            }
            CrowdStyle::OlympicBeach => {
                let bench_h = 22.0 * scale;
                draw_rectangle(x_left, row_y - 2.0 * scale, full_w, bench_h, Color::new(0.68, 0.55, 0.38, 0.95));
                draw_rectangle(x_left, row_y - 2.0 * scale, full_w, 3.5 * scale, Color::new(0.82, 0.70, 0.52, 0.95));
                draw_line(x_left, row_y - 2.0 * scale, x_right, row_y - 2.0 * scale, 1.2, Color::new(0.92, 0.82, 0.65, 0.9));
                let mut post_x = x_left + 40.0;
                while post_x < x_right {
                    draw_line(post_x, row_y - 11.0 * scale, post_x, row_y + 3.0 * scale, 1.5, Color::new(0.5, 0.38, 0.25, 0.7));
                    post_x += 110.0 * scale;
                }
            }
            CrowdStyle::Cyberpunk => {
                let bench_h = 20.0 * scale;
                draw_rectangle(x_left, row_y - 1.0 * scale, full_w, bench_h, Color::new(0.06, 0.05, 0.14, 0.95));
                draw_line(x_left, row_y - 1.0 * scale, x_right, row_y - 1.0 * scale, 1.5, Color::new(0.2, 0.85, 1.0, 0.75));
                let mut post_x = x_left + 30.0;
                while post_x < x_right {
                    draw_line(post_x, row_y - 8.0 * scale, post_x, row_y + 3.0 * scale, 1.2, Color::new(1.0, 0.15, 0.75, 0.5));
                    post_x += 90.0 * scale;
                }
            }
        }

        // 2. Spectator Blobs with Persistent Loyalty & Reactions
        let step_x = 22.0 * scale;
        let mut col = 0;
        let mut cur_x = x_left + 8.0 * scale;

        while cur_x < x_right {
            col += 1;
            let seed = ((col as f32 * 17.13 + r as f32 * 43.71).sin() * 93821.17).fract().abs();
            let cur_x_pos = cur_x;
            cur_x += step_x;

            if seed < 0.08 {
                continue;
            }

            let jitter_x = ((seed * 77.0).fract() - 0.5) * 5.0 * scale;
            let spec_x = cur_x_pos + jitter_x;

            // Persistent Loyalty Assignment:
            // Blobs tendentially sit in the half of the player they support, with natural mixed random seating
            let seat_rand = ((col as f32 * 91.31 + r as f32 * 53.79).sin() * 43758.54).fract().abs();
            let net_x = court_width * 0.5;
            let left_trans = net_x - 60.0;
            let right_trans = net_x + 60.0;
            let prob_left = if spec_x < left_trans {
                0.82
            } else if spec_x > right_trans {
                0.18
            } else {
                0.82 - ((spec_x - left_trans) / 120.0) * 0.64
            };
            let favored_side = if seat_rand < prob_left { PlayerSide::Left } else { PlayerSide::Right };

            let is_favored_scorer = cheer.last_scorer == Some(favored_side) && cheer.cheer_timer > 0.0;
            let is_favored_winner = cheer.match_winner == Some(favored_side);
            let is_rival_scorer = cheer.last_scorer == Some(favored_side.opponent()) && cheer.cheer_timer > 0.0;
            let is_rival_winner = cheer.match_winner == Some(favored_side.opponent());

            // 1. Occasional, distinctly visible Stadium Wave ("La Ola")
            // Runs once every 16 seconds for 3.5 seconds, sweeping from left to right across the bleachers.
            let wave_period = 16.0;
            let wave_phase = time % wave_period;
            let mut wave_lift = 0.0;
            let mut is_in_wave = false;

            if wave_phase < 3.5 && cheer.cheer_timer <= 0.0 && cheer.match_winner.is_none() {
                // Wave crest travels across the entire bleacher width
                let wave_x = x_left + (wave_phase / 3.5) * (full_w + 140.0) - 70.0;
                let dist = (spec_x - wave_x).abs();
                let wave_span = 80.0 * scale;
                if dist < wave_span {
                    let crest = 1.0 - (dist / wave_span);
                    let shape = (crest * std::f32::consts::PI * 0.5).sin().powf(2.0);
                    wave_lift = shape * (16.0 * scale);
                    if shape > 0.4 {
                        is_in_wave = true;
                    }
                }
            }

            // 2. Calm, subtle idle breathing when not cheering
            let idle_breath = (time * 1.8 + col as f32 * 0.25).sin() * (0.5 * scale);

            // 3. Clear, unmistakable celebrations on Point Scored and Match Victory!
            let mut cheer_lift = 0.0;
            let mut is_cheering = false;
            if is_favored_winner {
                // Euphoric victory celebration for preferred player
                let hop = (time * 8.0 + col as f32 * 1.1).sin().max(0.0).powf(1.6);
                cheer_lift = hop * (24.0 * scale);
                is_cheering = true;
            } else if is_favored_scorer {
                // Clear, punchy celebratory hops when preferred player scores a point!
                let hop = (time * 7.5 + col as f32 * 1.2).sin().max(0.0).powf(1.6);
                let fade = (cheer.cheer_timer / 2.5).clamp(0.0, 1.0);
                cheer_lift = hop * (18.0 * scale) * fade;
                is_cheering = true;
            } else if is_rival_scorer || is_rival_winner {
                // Disappointed slump when opponent scores
                cheer_lift = -3.5 * scale;
            }

            let total_lift = (wave_lift + idle_breath + cheer_lift).max(-4.0 * scale);
            let spec_y = row_y - total_lift;

            // Squash and stretch
            let stretch_y = 1.0 + (total_lift / (15.0 * scale)).clamp(-0.2, 0.55);
            let stretch_x = 1.0 / stretch_y.abs().max(0.5).sqrt();

            // Fan Color Allocation reflecting team preference
            let fan_rand = ((col as f32 * 47.9 + r as f32 * 83.1).sin() * 12345.67).fract().abs();
            let team_theme = match favored_side {
                PlayerSide::Left => cheer.left_theme,
                PlayerSide::Right => cheer.right_theme,
            };
            let (team_top, team_bot, _highlight) = team_theme.colors();

            // Depth attenuation
            let depth = 0.74 + (r as f32 / num_rows as f32) * 0.26;

            // Base body color: 65% team top/bot, 35% tinted fan variant
            let raw_col = if fan_rand < 0.40 {
                team_top
            } else if fan_rand < 0.70 {
                team_bot
            } else {
                Color::new(
                    (team_top.r * 0.6 + 0.3).min(1.0),
                    (team_top.g * 0.6 + 0.3).min(1.0),
                    (team_top.b * 0.6 + 0.3).min(1.0),
                    1.0,
                )
            };

            let base_col = Color::new(raw_col.r * depth, raw_col.g * depth, raw_col.b * depth, 1.0);

            // Body Geometry
            let body_w = 7.5 * scale * stretch_x;
            let body_h = 10.0 * scale * stretch_y;
            let head_r = 6.2 * scale * stretch_x;

            // Bleacher Contact Shadow
            draw_ellipse(spec_x, row_y + 1.5 * scale, body_w * 0.85, 2.2 * scale, 0.0, Color::new(0.0, 0.0, 0.0, 0.35 * depth));

            // Lower body
            draw_ellipse(spec_x, spec_y, body_w, body_h * 0.75, 0.0, base_col);

            // Head dome
            let head_y = spec_y - body_h * 0.4;
            draw_circle(spec_x, head_y, head_r, base_col);

            // Specular head highlight
            draw_circle(spec_x - head_r * 0.32, head_y - head_r * 0.32, head_r * 0.3, Color::new(1.0, 1.0, 1.0, 0.38 * depth));

            // Team Clothing & Accessories (Scarves, Headbands) in Player's Colors
            if fan_rand < 0.55 {
                // Team Scarf around neck
                let scarf_y = head_y + head_r * 0.62;
                draw_line(spec_x - body_w * 0.72, scarf_y, spec_x + body_w * 0.72, scarf_y, 2.8 * scale, team_top);
                // Scarf fluttering fringe
                let flutter = (time * 4.5 + spec_x * 0.3).sin() * (2.2 * scale);
                draw_line(spec_x + body_w * 0.35, scarf_y, spec_x + body_w * 0.55, scarf_y + 5.0 * scale + flutter, 2.0 * scale, team_bot);
            } else if fan_rand < 0.85 {
                // Team Headband across forehead
                let band_y = head_y - head_r * 0.28;
                draw_line(spec_x - head_r * 0.88, band_y, spec_x + head_r * 0.88, band_y, 2.2 * scale, team_top);
                draw_circle(spec_x, band_y, 1.3 * scale, team_bot);
            }

            // Cute Googly Eyes tracking Ball or downcast if rival scores
            let eye_y = head_y;
            let eye_spacing = 2.8 * scale;
            let eye_r = 2.2 * scale;
            let pupil_r = 1.15 * scale;

            let to_ball = Vec2::new(ball_pos.x - spec_x, ball_pos.y - eye_y);
            let dir = to_ball / to_ball.length().max(1.0);
            let pupil_off = if is_rival_scorer || is_rival_winner {
                Vec2::new(0.0, 1.1 * scale)
            } else {
                dir * (1.0 * scale)
            };

            if style == CrowdStyle::Cyberpunk && fan_rand > 0.45 {
                let visor_w = 11.5 * scale;
                let visor_h = 3.2 * scale;
                let v_col = if fan_rand > 0.72 {
                    Color::new(0.2, 0.95, 1.0, 0.95)
                } else {
                    Color::new(1.0, 0.2, 0.8, 0.95)
                };
                draw_rectangle(spec_x - visor_w * 0.5, eye_y - visor_h * 0.5, visor_w, visor_h, v_col);
            } else if style == CrowdStyle::OlympicBeach && fan_rand > 0.75 {
                let glass_w = 4.8 * scale;
                let glass_h = 3.2 * scale;
                draw_rectangle(spec_x - eye_spacing - glass_w * 0.5, eye_y - glass_h * 0.5, glass_w, glass_h, Color::new(0.1, 0.12, 0.18, 0.95));
                draw_rectangle(spec_x + eye_spacing - glass_w * 0.5, eye_y - glass_h * 0.5, glass_w, glass_h, Color::new(0.1, 0.12, 0.18, 0.95));
                draw_line(spec_x - eye_spacing, eye_y - 1.0 * scale, spec_x + eye_spacing, eye_y - 1.0 * scale, 1.2 * scale, Color::new(0.1, 0.12, 0.18, 0.95));
                draw_line(spec_x - eye_spacing - 1.2 * scale, eye_y - 1.0 * scale, spec_x - eye_spacing + 1.2 * scale, eye_y + 1.0 * scale, 0.8 * scale, Color::new(1.0, 1.0, 1.0, 0.5));
                draw_line(spec_x + eye_spacing - 1.2 * scale, eye_y - 1.0 * scale, spec_x + eye_spacing + 1.2 * scale, eye_y + 1.0 * scale, 0.8 * scale, Color::new(1.0, 1.0, 1.0, 0.5));
            } else {
                draw_circle(spec_x - eye_spacing, eye_y, eye_r, WHITE);
                draw_circle(spec_x + eye_spacing, eye_y, eye_r, WHITE);

                draw_circle(spec_x - eye_spacing + pupil_off.x, eye_y + pupil_off.y, pupil_r, Color::new(0.06, 0.06, 0.1, 1.0));
                draw_circle(spec_x + eye_spacing + pupil_off.x, eye_y + pupil_off.y, pupil_r, Color::new(0.06, 0.06, 0.1, 1.0));

                if scale > 0.5 {
                    draw_circle(spec_x - eye_spacing + pupil_off.x - 0.4 * scale, eye_y + pupil_off.y - 0.4 * scale, 0.4 * scale, WHITE);
                    draw_circle(spec_x + eye_spacing + pupil_off.x - 0.4 * scale, eye_y + pupil_off.y - 0.4 * scale, 0.4 * scale, WHITE);
                }
            }

            // Raised arms during celebrations and laola crest
            if is_cheering || is_in_wave {
                let arm_lift = if is_cheering { 0.85 } else { 0.72 };
                draw_circle(spec_x - body_w * 0.95, spec_y - body_h * arm_lift, 2.2 * scale, base_col);
                draw_circle(spec_x + body_w * 0.95, spec_y - body_h * arm_lift, 2.2 * scale, base_col);
            }

            // Waving Fan Flags & Signs in Player's Colors
            if fan_rand > 0.78 || is_favored_winner || (is_favored_scorer && fan_rand > 0.45) {
                let pole_x = spec_x + body_w * 0.85;
                let pole_y1 = spec_y;
                let pole_y0 = spec_y - 15.0 * scale;
                draw_line(pole_x, pole_y1, pole_x, pole_y0, 1.2 * scale, Color::new(0.8, 0.82, 0.88, 0.85));
                let flag_speed = if is_cheering { 16.0 } else { 8.0 };
                let flap = (time * flag_speed + spec_x * 0.2).sin() * (2.8 * scale);

                if fan_rand > 0.88 {
                    // Waving Pennant Flag
                    draw_triangle(
                        Vec2::new(pole_x, pole_y0),
                        Vec2::new(pole_x + 10.5 * scale, pole_y0 + 3.5 * scale + flap),
                        Vec2::new(pole_x, pole_y0 + 7.0 * scale),
                        team_top,
                    );
                    draw_circle(pole_x + 3.5 * scale, pole_y0 + 3.5 * scale + flap * 0.4, 1.5 * scale, team_bot);
                } else {
                    // Cheering Placard / Sign
                    let sign_w = 12.0 * scale;
                    let sign_h = 7.0 * scale;
                    let sign_x = pole_x - 2.0 * scale;
                    let sign_y = pole_y0 - sign_h + flap * 0.4;
                    draw_rectangle(sign_x, sign_y, sign_w, sign_h, team_top);
                    draw_rectangle_lines(sign_x, sign_y, sign_w, sign_h, 1.0, team_bot);
                    draw_circle(sign_x + sign_w * 0.5, sign_y + sign_h * 0.5, 1.6 * scale, team_bot);
                }
            }

            // Cyberpunk Glowsticks in Team Accent
            if style == CrowdStyle::Cyberpunk && fan_rand < 0.22 {
                let stick_x = spec_x + 5.0 * scale;
                let flap = (time * 4.0 + spec_x).sin() * (4.0 * scale);
                draw_line(stick_x, spec_y - 2.0 * scale, stick_x + 6.0 * scale, spec_y - 12.0 * scale + flap, 2.0 * scale, team_top);
            }
        }
    }
}

// =========================================================================
// REAL-TIME COURT SIZE VISUALIZER (HUD Measurement Overlay in World Space)
// =========================================================================
pub fn render_court_size_visualizer(time: f32, court_width: f32, aspect: f32) {
    let max_w = (600.0 * aspect).max(800.0);
    let is_at_window_border = (court_width - max_w).abs() <= 1.0;
    let is_default = (court_width - 800.0).abs() <= 1.0;

    let left_x = 0.0;
    let right_x = court_width;
    let ground_y = GROUND_PLANE_HEIGHT_MAX; // 500.0
    let sky_y = -350.0;

    let pulse = (time * 6.0).sin().abs() * 0.25 + 0.75;
    let accent_col = if is_at_window_border {
        Color::new(1.0, 0.92, 0.3, 0.95) // Gold when reaching window edges
    } else {
        Color::new(0.2, 0.9, 1.0, 0.95) // Cyan
    };
    let glow_col = Color::new(accent_col.r, accent_col.g, accent_col.b, 0.35 * pulse);

    // 1. Semi-transparent illuminated court floor highlight
    draw_rectangle(
        left_x,
        ground_y - 70.0,
        court_width,
        70.0,
        Color::new(accent_col.r * 0.5, accent_col.g * 0.7, accent_col.b, 0.12 * pulse),
    );
    draw_line(left_x, ground_y, right_x, ground_y, 4.5, accent_col);

    // 2. High-Tech Boundary Vertical Beacons (Left & Right)
    for (x, is_left) in [(left_x, true), (right_x, false)] {
        // Outer soft glow beam
        draw_line(x, sky_y, x, ground_y, 14.0, glow_col);
        // Inner core laser beam
        draw_line(x, sky_y, x, ground_y, 3.0, Color::new(1.0, 1.0, 1.0, 0.95));

        // Animated scanning chevrons moving along the beam
        let chevrons = 6;
        for i in 0..chevrons {
            let offset_t = (time * 0.8 + i as f32 / chevrons as f32) % 1.0;
            let cy = ground_y - offset_t * (ground_y - sky_y);
            let dir = if is_left { 12.0 } else { -12.0 };
            draw_line(x, cy - 8.0, x + dir, cy, 2.5, accent_col);
            draw_line(x + dir, cy, x, cy + 8.0, 2.5, accent_col);
        }

        // Ground anchor pulse rings
        draw_circle(x, ground_y, 12.0, glow_col);
        draw_circle(x, ground_y, 6.0, accent_col);
        draw_circle_lines(x, ground_y, 18.0 * pulse, 1.5, glow_col);
    }

    // 3. Center Net Guide Line
    let net_x = court_width * 0.5;
    draw_line(net_x, 160.0, net_x, ground_y, 1.5, Color::new(1.0, 0.9, 0.3, 0.45 * pulse));

    // 4. Overhead Dimension Measurement Caliper
    let cal_y = 120.0;
    // Caliper line
    draw_line(left_x, cal_y, right_x, cal_y, 2.5, accent_col);
    // End vertical ticks
    draw_line(left_x, cal_y - 14.0, left_x, cal_y + 14.0, 3.0, accent_col);
    draw_line(right_x, cal_y - 14.0, right_x, cal_y + 14.0, 3.0, accent_col);

    // Outward pointing arrows at boundaries
    draw_line(left_x, cal_y, left_x + 14.0, cal_y - 8.0, 2.5, accent_col);
    draw_line(left_x, cal_y, left_x + 14.0, cal_y + 8.0, 2.5, accent_col);
    draw_line(right_x, cal_y, right_x - 14.0, cal_y - 8.0, 2.5, accent_col);
    draw_line(right_x, cal_y, right_x - 14.0, cal_y + 8.0, 2.5, accent_col);

    // Center Badge with current measurement and status
    let label = if is_at_window_border {
        format!("{:.0}px [WINDOW BORDERS / SCREEN EDGE]", court_width)
    } else if is_default {
        format!("{:.0}px [STANDARD COURT WIDTH]", court_width)
    } else {
        format!("{:.0}px [ENLARGED COURT WIDTH]", court_width)
    };

    let font_size = 18.0;
    let dim = measure_text(&label, None, font_size as u16, 1.0);
    let box_w = dim.width + 36.0;
    let box_h = 32.0;
    let box_x = net_x - box_w * 0.5;
    let box_y = cal_y - box_h * 0.5;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::new(0.04, 0.08, 0.16, 0.95));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 1.8, accent_col);
    draw_text(&label, box_x + 18.0, box_y + 22.0, font_size, if is_at_window_border { Color::new(1.0, 0.95, 0.35, 1.0) } else { WHITE });

    // Left & Right boundary callout labels at floor level
    draw_text("0px [LEFT WALL]", left_x + 10.0, ground_y - 12.0, 14.0, accent_col);
    let right_lbl = format!("[RIGHT WALL] {:.0}px", court_width);
    let r_dim = measure_text(&right_lbl, None, 14, 1.0);
    draw_text(&right_lbl, right_x - r_dim.width - 10.0, ground_y - 12.0, 14.0, accent_col);
}
