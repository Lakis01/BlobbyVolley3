// Blobby Volley 3 - 2026 Remastered Procedural Jelly Blobby Renderer
use macroquad::prelude::*;
use glam::Vec2;
use crate::physics::{Blobby, PlayerSide};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobTheme {
    ElectricCyan,
    SunsetOrange,
    NeonMagenta,
    AcidLime,
    RoyalPurple,
    CyberYellow,
    IceWhite,
}

impl BlobTheme {
    pub fn name(self) -> &'static str {
        match self {
            BlobTheme::ElectricCyan => "Electric Cyan",
            BlobTheme::SunsetOrange => "Sunset Orange",
            BlobTheme::NeonMagenta => "Neon Magenta",
            BlobTheme::AcidLime => "Acid Lime",
            BlobTheme::RoyalPurple => "Royal Purple",
            BlobTheme::CyberYellow => "Cyber Yellow",
            BlobTheme::IceWhite => "Ice White",
        }
    }

    pub fn colors(self) -> (Color, Color, Color) {
        // (Base / Core color, Dark rim / shade, Bright specular highlight)
        match self {
            BlobTheme::ElectricCyan => (
                Color::new(0.05, 0.78, 0.95, 1.0),
                Color::new(0.02, 0.42, 0.65, 1.0),
                Color::new(0.65, 0.95, 1.0, 0.9),
            ),
            BlobTheme::SunsetOrange => (
                Color::new(1.0, 0.45, 0.1, 1.0),
                Color::new(0.75, 0.18, 0.02, 1.0),
                Color::new(1.0, 0.85, 0.5, 0.9),
            ),
            BlobTheme::NeonMagenta => (
                Color::new(0.95, 0.15, 0.65, 1.0),
                Color::new(0.55, 0.05, 0.38, 1.0),
                Color::new(1.0, 0.7, 0.9, 0.9),
            ),
            BlobTheme::AcidLime => (
                Color::new(0.42, 0.92, 0.12, 1.0),
                Color::new(0.2, 0.55, 0.05, 1.0),
                Color::new(0.85, 1.0, 0.6, 0.9),
            ),
            BlobTheme::RoyalPurple => (
                Color::new(0.55, 0.22, 0.95, 1.0),
                Color::new(0.3, 0.08, 0.6, 1.0),
                Color::new(0.85, 0.7, 1.0, 0.9),
            ),
            BlobTheme::CyberYellow => (
                Color::new(1.0, 0.82, 0.05, 1.0),
                Color::new(0.8, 0.5, 0.02, 1.0),
                Color::new(1.0, 0.98, 0.7, 0.9),
            ),
            BlobTheme::IceWhite => (
                Color::new(0.88, 0.94, 1.0, 1.0),
                Color::new(0.5, 0.65, 0.82, 1.0),
                Color::new(1.0, 1.0, 1.0, 0.95),
            ),
        }
    }

    pub fn next(self) -> Self {
        match self {
            BlobTheme::ElectricCyan => BlobTheme::SunsetOrange,
            BlobTheme::SunsetOrange => BlobTheme::NeonMagenta,
            BlobTheme::NeonMagenta => BlobTheme::AcidLime,
            BlobTheme::AcidLime => BlobTheme::RoyalPurple,
            BlobTheme::RoyalPurple => BlobTheme::CyberYellow,
            BlobTheme::CyberYellow => BlobTheme::IceWhite,
            BlobTheme::IceWhite => BlobTheme::ElectricCyan,
        }
    }
}

pub fn render_blobby(
    blob: &Blobby,
    side: PlayerSide,
    theme: BlobTheme,
    ball_pos: Vec2,
    time: f32,
) {
    let base_x = blob.pos.x;
    let base_y = blob.pos.y;

    // Contact shadow on the sand
    let dist_from_ground = (GROUND_PLANE_HEIGHT - base_y).max(0.0);
    let shadow_scale = (1.0 - (dist_from_ground / 220.0)).clamp(0.25, 1.0);
    let shadow_alpha = (0.45 * shadow_scale).clamp(0.08, 0.45);
    let shadow_width = (BLOBBY_LOWER_RADIUS + 12.0) * shadow_scale;
    let shadow_height = 10.0 * shadow_scale;

    draw_ellipse(
        base_x,
        GROUND_PLANE_HEIGHT_MAX - 4.0,
        shadow_width,
        shadow_height,
        0.0,
        Color::new(0.0, 0.0, 0.0, shadow_alpha),
    );

    // Volume-preserving squash & stretch
    let sy = blob.squash_factor;
    let sx = 1.0 / (sy.max(0.2).sqrt());

    let (core_color, shade_color, highlight_color) = theme.colors();

    // Generate smooth procedural contour
    // Centers of the two circles in local space (origin at blob.pos)
    let top_y = -BLOBBY_UPPER_SPHERE * sy;
    let top_r = BLOBBY_UPPER_RADIUS * sx;

    let bot_y = BLOBBY_LOWER_SPHERE * sy;
    let bot_r = BLOBBY_LOWER_RADIUS * sx;

    // Outer shade layer for depth / 3D volumetric feel
    let num_points = 36;
    let mut contour = Vec::with_capacity(num_points);

    for i in 0..num_points {
        let angle = (i as f32 / num_points as f32) * std::f32::consts::TAU;
        let sin = angle.sin();
        let cos = angle.cos();

        // Blend between top dome and bottom body
        let (cy, cr) = if sin < 0.0 {
            // Upper half
            (top_y, top_r)
        } else {
            // Lower half
            (bot_y, bot_r)
        };

        // Smooth contour interpolation
        let px = base_x + cos * cr;
        let py = base_y + cy + sin * cr;
        contour.push(Vec2::new(px, py));
    }

    // Draw solid body with inner ambient occlusion
    draw_blobby_mesh(&contour, base_x, base_y + (top_y + bot_y) * 0.5, shade_color, core_color);

    // Inner translucent glow (smaller contour)
    let inner_points: Vec<Vec2> = contour
        .iter()
        .map(|p| {
            let center = Vec2::new(base_x, base_y + (top_y + bot_y) * 0.5);
            center + (*p - center) * 0.82
        })
        .collect();
    draw_blobby_mesh(
        &inner_points,
        base_x,
        base_y + (top_y + bot_y) * 0.5,
        core_color,
        Color::new(core_color.r * 1.15, core_color.g * 1.15, core_color.b * 1.15, 0.85),
    );

    // Specular curved glass gloss highlight (top-left shoulder)
    let spec_center = Vec2::new(
        base_x - top_r * 0.32,
        base_y + top_y - top_r * 0.28,
    );
    draw_ellipse(
        spec_center.x,
        spec_center.y,
        top_r * 0.48,
        top_r * 0.28,
        -0.45,
        highlight_color,
    );

    // Secondary rim highlight on the lower right flank
    let rim_center = Vec2::new(
        base_x + bot_r * 0.42,
        base_y + bot_y + bot_r * 0.25,
    );
    draw_ellipse(
        rim_center.x,
        rim_center.y,
        bot_r * 0.35,
        bot_r * 0.16,
        0.3,
        Color::new(highlight_color.r, highlight_color.g, highlight_color.b, 0.4),
    );

    // Render Animated Expressive Eyes
    render_blobby_eyes(
        base_x,
        base_y + top_y * 0.92,
        top_r,
        side,
        ball_pos,
        time,
    );
}

/// Renders a fan mesh representing the blobby jelly body with center gradient
fn draw_blobby_mesh(
    points: &[Vec2],
    center_x: f32,
    center_y: f32,
    edge_color: Color,
    center_color: Color,
) {
    let center = Vec2::new(center_x, center_y);
    let len = points.len();
    if len < 3 {
        return;
    }

    for i in 0..len {
        let p1 = points[i];
        let p2 = points[(i + 1) % len];

        draw_triangle(
            center,
            p1,
            p2,
            center_color,
        );
        // Draw anti-aliased edge ribbon
        draw_line(p1.x, p1.y, p2.x, p2.y, 2.0, edge_color);
    }
}

/// Renders large expressive cartoon eyes with real-time ball tracking and blinking
fn render_blobby_eyes(
    cx: f32,
    cy: f32,
    top_r: f32,
    side: PlayerSide,
    ball_pos: Vec2,
    time: f32,
) {
    // Face direction: left player faces right, right player faces left
    let facing_dir = match side {
        PlayerSide::Left => 1.0,
        PlayerSide::Right => -1.0,
    };

    // Eye spacing and radius
    let eye_spacing = top_r * 0.38;
    let eye_rad = top_r * 0.30;
    let eye_center_x = cx + facing_dir * (top_r * 0.18);
    let eye_y = cy - 2.0;

    let left_eye = Vec2::new(eye_center_x - eye_spacing * 0.5, eye_y);
    let right_eye = Vec2::new(eye_center_x + eye_spacing * 0.5, eye_y);

    // Blinking logic: quick natural blink every ~3.5 seconds
    let blink_cycle = (time * 1.8 + cx * 0.01) % 4.0;
    let is_blinking = blink_cycle < 0.14;

    for eye_pos in [left_eye, right_eye] {
        if is_blinking {
            // Eyelid slit during blink
            draw_line(
                eye_pos.x - eye_rad * 0.9,
                eye_pos.y,
                eye_pos.x + eye_rad * 0.9,
                eye_pos.y,
                2.5,
                Color::new(0.1, 0.1, 0.15, 0.8),
            );
        } else {
            // Eye socket shadow
            draw_circle(eye_pos.x, eye_pos.y + 1.0, eye_rad + 1.0, Color::new(0.0, 0.0, 0.0, 0.25));

            // Eye white (sclera)
            draw_circle(eye_pos.x, eye_pos.y, eye_rad, Color::new(0.98, 0.98, 1.0, 1.0));

            // Calculate pupil look-at direction vector towards volleyball
            let look_diff = ball_pos - eye_pos;
            let look_dir = if look_diff.length_squared() > 1e-4 {
                look_diff.normalize()
            } else {
                Vec2::new(facing_dir, 0.0)
            };

            // Limit pupil movement within eye white
            let pupil_offset = look_dir * (eye_rad * 0.42);
            let pupil_pos = eye_pos + pupil_offset;
            let pupil_rad = eye_rad * 0.48;

            // Pupil
            draw_circle(pupil_pos.x, pupil_pos.y, pupil_rad, Color::new(0.08, 0.1, 0.14, 1.0));

            // Cute catchlight / specular gleam dot
            let gleam_offset = Vec2::new(-pupil_rad * 0.35, -pupil_rad * 0.35);
            draw_circle(
                pupil_pos.x + gleam_offset.x,
                pupil_pos.y + gleam_offset.y,
                pupil_rad * 0.35,
                Color::new(1.0, 1.0, 1.0, 0.9),
            );
        }
    }
}
