// Blobby Volley 3 - 2026 Remastered 3D Volleyball Renderer with Motion Trails
use macroquad::prelude::*;
use glam::Vec2;
use crate::physics::Ball;
use crate::constants::*;

pub struct BallTrail {
    history: Vec<(Vec2, f32)>, // (position, speed)
}

impl BallTrail {
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(16),
        }
    }

    pub fn update(&mut self, pos: Vec2, speed: f32) {
        if speed > 6.0 {
            self.history.insert(0, (pos, speed));
            if self.history.len() > 10 {
                self.history.pop();
            }
        } else if !self.history.is_empty() {
            self.history.pop();
        }
    }

    pub fn render(&self) {
        let len = self.history.len();
        if len < 2 {
            return;
        }

        for i in 0..(len - 1) {
            let (p1, speed1) = self.history[i];
            let (p2, _speed2) = self.history[i + 1];

            let t = 1.0 - (i as f32 / len as f32);
            let alpha = t * 0.45;
            let width = (BALL_RADIUS * 1.5) * t;

            // Fiery trail for high-speed spikes, cyan for regular fast hits
            let trail_color = if speed1 > 12.0 {
                Color::new(1.0, 0.45, 0.1, alpha)
            } else {
                Color::new(0.2, 0.7, 1.0, alpha)
            };

            draw_line(p1.x, p1.y, p2.x, p2.y, width, trail_color);
        }
    }
}

pub fn render_ball(ball: &Ball, trail: &BallTrail) {
    // 1. Render motion trail behind ball
    trail.render();

    // 2. Render dynamic projected ground shadow
    let ground_y = GROUND_PLANE_HEIGHT_MAX - 2.0;
    let height_above_ground = (ground_y - ball.pos.y).max(0.0);
    let shadow_scale = (1.0 - (height_above_ground / 450.0)).clamp(0.2, 1.0);
    let shadow_alpha = (0.4 * shadow_scale).clamp(0.05, 0.4);
    let shadow_radius_x = BALL_RADIUS * shadow_scale * 1.1;
    let shadow_radius_y = BALL_RADIUS * shadow_scale * 0.35;

    draw_ellipse(
        ball.pos.x,
        ground_y,
        shadow_radius_x,
        shadow_radius_y,
        0.0,
        Color::new(0.0, 0.0, 0.0, shadow_alpha),
    );

    // 3. Render 3D shaded volleyball with curved rotating panels
    let cx = ball.pos.x;
    let cy = ball.pos.y;
    let r = BALL_RADIUS;

    // Drop shadow under ball edge
    draw_circle(cx + 1.0, cy + 2.0, r + 0.5, Color::new(0.0, 0.0, 0.0, 0.25));

    // Base sphere (white leather panel)
    draw_circle(cx, cy, r, Color::new(0.96, 0.96, 0.98, 1.0));

    // Beach volleyball colored panels (classic blue & yellow curves)
    let rot = ball.rotation;
    let num_panels = 3;
    for i in 0..num_panels {
        let base_angle = rot + (i as f32 / num_panels as f32) * std::f32::consts::TAU;

        // Draw curved panel ribbons
        let panel_color = match i {
            0 => Color::new(0.08, 0.42, 0.88, 0.85), // Vibrant Royal Blue
            1 => Color::new(0.98, 0.82, 0.12, 0.85), // Sunburst Yellow
            _ => Color::new(0.95, 0.95, 0.95, 0.85), // Crisp White
        };

        let p_start = Vec2::new(cx + base_angle.cos() * r * 0.95, cy + base_angle.sin() * r * 0.95);
        let p_mid = Vec2::new(
            cx + (base_angle + 0.6).cos() * r * 0.45,
            cy + (base_angle + 0.6).sin() * r * 0.45,
        );
        let p_end = Vec2::new(
            cx + (base_angle + 1.2).cos() * r * 0.95,
            cy + (base_angle + 1.2).sin() * r * 0.95,
        );

        draw_triangle(p_mid, p_start, p_end, panel_color);
    }

    // Panel seam lines
    for i in 0..6 {
        let seam_angle = rot + (i as f32 / 6.0) * std::f32::consts::TAU;
        let sx = cx + seam_angle.cos() * r;
        let sy = cy + seam_angle.sin() * r;
        draw_line(cx, cy, sx, sy, 1.5, Color::new(0.2, 0.25, 0.35, 0.4));
    }

    // 4. Volumetric sphere 3D lighting overlay
    // Shading on bottom-right
    let shadow_center = Vec2::new(cx + r * 0.3, cy + r * 0.3);
    draw_circle(
        shadow_center.x,
        shadow_center.y,
        r * 0.88,
        Color::new(0.05, 0.08, 0.18, 0.22),
    );

    // Specular glossy glint on upper-left
    let spec_pos = Vec2::new(cx - r * 0.32, cy - r * 0.32);
    draw_circle(spec_pos.x, spec_pos.y, r * 0.38, Color::new(1.0, 1.0, 1.0, 0.65));
    draw_circle(spec_pos.x - 2.0, spec_pos.y - 2.0, r * 0.18, Color::new(1.0, 1.0, 1.0, 0.85));

    // Outer anti-aliased rim ring
    draw_circle_lines(cx, cy, r, 2.0, Color::new(0.15, 0.2, 0.3, 0.6));
}
