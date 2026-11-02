// Blobby Volley 3 - 2026 Remastered Arenas & Full-Screen Visual Overhaul
// Eliminates pillarboxing / black borders by dynamically expanding scenic horizons across all aspect ratios (16:10, 16:9, 21:9)
// while perfectly maintaining 100% authentic physics court dynamics (0..800x600).
use macroquad::prelude::*;
use glam::Vec2;
use crate::constants::*;
use crate::physics::PlayerSide;

#[derive(Debug, Clone, Copy, Default)]
pub struct CheerState {
    pub last_scorer: Option<PlayerSide>,
    pub cheer_timer: f32,
    pub match_winner: Option<PlayerSide>,
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

pub fn render_arena(arena: ArenaType, time: f32, view_x0: f32, view_w: f32, ball_pos: Vec2, cheer: CheerState) {
    // Calculate full dynamic screen bounds to seamlessly fill all displays (e.g. 16:10, 16:9, 21:9 ultrawide)
    let x_left = view_x0.min(-250.0);
    let x_right = (view_x0 + view_w).max(1050.0);
    let full_w = x_right - x_left;

    match arena {
        ArenaType::SunsetBeach => render_sunset_beach(time, x_left, x_right, full_w),
        ArenaType::GrandStadium => render_grand_stadium(time, x_left, x_right, full_w, ball_pos, cheer),
        ArenaType::CyberNeon => render_cyber_neon(time, x_left, x_right, full_w, ball_pos, cheer),
        ArenaType::OlympicBeach => render_olympic_beach(time, x_left, x_right, full_w, ball_pos, cheer),
        ArenaType::ClassicRemaster => render_classic_remaster(time, x_left, x_right, full_w),
        ArenaType::AlpinePeaks => render_alpine_peaks(time, x_left, x_right, full_w),
        ArenaType::MagmaCaldera => render_magma_caldera(time, x_left, x_right, full_w),
        ArenaType::OrbitalStation => render_orbital_station(time, x_left, x_right, full_w),
    }

    // Render tournament court boundary guidelines and floodlight towers
    render_court_boundaries(time, arena);

    // Render in-game perimeter advertising boards across active court and side extensions
    render_perimeter_boards(time, arena, x_left, x_right);
}

// =========================================================================
// 1. COPACABANA SUNSET BEACH (Modern Tropical Sunset with Parallax Vista)
// =========================================================================
fn render_sunset_beach(time: f32, x_left: f32, x_right: f32, full_w: f32) {
    let sky_h = 420.0;
    let bands = 16;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = t * sky_h;
        let h = sky_h / bands as f32 + 1.0;

        let r = 0.16 + t * 0.82;
        let g = 0.06 + t * 0.48;
        let b = 0.48 - t * 0.38;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Distant mountain ridge silhouettes (Sugarloaf & coastal peaks)
    let mountain_pts = [
        (x_left + 40.0, 260.0, 320.0),
        (180.0, 240.0, 260.0),
        (380.0, 270.0, 290.0),
        (620.0, 250.0, 310.0),
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
    let sun_x = 400.0;
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
    render_palm_tree(705.0, GROUND_PLANE_HEIGHT_MAX, 1.1, sway_right * 0.9, false);
    render_palm_tree(x_right - 80.0, GROUND_PLANE_HEIGHT_MAX, 1.25, sway_right, false);

    // Sand dunes extending across entire display
    let sand_top = GROUND_PLANE_HEIGHT_MAX - 75.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top + 50.0;
    draw_rectangle(x_left, sand_top, full_w, sand_h, Color::new(0.92, 0.76, 0.48, 1.0));
    draw_rectangle(x_left, sand_top, full_w, 18.0, Color::new(0.82, 0.64, 0.38, 0.5));

    // Court sand highlight inside official play boundary (0..800)
    draw_rectangle(0.0, sand_top + 10.0, VIRTUAL_WIDTH, sand_h, Color::new(0.95, 0.80, 0.52, 0.35));
}

// =========================================================================
// 2. BLOBBY GRAND ARENA (Premier World Championship Stadium with Spectators)
// =========================================================================
fn render_grand_stadium(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState) {
    // 1. Indoor Stadium Enclosed Ceiling Dome
    let ceiling_h = 175.0;
    for i in 0..10 {
        let t = i as f32 / 10.0;
        let y = t * ceiling_h;
        let h = ceiling_h / 10.0 + 1.0;
        let r = 0.05 + t * 0.07;
        let g = 0.06 + t * 0.09;
        let b = 0.12 + t * 0.15;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Steel Lattice Roof Truss Arches
    let arch_y = 65.0;
    draw_line(x_left, arch_y, x_right, arch_y, 3.0, Color::new(0.25, 0.32, 0.45, 0.85));
    draw_line(x_left, arch_y + 18.0, x_right, arch_y + 18.0, 2.0, Color::new(0.18, 0.24, 0.35, 0.8));
    let mut tx = x_left + 20.0;
    while tx < x_right {
        draw_line(tx, arch_y, tx + 25.0, arch_y + 18.0, 1.5, Color::new(0.25, 0.32, 0.45, 0.7));
        draw_line(tx + 25.0, arch_y + 18.0, tx + 50.0, arch_y, 1.5, Color::new(0.25, 0.32, 0.45, 0.7));
        tx += 50.0;
    }

    // Suspended Arena Sound System Speakers
    draw_rectangle(180.0, arch_y + 18.0, 32.0, 18.0, Color::new(0.08, 0.1, 0.15, 0.95));
    draw_rectangle(590.0, arch_y + 18.0, 32.0, 18.0, Color::new(0.08, 0.1, 0.15, 0.95));

    // 2. Twin Sideline Arena Display Boards (x: 40..220 and 580..760)
    // Placed on the stadium wings so the center court is 100% unobstructed for the HUD
    let screens = [(55.0, 78.0, 185.0, 68.0, "★ TEAM BLUE ★"), (560.0, 78.0, 185.0, 68.0, "★ TEAM RED ★")];
    for &(sx, sy, sw, sh, s_title) in &screens {
        // Cables from ceiling truss
        draw_line(sx + 15.0, arch_y + 18.0, sx + 15.0, sy, 2.0, Color::new(0.15, 0.2, 0.28, 0.9));
        draw_line(sx + sw - 15.0, arch_y + 18.0, sx + sw - 15.0, sy, 2.0, Color::new(0.15, 0.2, 0.28, 0.9));

        // Chassis Frame
        draw_rectangle(sx - 2.0, sy - 2.0, sw + 4.0, sh + 4.0, Color::new(0.08, 0.1, 0.15, 0.98));
        draw_rectangle_lines(sx - 2.0, sy - 2.0, sw + 4.0, sh + 4.0, 1.5, Color::new(0.95, 0.75, 0.2, 0.85));

        // Screen display
        draw_rectangle(sx, sy, sw, sh, Color::new(0.04, 0.07, 0.16, 1.0));

        // Header
        let pulse = (time * 4.0).sin().abs() * 0.2 + 0.8;
        draw_rectangle(sx, sy, sw, 15.0, Color::new(0.1, 0.22, 0.48, 0.9));
        let title_dim = measure_text(s_title, None, 10, 1.0);
        draw_text(s_title, sx + (sw - title_dim.width) * 0.5, sy + 11.0, 10.0, Color::new(1.0, 0.85, 0.25, pulse));

        // Live Audio / Sine Visualizer Bars
        for bar in 0..12 {
            let bx = sx + 12.0 + bar as f32 * 13.5;
            let bar_h = (time * 6.0 + bar as f32 * 0.8 + sx * 0.01).sin().abs() * 20.0 + 4.0;
            let by = sy + 44.0 - bar_h * 0.5;
            let col = if bar % 2 == 0 { Color::new(0.2, 0.95, 1.0, 0.9) } else { Color::new(1.0, 0.35, 0.75, 0.9) };
            draw_rectangle(bx, by, 7.0, bar_h, col);
        }

        // Screen Live Ticker
        draw_rectangle(sx, sy + sh - 13.0, sw, 13.0, Color::new(0.05, 0.08, 0.16, 0.95));
        let ticker_msg = "OFFICIAL 2026 WORLD TOUR";
        let t_dim = measure_text(ticker_msg, None, 8, 1.0);
        draw_text(ticker_msg, sx + (sw - t_dim.width) * 0.5, sy + sh - 3.0, 8.0, Color::new(0.4, 0.95, 1.0, 0.85));
    }

    // 3. Upper Balcony Tier Bleachers (y = 165, 192, 219)
    let upper_rows = [165.0, 192.0, 219.0];
    let upper_scales = [0.48, 0.53, 0.58];
    render_stadium_crowd(time, x_left, x_right, &upper_rows, &upper_scales, CrowdStyle::GrandColiseum, ball_pos, cheer);

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
    render_stadium_crowd(time, x_left, x_right, &lower_rows, &lower_scales, CrowdStyle::GrandColiseum, ball_pos, cheer);

    // 5. Hardwood Parquet Championship Court Floor
    let court_top = GROUND_PLANE_HEIGHT_MAX - 75.0; // 425.0
    let court_h = VIRTUAL_HEIGHT - court_top + 150.0;
    draw_rectangle(x_left, court_top, full_w, court_h, Color::new(0.84, 0.65, 0.42, 1.0));
    draw_rectangle(0.0, court_top, VIRTUAL_WIDTH, court_h, Color::new(0.89, 0.70, 0.46, 0.6));

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
fn render_cyber_neon(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState) {
    let bands = 14;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = t * 240.0;
        let h = 240.0 / bands as f32 + 1.0;
        let r = 0.06 + t * 0.35;
        let g = 0.02 + t * 0.04;
        let b = 0.16 + t * 0.42;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Giant Wireframe Retro Sun
    let sun_x = 400.0;
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
    render_stadium_crowd(time, x_left, x_right, &cyber_rows, &cyber_scales, CrowdStyle::Cyberpunk, ball_pos, cheer);

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
    for i in 0..=v_rays {
        let bottom_x = x_left + i as f32 * (full_w / v_rays as f32);
        let vanish_x = 400.0 + (bottom_x - 400.0) * 0.2;
        draw_line(vanish_x, grid_top, bottom_x, VIRTUAL_HEIGHT + 50.0, 1.2, Color::new(1.0, 0.15, 0.7, 0.35));
    }
}

// =========================================================================
// 4. OLYMPIC BEACH STADIUM (Open-Air Seaside Tournament Grandstands)
// =========================================================================
fn render_olympic_beach(time: f32, x_left: f32, x_right: f32, full_w: f32, ball_pos: Vec2, cheer: CheerState) {
    // 1. Sunny Coastal Azure Sky
    let sky_h = 240.0;
    for i in 0..10 {
        let t = i as f32 / 10.0;
        let y = t * sky_h;
        let h = sky_h / 10.0 + 1.0;
        let r = 0.2 + t * 0.35;
        let g = 0.55 + t * 0.35;
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
    render_stadium_crowd(time, x_left, x_right, &beach_rows, &beach_scales, CrowdStyle::OlympicBeach, ball_pos, cheer);

    // 4. Golden Sand Court
    let sand_top = GROUND_PLANE_HEIGHT_MAX - 75.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top + 50.0;
    draw_rectangle(x_left, sand_top, full_w, sand_h, Color::new(0.92, 0.77, 0.49, 1.0));
    draw_rectangle(x_left, sand_top, full_w, 16.0, Color::new(0.82, 0.65, 0.38, 0.5));
    draw_rectangle(0.0, sand_top + 8.0, VIRTUAL_WIDTH, sand_h, Color::new(0.96, 0.82, 0.54, 0.35));
}

// =========================================================================
// 3. CLASSIC BEACH HD (Nostalgic Sunny Sky & Crystal Ocean)
// =========================================================================
fn render_classic_remaster(time: f32, x_left: f32, x_right: f32, full_w: f32) {
    let sky_h = 360.0;
    for i in 0..12 {
        let t = i as f32 / 12.0;
        let y = t * sky_h;
        let h = sky_h / 12.0 + 1.0;
        let r = 0.22 + t * 0.48;
        let g = 0.52 + t * 0.38;
        let b = 0.95 + t * 0.05;
        draw_rectangle(x_left, y, full_w, h, Color::new(r, g, b, 1.0));
    }

    // Brilliant golden summer sun
    draw_circle(680.0, 85.0, 52.0, Color::new(1.0, 0.98, 0.8, 0.18));
    draw_circle(680.0, 85.0, 34.0, Color::new(1.0, 0.98, 0.85, 1.0));

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
    let sky_h = 360.0;
    for i in 0..14 {
        let t = i as f32 / 14.0;
        let y = t * sky_h;
        let h = sky_h / 14.0 + 1.0;
        let r = 0.08 + t * 0.35;
        let g = 0.15 + t * 0.45;
        let b = 0.35 + t * 0.55;
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
    let sky_h = 360.0;
    for i in 0..14 {
        let t = i as f32 / 14.0;
        let y = t * sky_h;
        let h = sky_h / 14.0 + 1.0;
        let r = 0.16 + t * 0.70;
        let g = 0.03 + t * 0.18;
        let b = 0.04 + t * 0.05;
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
fn render_orbital_station(time: f32, x_left: f32, x_right: f32, full_w: f32) {
    // Deep cosmic space
    draw_rectangle(x_left, 0.0, full_w, 420.0, Color::new(0.02, 0.02, 0.06, 1.0));

    // Cosmic nebula cloud
    draw_circle(250.0, 150.0, 170.0, Color::new(0.45, 0.1, 0.65, 0.14));
    draw_circle(580.0, 180.0, 150.0, Color::new(0.1, 0.4, 0.75, 0.16));

    // Starfield across full screen
    for s in 0..65 {
        let seed = s as f32 * 17.13;
        let sx = x_left + (seed * 83.1) % full_w;
        let sy = (seed * 47.7) % 330.0;
        let twinkle = ((time * 3.0 + seed).sin() * 0.4 + 0.6).clamp(0.1, 1.0);
        let sz = if s % 5 == 0 { 2.2 } else { 1.2 };
        draw_circle(sx, sy, sz, Color::new(0.9, 0.95, 1.0, twinkle));
    }

    // Curved Earth limb in the background
    let earth_cx = 400.0;
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
fn render_court_boundaries(time: f32, arena: ArenaType) {
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

    // 1. Official Ground Baseline Tape (0..800 at y = 500)
    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.5, line_col);

    // 2. High-Tech Vertical Court Barrier Posts at x = 0.0 and x = 800.0
    // Perfectly aligns with physical wall bounce planes, giving players crisp visual feedback
    let pulse = (time * 4.0).sin().abs() * 0.2 + 0.8;
    draw_line(LEFT_PLANE, 0.0, LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, 2.5, Color::new(glow_col.r, glow_col.g, glow_col.b, glow_col.a * pulse));
    draw_line(RIGHT_PLANE, 0.0, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 2.5, Color::new(glow_col.r, glow_col.g, glow_col.b, glow_col.a * pulse));

    // Outer subtle glow halo
    draw_line(LEFT_PLANE - 1.5, 0.0, LEFT_PLANE - 1.5, GROUND_PLANE_HEIGHT_MAX, 5.0, Color::new(glow_col.r, glow_col.g, glow_col.b, 0.15));
    draw_line(RIGHT_PLANE + 1.5, 0.0, RIGHT_PLANE + 1.5, GROUND_PLANE_HEIGHT_MAX, 5.0, Color::new(glow_col.r, glow_col.g, glow_col.b, 0.15));

    // 3. Modern Tournament Floodlight Stanchions on Court Flanks
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
        let target_x = if is_left { 200.0 } else { 600.0 };
        draw_triangle(
            Vec2::new(tx - 6.0, 44.0),
            Vec2::new(tx + 6.0, 44.0),
            Vec2::new(target_x, GROUND_PLANE_HEIGHT_MAX),
            Color::new(1.0, 0.95, 0.8, 0.04),
        );
    };

    draw_floodlight_tower(LEFT_PLANE - 14.0, true);
    draw_floodlight_tower(RIGHT_PLANE + 14.0, false);
}

// =========================================================================
// OPEN-SOURCE PERIMETER ADVERTISING BOARDS (Full-Screen Stadium Barrier)
// =========================================================================
fn render_perimeter_boards(time: f32, _arena: ArenaType, x_left: f32, x_right: f32) {
    let board_y = 398.0;
    let board_h = 24.0;
    let board_start_x = 35.0;
    let board_end_x = 765.0;
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

    // Open-Source distros & projects roster
    struct AdBanner {
        name: &'static str,
        accent: Color,
        tagline: &'static str,
    }

    let banners = [
        AdBanner { name: "arch linux", accent: Color::new(0.09, 0.58, 0.82, 1.0), tagline: "keep it simple" },
        AdBanner { name: "debian", accent: Color::new(0.84, 0.04, 0.33, 1.0), tagline: "universal os" },
        AdBanner { name: "fedora", accent: Color::new(0.2, 0.45, 0.85, 1.0), tagline: "innovate" },
        AdBanner { name: "CachyOS", accent: Color::new(0.0, 0.85, 0.45, 1.0), tagline: "speed & kernel" },
        AdBanner { name: "KDE Plasma", accent: Color::new(0.11, 0.6, 0.95, 1.0), tagline: "simple & powerful" },
        AdBanner { name: "Rust", accent: Color::new(0.95, 0.45, 0.15, 1.0), tagline: "empowering everyone" },
        AdBanner { name: "GODOT", accent: Color::new(0.28, 0.55, 0.82, 1.0), tagline: "free game engine" },
        AdBanner { name: "blender", accent: Color::new(0.92, 0.52, 0.08, 1.0), tagline: "open 3D creation" },
        AdBanner { name: "FFmpeg", accent: Color::new(0.15, 0.75, 0.25, 1.0), tagline: "fast multimedia" },
        AdBanner { name: "VLC media", accent: Color::new(1.0, 0.55, 0.05, 1.0), tagline: "plays everything" },
        AdBanner { name: "git", accent: Color::new(0.94, 0.32, 0.2, 1.0), tagline: "source control" },
        AdBanner { name: "NEOVIM", accent: Color::new(0.35, 0.72, 0.28, 1.0), tagline: "hyper extensible" },
    ];

    let panel_w = 120.0;
    let num_panels = banners.len();
    let scroll_speed = 18.0; // pixels per second
    let total_carousel_w = num_panels as f32 * panel_w;
    let offset_x = (time * scroll_speed) % total_carousel_w;

    // Render continuous banner ribbon within board bounds
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
        draw_rectangle(px + 2.0, board_y + 3.0, 4.0, board_h - 6.0, ad.accent);

        // Name and mini tagline
        if px + 12.0 < board_end_x - 20.0 {
            draw_text(ad.name, px + 10.0, board_y + 13.0, 12.0, WHITE);
            draw_text(ad.tagline, px + 10.0, board_y + 21.0, 8.0, ad.accent);
        }
    }

    // Glass sheen gloss overlay across top half of boards
    draw_rectangle(board_start_x, board_y, board_w, 6.0, Color::new(1.0, 1.0, 1.0, 0.08));
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
            let prob_left = if spec_x < 340.0 {
                0.82
            } else if spec_x > 460.0 {
                0.18
            } else {
                0.82 - ((spec_x - 340.0) / 120.0) * 0.64
            };
            let favored_side = if seat_rand < prob_left { PlayerSide::Left } else { PlayerSide::Right };

            let is_favored_scorer = cheer.last_scorer == Some(favored_side) && cheer.cheer_timer > 0.0;
            let is_favored_winner = cheer.match_winner == Some(favored_side);
            let is_rival_scorer = cheer.last_scorer == Some(favored_side.opponent()) && cheer.cheer_timer > 0.0;
            let is_rival_winner = cheer.match_winner == Some(favored_side.opponent());

            // Stadium Wave ("La Ola")
            let wave_speed = 3.6;
            let wave_phase = spec_x * 0.013 - time * wave_speed;
            let in_wave = wave_phase.sin().max(0.0).powf(3.5);
            let wave_lift = in_wave * 13.0 * scale;

            // Cheering / idle bounce
            let bounce = (time * 2.6 + col as f32 * 0.5 + r as f32 * 1.3).sin() * (1.2 * scale);
            let is_hyper = ((col as f32 * 31.7 + r as f32 * 17.3).sin() * 43758.54).fract().abs() > 0.82;
            let hyper_hop = if is_hyper {
                (time * 6.0 + col as f32 * 1.2).sin().max(0.0).powf(2.0) * (8.0 * scale)
            } else {
                0.0
            };

            // Celebratory Jump when favored player scores or wins!
            let mut cheer_lift = 0.0;
            let mut is_cheering = false;
            if is_favored_winner {
                // Extra hard celebratory cheering when preferred player wins the match!
                let intensity = (time * 8.5 + col as f32 * 1.3).sin().max(0.0).powf(1.8);
                cheer_lift = intensity * (22.0 * scale);
                is_cheering = true;
            } else if is_favored_scorer {
                // Cheering when preferred player scores a point!
                let fade = (cheer.cheer_timer / 2.5).clamp(0.0, 1.0);
                let jump = (time * 7.5 + col as f32 * 1.5).sin().max(0.0).powf(2.0);
                cheer_lift = jump * (15.0 * scale) * fade;
                is_cheering = true;
            } else if is_rival_scorer || is_rival_winner {
                cheer_lift = -2.5 * scale;
            }

            let total_lift = (wave_lift + hyper_hop + bounce + cheer_lift).max(-3.0 * scale);
            let spec_y = row_y - total_lift;

            // Squash and stretch
            let stretch_y = 1.0 + (total_lift / (15.0 * scale)).clamp(-0.2, 0.55);
            let stretch_x = 1.0 / stretch_y.abs().max(0.5).sqrt();

            // Fan Color Allocation reflecting team preference
            let fan_rand = ((col as f32 * 47.9 + r as f32 * 83.1).sin() * 12345.67).fract().abs();
            let raw_col = match favored_side {
                PlayerSide::Left => {
                    match ((fan_rand * 10.0) as usize) % 5 {
                        0 => Color::new(0.18, 0.65, 1.0, 1.0),
                        1 => Color::new(0.2, 0.88, 0.45, 1.0),
                        2 => Color::new(0.15, 0.95, 0.9, 1.0),
                        3 => Color::new(0.35, 0.72, 1.0, 1.0),
                        _ => Color::new(0.92, 0.95, 1.0, 1.0),
                    }
                }
                PlayerSide::Right => {
                    match ((fan_rand * 10.0) as usize) % 5 {
                        0 => Color::new(0.98, 0.32, 0.32, 1.0),
                        1 => Color::new(1.0, 0.58, 0.15, 1.0),
                        2 => Color::new(1.0, 0.82, 0.2, 1.0),
                        3 => Color::new(0.85, 0.3, 0.95, 1.0),
                        _ => Color::new(1.0, 0.42, 0.7, 1.0),
                    }
                }
            };

            let depth = 0.74 + (r as f32 / num_rows as f32) * 0.26;
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

            // Cheering arms raised in the air!
            if is_cheering || total_lift > 5.0 * scale {
                draw_circle(spec_x - body_w * 0.95, spec_y - body_h * 0.72, 2.2 * scale, base_col);
                draw_circle(spec_x + body_w * 0.95, spec_y - body_h * 0.72, 2.2 * scale, base_col);
            }

            // Waving Fan Flags
            if fan_rand > 0.85 || is_favored_winner {
                let pole_x = spec_x + body_w * 0.85;
                let pole_y1 = spec_y;
                let pole_y0 = spec_y - 14.0 * scale;
                draw_line(pole_x, pole_y1, pole_x, pole_y0, 1.2 * scale, Color::new(0.7, 0.75, 0.8, 0.8));
                let flag_speed = if is_cheering { 16.0 } else { 8.0 };
                let flap = (time * flag_speed + spec_x * 0.2).sin() * (2.8 * scale);
                draw_triangle(
                    Vec2::new(pole_x, pole_y0),
                    Vec2::new(pole_x + 9.5 * scale, pole_y0 + 3.2 * scale + flap),
                    Vec2::new(pole_x, pole_y0 + 6.5 * scale),
                    base_col,
                );
            }

            // Cyberpunk Glowsticks
            if style == CrowdStyle::Cyberpunk && fan_rand < 0.22 {
                let stick_x = spec_x + 5.0 * scale;
                let flap = (time * 4.0 + spec_x).sin() * (4.0 * scale);
                let g_col = if fan_rand < 0.11 {
                    Color::new(0.2, 1.0, 0.85, 0.9)
                } else {
                    Color::new(1.0, 0.25, 0.85, 0.9)
                };
                draw_line(stick_x, spec_y - 2.0 * scale, stick_x + 6.0 * scale, spec_y - 12.0 * scale + flap, 2.0 * scale, g_col);
            }
        }
    }
}
