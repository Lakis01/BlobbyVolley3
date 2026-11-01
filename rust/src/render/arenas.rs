// Blobby Volley 3 - 2026 Remastered Arenas
use macroquad::prelude::*;
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaType {
    SunsetBeach,
    CyberNeon,
    ClassicRemaster,
}

impl ArenaType {
    pub fn name(self) -> &'static str {
        match self {
            ArenaType::SunsetBeach => "Copacabana Sunset",
            ArenaType::CyberNeon => "Neo Cyber Stadium",
            ArenaType::ClassicRemaster => "Classic Beach HD",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ArenaType::SunsetBeach => ArenaType::CyberNeon,
            ArenaType::CyberNeon => ArenaType::ClassicRemaster,
            ArenaType::ClassicRemaster => ArenaType::SunsetBeach,
        }
    }
}

pub fn render_arena(arena: ArenaType, time: f32) {
    match arena {
        ArenaType::SunsetBeach => render_sunset_beach(time),
        ArenaType::CyberNeon => render_cyber_neon(time),
        ArenaType::ClassicRemaster => render_classic_remaster(time),
    }
}

// =========================================================================
// 1. COPACABANA SUNSET BEACH (Modern Tropical Sunset)
// =========================================================================
fn render_sunset_beach(time: f32) {
    // 1. Sky Gradient (Deep violet -> crimson -> warm gold)
    let sky_h = 420.0;
    let bands = 14;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let y = t * sky_h;
        let h = sky_h / bands as f32 + 1.0;

        // Color interpolation: top deep violet to bottom gold
        let r = 0.22 + t * 0.76;
        let g = 0.08 + t * 0.45;
        let b = 0.45 - t * 0.35;
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, Color::new(r, g, b, 1.0));
    }

    // 2. Radiant Setting Sun with Glow Halo
    let sun_x = 400.0;
    let sun_y = 230.0;
    // Outer atmospheric glow
    draw_circle(sun_x, sun_y, 95.0, Color::new(1.0, 0.75, 0.2, 0.12));
    draw_circle(sun_x, sun_y, 65.0, Color::new(1.0, 0.85, 0.3, 0.25));
    // Sun core
    draw_circle(sun_x, sun_y, 42.0, Color::new(1.0, 0.96, 0.75, 0.95));

    // 3. Drifting Soft Clouds
    render_stylized_cloud(120.0 + (time * 6.0) % 900.0 - 50.0, 90.0, 1.2, 0.35);
    render_stylized_cloud(520.0 + (time * 4.5) % 900.0 - 50.0, 130.0, 0.9, 0.28);
    render_stylized_cloud(280.0 + (time * 5.2) % 900.0 - 50.0, 70.0, 0.7, 0.22);

    // 4. Distant Mountain Silhouettes
    let mut mountain_points = Vec::new();
    mountain_points.push(Vec2::new(0.0, 360.0));
    mountain_points.push(Vec2::new(140.0, 310.0));
    mountain_points.push(Vec2::new(260.0, 345.0));
    mountain_points.push(Vec2::new(380.0, 325.0));
    mountain_points.push(Vec2::new(510.0, 350.0));
    mountain_points.push(Vec2::new(650.0, 305.0));
    mountain_points.push(Vec2::new(800.0, 360.0));
    mountain_points.push(Vec2::new(800.0, 380.0));
    mountain_points.push(Vec2::new(0.0, 380.0));

    for i in 1..(mountain_points.len() - 2) {
        draw_triangle(
            mountain_points[0],
            mountain_points[i],
            mountain_points[i + 1],
            Color::new(0.32, 0.14, 0.38, 0.75),
        );
    }

    // 5. Ocean Waves with Shimmering Foam & Sun Reflection
    let ocean_top = 360.0;
    let ocean_h = 70.0;
    draw_rectangle(0.0, ocean_top, VIRTUAL_WIDTH, ocean_h, Color::new(0.12, 0.28, 0.45, 1.0));

    // Sunlight vertical reflection shimmer on water
    let shimmer_w = 120.0;
    draw_rectangle(
        sun_x - shimmer_w * 0.5,
        ocean_top,
        shimmer_w,
        ocean_h,
        Color::new(1.0, 0.65, 0.2, 0.28),
    );

    // Animated rolling waves
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

    // 6. Warm Golden Sand Beach Court
    let sand_top = GROUND_PLANE_HEIGHT_MAX - 70.0;
    let sand_h = VIRTUAL_HEIGHT - sand_top;
    draw_rectangle(0.0, sand_top, VIRTUAL_WIDTH, sand_h, Color::new(0.92, 0.76, 0.48, 1.0));

    // Sand depth gradient
    draw_rectangle(
        0.0,
        sand_top,
        VIRTUAL_WIDTH,
        18.0,
        Color::new(0.82, 0.64, 0.38, 0.5),
    );

    // Court boundary lines (white beach straps with stakes)
    draw_line(
        LEFT_PLANE + 40.0,
        GROUND_PLANE_HEIGHT_MAX - 2.0,
        RIGHT_PLANE - 40.0,
        GROUND_PLANE_HEIGHT_MAX - 2.0,
        3.0,
        Color::new(0.98, 0.96, 0.92, 0.85),
    );

    // 7. Swaying Tropical Palm Trees on court flanks
    let sway_left = (time * 1.4).sin() * 5.0;
    let sway_right = ((time + 1.0) * 1.3).sin() * 5.0;

    render_palm_tree(45.0, 440.0, 1.15, sway_left, true);
    render_palm_tree(755.0, 440.0, 1.15, sway_right, false);

    // Ground boundary ground plane line
    draw_line(
        0.0,
        GROUND_PLANE_HEIGHT_MAX,
        VIRTUAL_WIDTH,
        GROUND_PLANE_HEIGHT_MAX,
        2.0,
        Color::new(0.78, 0.62, 0.36, 1.0),
    );
}

// =========================================================================
// 2. NEO CYBER STADIUM (Synthwave / Cyberpunk Arena)
// =========================================================================
fn render_cyber_neon(time: f32) {
    // 1. Dark synthwave backdrop
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.06, 0.04, 0.12, 1.0));

    // 2. Wireframe Sunset Sun
    let sun_x = 400.0;
    let sun_y = 200.0;
    draw_circle(sun_x, sun_y, 75.0, Color::new(1.0, 0.15, 0.6, 0.85));
    // Blinds / horizontal cutout lines on synthwave sun
    for i in 0..7 {
        let line_y = sun_y + i as f32 * 11.0;
        let line_w = (75.0_f32.powi(2) - (line_y - sun_y).powi(2)).max(0.0).sqrt() * 2.0;
        draw_rectangle(
            sun_x - line_w * 0.5,
            line_y,
            line_w,
            3.0,
            Color::new(0.06, 0.04, 0.12, 1.0),
        );
    }

    // 3. Cyberpunk Skyline Silhouettes
    for i in 0..10 {
        let bx = i as f32 * 85.0 - 20.0;
        let bw = 70.0;
        let bh = 140.0 + ((i * 37) % 110) as f32;
        let by = 380.0 - bh;
        draw_rectangle(bx, by, bw, bh, Color::new(0.12, 0.08, 0.22, 0.95));
        // Neon rooftop edge
        draw_line(bx, by, bx + bw, by, 2.0, Color::new(0.0, 0.9, 1.0, 0.6));
    }

    // 4. Perspective Neon Grid Floor
    let floor_y = 380.0;
    let floor_h = VIRTUAL_HEIGHT - floor_y;
    draw_rectangle(0.0, floor_y, VIRTUAL_WIDTH, floor_h, Color::new(0.04, 0.02, 0.09, 1.0));

    // Horizontal grid lines
    let grid_lines = 10;
    for i in 0..grid_lines {
        let t = (i as f32 / grid_lines as f32).powf(1.6);
        let y = floor_y + t * floor_h;
        let alpha = 0.2 + t * 0.65;
        draw_line(0.0, y, VIRTUAL_WIDTH, y, 1.5, Color::new(0.85, 0.1, 0.7, alpha));
    }

    // Vanishing perspective grid lines converging towards center horizon
    for i in 0..16 {
        let bottom_x = i as f32 * 60.0 - 50.0;
        let top_x = 400.0 + (bottom_x - 400.0) * 0.15;
        draw_line(
            top_x,
            floor_y,
            bottom_x,
            VIRTUAL_HEIGHT,
            1.5,
            Color::new(0.0, 0.85, 1.0, 0.45),
        );
    }

    // Stadium laser beams pulsating to time
    let beam_pulse = (time * 3.0).sin().abs() * 0.35 + 0.15;
    draw_line(
        400.0,
        NET_SPHERE_POSITION,
        400.0,
        GROUND_PLANE_HEIGHT_MAX,
        4.0,
        Color::new(0.0, 1.0, 0.9, beam_pulse + 0.4),
    );

    // Main baseline
    draw_line(
        LEFT_PLANE,
        GROUND_PLANE_HEIGHT_MAX,
        RIGHT_PLANE,
        GROUND_PLANE_HEIGHT_MAX,
        3.0,
        Color::new(0.0, 0.95, 1.0, 0.9),
    );
}

// =========================================================================
// 3. CLASSIC BEACH HD (Nostalgic 2026 Remaster)
// =========================================================================
fn render_classic_remaster(time: f32) {
    // 1. Crisp Blue Sky Gradient
    let sky_h = 370.0;
    for i in 0..12 {
        let t = i as f32 / 12.0;
        let y = t * sky_h;
        let h = sky_h / 12.0 + 1.0;
        let c = Color::new(0.18 + t * 0.35, 0.55 + t * 0.35, 0.95, 1.0);
        draw_rectangle(0.0, y, VIRTUAL_WIDTH, h, c);
    }

    // 2. Bright midday sun
    draw_circle(680.0, 90.0, 45.0, Color::new(1.0, 0.98, 0.7, 0.95));
    draw_circle(680.0, 90.0, 70.0, Color::new(1.0, 0.95, 0.5, 0.2));

    // 3. Clouds
    render_stylized_cloud(160.0 + (time * 8.0) % 900.0 - 50.0, 110.0, 1.3, 0.8);
    render_stylized_cloud(460.0 + (time * 6.5) % 900.0 - 50.0, 80.0, 1.0, 0.75);

    // 4. Sparkling Blue Ocean
    let ocean_y = 370.0;
    draw_rectangle(0.0, ocean_y, VIRTUAL_WIDTH, 60.0, Color::new(0.1, 0.45, 0.75, 1.0));

    // Wave crests
    for wave in 0..3 {
        let wy = ocean_y + wave as f32 * 18.0 + 8.0;
        let mut x = 0.0;
        while x <= VIRTUAL_WIDTH {
            let dy = ((x * 0.04 + time * 2.0).sin()) * 2.5;
            draw_circle(x, wy + dy, 3.0, Color::new(0.9, 0.96, 1.0, 0.6));
            x += 24.0;
        }
    }

    // 5. Sand Court
    let sand_y = 430.0;
    draw_rectangle(0.0, sand_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - sand_y, Color::new(0.92, 0.82, 0.52, 1.0));

    // Court boundary lines
    draw_line(
        LEFT_PLANE,
        GROUND_PLANE_HEIGHT_MAX,
        RIGHT_PLANE,
        GROUND_PLANE_HEIGHT_MAX,
        3.0,
        Color::new(0.98, 0.98, 1.0, 0.9),
    );
}

// Helper: Stylized soft puffy cloud
fn render_stylized_cloud(x: f32, y: f32, scale: f32, alpha: f32) {
    let cloud_color = Color::new(1.0, 1.0, 1.0, alpha);
    draw_circle(x, y, 22.0 * scale, cloud_color);
    draw_circle(x + 20.0 * scale, y - 6.0 * scale, 28.0 * scale, cloud_color);
    draw_circle(x + 44.0 * scale, y, 24.0 * scale, cloud_color);
    draw_circle(x + 22.0 * scale, y + 8.0 * scale, 20.0 * scale, cloud_color);
}

// Helper: Tropical palm tree
fn render_palm_tree(trunk_x: f32, ground_y: f32, scale: f32, sway: f32, tilt_right: bool) {
    let top_x = trunk_x + if tilt_right { 35.0 } else { -35.0 } + sway;
    let top_y = ground_y - 180.0 * scale;

    // Curved brown trunk segments
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

    // Lush green fronds radiating from top
    let frond_angles: [f32; 7] = [-1.4, -0.9, -0.4, 0.1, 0.6, 1.1, 1.6];
    for angle in frond_angles {
        let frond_len = 65.0 * scale;
        let end_x = top_x + angle.cos() * frond_len + sway * 0.8;
        let end_y = top_y + angle.sin() * frond_len + 12.0;

        draw_line(top_x, top_y, end_x, end_y, 4.0 * scale, Color::new(0.18, 0.58, 0.22, 0.95));
        // Leaf fringe
        draw_line(top_x, top_y - 2.0, end_x, end_y, 2.0 * scale, Color::new(0.35, 0.78, 0.32, 0.85));
    }
}
