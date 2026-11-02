// Blobby Volley 3 - 2026 Remastered Arenas & Open-Source Perimeter Advertising Boards
use macroquad::prelude::*;
use glam::Vec2;
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaType {
    SunsetBeach,
    CyberNeon,
    ClassicRemaster,
    AlpinePeaks,
    MagmaCaldera,
    OrbitalStation,
}

impl ArenaType {
    pub fn name(self) -> &'static str {
        match self {
            ArenaType::SunsetBeach => "Copacabana Sunset",
            ArenaType::CyberNeon => "Neo Cyber Stadium",
            ArenaType::ClassicRemaster => "Classic Beach HD",
            ArenaType::AlpinePeaks => "Alpine Peaks (Snow)",
            ArenaType::MagmaCaldera => "Magma Caldera (Volcano)",
            ArenaType::OrbitalStation => "Orbital Station (Space)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ArenaType::SunsetBeach => ArenaType::CyberNeon,
            ArenaType::CyberNeon => ArenaType::ClassicRemaster,
            ArenaType::ClassicRemaster => ArenaType::AlpinePeaks,
            ArenaType::AlpinePeaks => ArenaType::MagmaCaldera,
            ArenaType::MagmaCaldera => ArenaType::OrbitalStation,
            ArenaType::OrbitalStation => ArenaType::SunsetBeach,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ArenaType::SunsetBeach => ArenaType::OrbitalStation,
            ArenaType::CyberNeon => ArenaType::SunsetBeach,
            ArenaType::ClassicRemaster => ArenaType::CyberNeon,
            ArenaType::AlpinePeaks => ArenaType::ClassicRemaster,
            ArenaType::MagmaCaldera => ArenaType::AlpinePeaks,
            ArenaType::OrbitalStation => ArenaType::MagmaCaldera,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => ArenaType::SunsetBeach,
            1 => ArenaType::CyberNeon,
            2 => ArenaType::ClassicRemaster,
            3 => ArenaType::AlpinePeaks,
            4 => ArenaType::MagmaCaldera,
            _ => ArenaType::OrbitalStation,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            ArenaType::SunsetBeach => 0,
            ArenaType::CyberNeon => 1,
            ArenaType::ClassicRemaster => 2,
            ArenaType::AlpinePeaks => 3,
            ArenaType::MagmaCaldera => 4,
            ArenaType::OrbitalStation => 5,
        }
    }
}

pub fn render_arena(arena: ArenaType, time: f32) {
    match arena {
        ArenaType::SunsetBeach => render_sunset_beach(time),
        ArenaType::CyberNeon => render_cyber_neon(time),
        ArenaType::ClassicRemaster => render_classic_remaster(time),
        ArenaType::AlpinePeaks => render_alpine_peaks(time),
        ArenaType::MagmaCaldera => render_magma_caldera(time),
        ArenaType::OrbitalStation => render_orbital_station(time),
    }

    // Render in-game perimeter advertising boards across all arenas
    render_perimeter_boards(time, arena);
}

// =========================================================================
// 1. COPACABANA SUNSET BEACH (Modern Tropical Sunset)
// =========================================================================
fn render_sunset_beach(time: f32) {
    let sky_h = 420.0;
    let bands = 14;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = t * sky_h;
        let h = sky_h / bands as f32 + 1.0;

        let r = 0.22 + t * 0.76;
        let g = 0.08 + t * 0.45;
        let b = 0.45 - t * 0.35;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    let sun_x = 400.0;
    let sun_y = 230.0;
    draw_circle(sun_x, sun_y, 95.0, Color::new(1.0, 0.75, 0.2, 0.12));
    draw_circle(sun_x, sun_y, 65.0, Color::new(1.0, 0.85, 0.3, 0.25));
    draw_circle(sun_x, sun_y, 42.0, Color::new(1.0, 0.96, 0.75, 0.95));

    render_stylized_cloud(120.0 + (time * 6.0) % 900.0 - 50.0, 90.0, 1.2, 0.35);
    render_stylized_cloud(520.0 + (time * 4.5) % 900.0 - 50.0, 130.0, 0.9, 0.28);
    render_stylized_cloud(280.0 + (time * 5.2) % 900.0 - 50.0, 70.0, 0.7, 0.22);

    let ocean_top = 310.0;
    let ocean_h = 100.0;
    draw_rectangle(0.0, ocean_top, VIRTUAL_WIDTH, ocean_h, Color::new(0.08, 0.22, 0.45, 1.0));

    let shimmer_w = 140.0;
    draw_rectangle(
        sun_x - shimmer_w * 0.5,
        ocean_top,
        shimmer_w,
        ocean_h,
        Color::new(1.0, 0.65, 0.2, 0.28),
    );

    for wave in 0..4 {
        let wave_y = ocean_top + 14.0 * wave as f32 + 10.0;
        let speed = (wave as f32 + 1.0) * 1.5;
        let mut prev_x = 0.0;
        let mut prev_y = wave_y;

        let mut x = 0.0;
        while x <= VIRTUAL_WIDTH {
            let offset_y = ((x * 0.03 + time * speed).sin()) * 3.5;
            let curr_y = wave_y + offset_y;
            draw_line(prev_x, prev_y, x, curr_y, 2.0, Color::new(0.65, 0.88, 0.95, 0.45));
            prev_x = x;
            prev_y = curr_y;
            x += 16.0;
        }
    }

    let sway_left = (time * 1.2).sin() * 8.0;
    let sway_right = ((time * 1.4) + 1.2).sin() * 9.0;
    render_palm_tree(95.0, GROUND_PLANE_HEIGHT_MAX, 1.1, sway_left, true);
    render_palm_tree(705.0, GROUND_PLANE_HEIGHT_MAX, 1.15, sway_right, false);

    let sand_top = GROUND_PLANE_HEIGHT_MAX - 70.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top;
    draw_rectangle(0.0, sand_top, VIRTUAL_WIDTH, sand_h, Color::new(0.92, 0.76, 0.48, 1.0));
    draw_rectangle(0.0, sand_top, VIRTUAL_WIDTH, 18.0, Color::new(0.82, 0.64, 0.38, 0.5));

    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.0, Color::new(1.0, 1.0, 1.0, 0.85));
}

// =========================================================================
// 2. NEO TOKYO CYBER STADIUM (Synthwave / Cyberpunk)
// =========================================================================
fn render_cyber_neon(time: f32) {
    let bands = 12;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = t * 340.0;
        let h = 340.0 / bands as f32 + 1.0;
        let r = 0.08 + t * 0.32;
        let g = 0.02 + t * 0.04;
        let b = 0.18 + t * 0.38;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    let sun_x = 400.0;
    let sun_y = 200.0;
    let sun_radius = 80.0;
    draw_circle(sun_x, sun_y, sun_radius + 20.0, Color::new(1.0, 0.15, 0.65, 0.15));
    draw_circle(sun_x, sun_y, sun_radius, Color::new(1.0, 0.25, 0.55, 0.95));

    for bar in 0..6 {
        let bar_y = sun_y + 12.0 + bar as f32 * 11.0;
        let bar_h = 4.0 + bar as f32 * 0.8;
        let dist_y = bar_y - sun_y;
        if dist_y < sun_radius {
            let half_w = (sun_radius * sun_radius - dist_y * dist_y).sqrt();
            draw_rectangle(sun_x - half_w, bar_y, half_w * 2.0, bar_h, Color::new(0.12, 0.04, 0.24, 1.0));
        }
    }

    let bldgs: [(f32, f32, f32, Color); 9] = [
        (20.0, 190.0, 50.0, Color::new(0.08, 0.08, 0.18, 0.95)),
        (65.0, 150.0, 65.0, Color::new(0.12, 0.09, 0.24, 0.95)),
        (125.0, 220.0, 55.0, Color::new(0.07, 0.07, 0.16, 0.95)),
        (175.0, 170.0, 70.0, Color::new(0.14, 0.1, 0.26, 0.95)),
        (550.0, 160.0, 65.0, Color::new(0.13, 0.09, 0.25, 0.95)),
        (610.0, 210.0, 55.0, Color::new(0.08, 0.07, 0.17, 0.95)),
        (660.0, 140.0, 75.0, Color::new(0.14, 0.11, 0.27, 0.95)),
        (730.0, 195.0, 60.0, Color::new(0.09, 0.08, 0.19, 0.95)),
        (370.0, 230.0, 60.0, Color::new(0.06, 0.05, 0.15, 0.95)),
    ];

    for (bx, by, bw, bcol) in bldgs {
        draw_rectangle(bx, by, bw, 340.0 - by, bcol);
        draw_rectangle_lines(bx, by, bw, 340.0 - by, 1.0, Color::new(0.3, 0.85, 1.0, 0.25));
    }

    let grid_top = 340.0;
    draw_rectangle(0.0, grid_top, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - grid_top, Color::new(0.05, 0.04, 0.12, 1.0));

    let horiz_lines = 10;
    for i in 1..=horiz_lines {
        let t = (i as f32 / horiz_lines as f32).powf(1.8);
        let y = grid_top + t * (VIRTUAL_HEIGHT - grid_top);
        let alpha = 0.25 + t * 0.5;
        draw_line(0.0, y, VIRTUAL_WIDTH, y, 1.5, Color::new(0.15, 0.85, 1.0, alpha));
    }

    let v_rays = 16;
    for i in 0..=v_rays {
        let bottom_x = i as f32 * (VIRTUAL_WIDTH / v_rays as f32);
        let vanish_x = 400.0 + (bottom_x - 400.0) * 0.2;
        draw_line(vanish_x, grid_top, bottom_x, VIRTUAL_HEIGHT, 1.2, Color::new(1.0, 0.15, 0.7, 0.35));
    }

    let pulse = (time * 4.0).sin().abs() * 0.3 + 0.7;
    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.5, Color::new(0.2, 0.95, 1.0, pulse));
}

// =========================================================================
// 3. CLASSIC BEACH HD (Nostalgic Sunny Sky & Crystal Ocean)
// =========================================================================
fn render_classic_remaster(time: f32) {
    let sky_h = 360.0;
    for i in 0..10 {
        let t = i as f32 / 10.0;
        let y = t * sky_h;
        let h = sky_h / 10.0 + 1.0;
        let r = 0.25 + t * 0.45;
        let g = 0.55 + t * 0.35;
        let b = 0.95 + t * 0.05;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    draw_circle(680.0, 85.0, 48.0, Color::new(1.0, 0.98, 0.8, 0.2));
    draw_circle(680.0, 85.0, 32.0, Color::new(1.0, 0.98, 0.85, 1.0));

    render_stylized_cloud(80.0 + (time * 3.5) % 900.0 - 50.0, 75.0, 1.1, 0.85);
    render_stylized_cloud(380.0 + (time * 2.8) % 900.0 - 50.0, 110.0, 1.3, 0.8);
    render_stylized_cloud(620.0 + (time * 3.2) % 900.0 - 50.0, 60.0, 0.85, 0.75);

    let ocean_y = 330.0;
    draw_rectangle(0.0, ocean_y, VIRTUAL_WIDTH, 90.0, Color::new(0.12, 0.55, 0.82, 1.0));

    for wave in 0..3 {
        let wy = ocean_y + wave as f32 * 18.0 + 8.0;
        let mut x = 0.0;
        while x <= VIRTUAL_WIDTH {
            let dy = ((x * 0.04 + time * 2.0).sin()) * 2.5;
            draw_circle(x, wy + dy, 3.0, Color::new(0.9, 0.96, 1.0, 0.6));
            x += 24.0;
        }
    }

    let sand_y = 420.0;
    draw_rectangle(0.0, sand_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - sand_y, Color::new(0.95, 0.84, 0.55, 1.0));
    draw_rectangle(0.0, sand_y, VIRTUAL_WIDTH, 14.0, Color::new(0.85, 0.72, 0.42, 0.45));

    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.0, Color::new(0.98, 0.98, 1.0, 0.9));
}

// =========================================================================
// 4. ALPINE PEAKS (Winter Snow Arena)
// =========================================================================
fn render_alpine_peaks(time: f32) {
    // Crisp icy alpine sky (pale cyan to cool blue)
    let sky_h = 360.0;
    for i in 0..12 {
        let t = i as f32 / 12.0;
        let y = t * sky_h;
        let h = sky_h / 12.0 + 1.0;
        let r = 0.35 + t * 0.45;
        let g = 0.58 + t * 0.32;
        let b = 0.88 + t * 0.12;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    // Distant mountain ranges with snow caps
    let peaks: [(f32, f32, f32); 5] = [
        (80.0, 160.0, 180.0),
        (260.0, 130.0, 220.0),
        (450.0, 150.0, 200.0),
        (620.0, 120.0, 240.0),
        (760.0, 170.0, 190.0),
    ];

    for (px, py, pw) in peaks {
        // Mountain body (slate slate/blue)
        draw_triangle(
            Vec2::new(px - pw * 0.5, 360.0),
            Vec2::new(px, py),
            Vec2::new(px + pw * 0.5, 360.0),
            Color::new(0.38, 0.46, 0.58, 1.0),
        );
        // Snow cap
        draw_triangle(
            Vec2::new(px - pw * 0.2, py + (360.0 - py) * 0.35),
            Vec2::new(px, py),
            Vec2::new(px + pw * 0.2, py + (360.0 - py) * 0.35),
            Color::new(0.95, 0.98, 1.0, 0.95),
        );
    }

    // Snowy pine trees along the foothills
    for i in 0..14 {
        let tx = 30.0 + i as f32 * 55.0;
        let ty = 340.0 + (i as f32 * 1.7).sin() * 12.0;
        draw_triangle(Vec2::new(tx - 14.0, ty + 30.0), Vec2::new(tx, ty), Vec2::new(tx + 14.0, ty + 30.0), Color::new(0.12, 0.32, 0.22, 1.0));
        draw_triangle(Vec2::new(tx - 10.0, ty + 16.0), Vec2::new(tx, ty - 6.0), Vec2::new(tx + 10.0, ty + 16.0), Color::new(0.16, 0.42, 0.28, 1.0));
        // Snow on tips
        draw_triangle(Vec2::new(tx - 5.0, ty + 4.0), Vec2::new(tx, ty - 6.0), Vec2::new(tx + 5.0, ty + 4.0), Color::new(0.92, 0.96, 1.0, 0.9));
    }

    // Frosted Snow Sand Court
    let snow_y = 415.0;
    draw_rectangle(0.0, snow_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - snow_y, Color::new(0.88, 0.94, 0.98, 1.0));
    draw_rectangle(0.0, snow_y, VIRTUAL_WIDTH, 15.0, Color::new(0.75, 0.85, 0.95, 0.5));

    // Court boundary lines (vibrant glacial blue)
    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.0, Color::new(0.25, 0.75, 1.0, 0.9));

    // Falling procedural snow flakes
    for flake in 0..28 {
        let seed = flake as f32 * 31.7;
        let fx = ((seed * 17.3 + time * 24.0) % (VIRTUAL_WIDTH + 60.0)) - 30.0;
        let fy = (seed * 29.1 + time * 45.0) % 520.0;
        let sway = (time * 2.0 + seed).sin() * 6.0;
        let size = 1.8 + (seed % 2.5);
        draw_circle(fx + sway, fy, size, Color::new(1.0, 1.0, 1.0, 0.75));
    }
}

// =========================================================================
// 5. MAGMA CALDERA (Volcanic Magma Arena)
// =========================================================================
fn render_magma_caldera(time: f32) {
    // Smoldering volcanic sky (dark ash smoke to deep blood-crimson)
    let sky_h = 360.0;
    for i in 0..12 {
        let t = i as f32 / 12.0;
        let y = t * sky_h;
        let h = sky_h / 12.0 + 1.0;
        let r = 0.18 + t * 0.65;
        let g = 0.04 + t * 0.18;
        let b = 0.05 + t * 0.05;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    // Jagged basalt peaks & smoking caldera rim
    let spires: [(f32, f32, f32); 6] = [
        (60.0, 180.0, 120.0),
        (200.0, 140.0, 160.0),
        (350.0, 190.0, 140.0),
        (480.0, 150.0, 170.0),
        (640.0, 170.0, 150.0),
        (750.0, 130.0, 160.0),
    ];

    for (sx, sy, sw) in spires {
        draw_triangle(
            Vec2::new(sx - sw * 0.5, 360.0),
            Vec2::new(sx, sy),
            Vec2::new(sx + sw * 0.5, 360.0),
            Color::new(0.14, 0.08, 0.09, 1.0),
        );
        // Magma fissure lines
        draw_line(sx, sy + 30.0, sx + 10.0, 360.0, 2.0, Color::new(1.0, 0.35, 0.05, 0.6));
    }

    // Molten lava pool under court
    let lava_y = 350.0;
    let pulse = (time * 3.5).sin().abs() * 0.2 + 0.8;
    draw_rectangle(0.0, lava_y, VIRTUAL_WIDTH, 70.0, Color::new(0.85 * pulse, 0.22 * pulse, 0.02, 1.0));

    // Lava crust islands
    for c in 0..8 {
        let cx = c as f32 * 105.0 + 10.0;
        let cy = lava_y + 12.0 + (c as f32 * 1.5).sin() * 8.0;
        draw_rectangle(cx, cy, 70.0, 16.0, Color::new(0.2, 0.1, 0.08, 0.95));
        draw_line(cx, cy, cx + 70.0, cy, 1.5, Color::new(1.0, 0.55, 0.1, 0.7));
    }

    // Dark Basalt Tile Court
    let court_y = 420.0;
    draw_rectangle(0.0, court_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - court_y, Color::new(0.16, 0.14, 0.16, 1.0));
    draw_rectangle(0.0, court_y, VIRTUAL_WIDTH, 14.0, Color::new(0.28, 0.18, 0.12, 0.7));

    // Court boundary lines (fiery glowing orange)
    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.5, Color::new(1.0, 0.45, 0.1, 0.95));

    // Rising burning embers
    for ember in 0..22 {
        let seed = ember as f32 * 23.9;
        let ex = ((seed * 19.3 + time * 18.0) % (VIRTUAL_WIDTH + 40.0)) - 20.0;
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
// 6. ORBITAL STATION (Deep Space Zero-G Deck)
// =========================================================================
fn render_orbital_station(time: f32) {
    // Deep cosmic space (pure black to interstellar purple)
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, 420.0, Color::new(0.02, 0.02, 0.06, 1.0));

    // Cosmic nebula cloud
    draw_circle(250.0, 150.0, 160.0, Color::new(0.45, 0.1, 0.65, 0.12));
    draw_circle(580.0, 180.0, 140.0, Color::new(0.1, 0.4, 0.75, 0.14));

    // Starfield with twinkling stars
    for s in 0..45 {
        let seed = s as f32 * 17.13;
        let sx = (seed * 83.1) % VIRTUAL_WIDTH;
        let sy = (seed * 47.7) % 320.0;
        let twinkle = ((time * 3.0 + seed).sin() * 0.4 + 0.6).clamp(0.1, 1.0);
        let sz = if s % 5 == 0 { 2.2 } else { 1.2 };
        draw_circle(sx, sy, sz, Color::new(0.9, 0.95, 1.0, twinkle));
    }

    // Curved Earth limb in the background
    let earth_cx = 400.0;
    let earth_cy = 980.0;
    let earth_r = 750.0;
    // Atmosphere glow
    draw_circle(earth_cx, earth_cy, earth_r + 28.0, Color::new(0.15, 0.6, 1.0, 0.15));
    draw_circle(earth_cx, earth_cy, earth_r + 12.0, Color::new(0.3, 0.8, 1.0, 0.35));
    // Earth surface
    draw_circle(earth_cx, earth_cy, earth_r, Color::new(0.08, 0.35, 0.65, 1.0));

    // High-tech station solar array & truss
    draw_rectangle(0.0, 310.0, VIRTUAL_WIDTH, 14.0, Color::new(0.2, 0.22, 0.3, 1.0));
    draw_rectangle_lines(0.0, 310.0, VIRTUAL_WIDTH, 14.0, 1.5, Color::new(0.4, 0.6, 0.9, 0.6));

    for panel in 0..12 {
        let px = panel as f32 * 68.0 + 10.0;
        draw_rectangle(px, 326.0, 56.0, 20.0, Color::new(0.08, 0.25, 0.55, 0.9));
        draw_rectangle_lines(px, 326.0, 56.0, 20.0, 1.0, Color::new(0.3, 0.7, 1.0, 0.5));
    }

    // Magnetic Hull Deck Court
    let deck_y = 415.0;
    draw_rectangle(0.0, deck_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - deck_y, Color::new(0.12, 0.14, 0.2, 1.0));
    draw_rectangle(0.0, deck_y, VIRTUAL_WIDTH, 16.0, Color::new(0.22, 0.25, 0.35, 0.8));

    // Deck grid panels
    for i in 0..10 {
        let x = i as f32 * 80.0;
        draw_line(x, deck_y, x, VIRTUAL_HEIGHT, 1.0, Color::new(0.25, 0.35, 0.5, 0.35));
    }

    // Glowing Neon Blue Laser Boundary
    let laser_pulse = (time * 5.0).sin().abs() * 0.25 + 0.75;
    draw_line(LEFT_PLANE, GROUND_PLANE_HEIGHT_MAX, RIGHT_PLANE, GROUND_PLANE_HEIGHT_MAX, 3.5, Color::new(0.1, 0.85, 1.0, laser_pulse));
}

// =========================================================================
// OPEN-SOURCE PERIMETER ADVERTISING BOARDS
// =========================================================================
fn render_perimeter_boards(time: f32, _arena: ArenaType) {
    let board_y = 398.0;
    let board_h = 24.0;
    let board_start_x = 35.0;
    let board_end_x = 765.0;
    let board_w = board_end_x - board_start_x;

    // 1. Dark metallic chassis & support stanchions
    draw_rectangle(board_start_x - 4.0, board_y - 2.0, board_w + 8.0, board_h + 4.0, Color::new(0.06, 0.07, 0.1, 0.95));
    draw_rectangle_lines(board_start_x - 4.0, board_y - 2.0, board_w + 8.0, board_h + 4.0, 1.5, Color::new(0.3, 0.35, 0.45, 0.7));

    // Top illuminated LED rail
    draw_line(board_start_x - 4.0, board_y - 2.0, board_end_x + 4.0, board_y - 2.0, 2.0, Color::new(0.6, 0.85, 1.0, 0.85));

    // 2. Open-Source distros & projects showcase roster
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

        // Skip off-screen panels
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
