// Blobby Volley 3 - 2026 Remastered Beach Volleyball Net with Physics Reaction
use macroquad::prelude::*;
use crate::constants::*;

pub struct NetRenderer {
    pub wobble: f32,
    pub wobble_vel: f32,
}

impl NetRenderer {
    pub fn new() -> Self {
        Self {
            wobble: 0.0,
            wobble_vel: 0.0,
        }
    }

    pub fn trigger_hit(&mut self, intensity: f32) {
        self.wobble_vel += intensity * 8.0;
    }

    pub fn update(&mut self, dt: f32) {
        // Damped harmonic oscillation for net wobble
        let spring_k = 60.0;
        let damping = 6.5;
        let force = -spring_k * self.wobble - damping * self.wobble_vel;
        self.wobble_vel += force * dt;
        self.wobble += self.wobble_vel * dt;
    }

    pub fn render(&self, time: f32, net_x: f32) {
        let x = net_x;
        let top_y = NET_SPHERE_POSITION;
        let bot_y = GROUND_PLANE_HEIGHT_MAX;
        let rad = NET_RADIUS;

        let wobble_offset = self.wobble * (time * 12.0).sin();

        // 1. Post shadow on ground
        draw_ellipse(
            x,
            bot_y - 2.0,
            rad * 2.2,
            rad * 0.7,
            0.0,
            Color::new(0.0, 0.0, 0.0, 0.35),
        );

        // 2. Main post cylinder
        let post_left = x - rad;
        let _post_right = x + rad;
        let post_w = rad * 2.0;
        let post_h = bot_y - top_y;

        // Base post: metallic gradient
        draw_rectangle(
            post_left,
            top_y,
            post_w,
            post_h,
            Color::new(0.85, 0.88, 0.92, 1.0),
        );
        // Post shadow side
        draw_rectangle(
            post_left + rad * 0.7,
            top_y,
            rad * 1.3,
            post_h,
            Color::new(0.55, 0.58, 0.65, 0.5),
        );
        // Post specular shine stripe
        draw_rectangle(
            post_left + rad * 0.25,
            top_y,
            rad * 0.4,
            post_h,
            Color::new(1.0, 1.0, 1.0, 0.7),
        );

        // 3. Woven net grid pattern
        let grid_size = 14.0;
        let start_y = top_y + 12.0;
        let mut curr_y = start_y;
        while curr_y < bot_y - 8.0 {
            let offset = wobble_offset * ((curr_y - start_y) / (bot_y - start_y));
            draw_line(
                x - rad * 1.8 + offset,
                curr_y,
                x + rad * 1.8 + offset,
                curr_y,
                1.5,
                Color::new(0.9, 0.9, 0.95, 0.65),
            );
            curr_y += grid_size;
        }

        // 4. Top rounded sphere cap
        draw_circle(
            x + wobble_offset * 0.3,
            top_y,
            rad,
            Color::new(0.95, 0.96, 0.98, 1.0),
        );
        // Top cap shine
        draw_circle(
            x - rad * 0.25 + wobble_offset * 0.3,
            top_y - rad * 0.25,
            rad * 0.45,
            Color::new(1.0, 1.0, 1.0, 0.85),
        );

        // 5. High-visibility top band (white canvas tape with red safety edge)
        let band_h = 10.0;
        draw_rectangle(
            post_left - 3.0 + wobble_offset,
            top_y,
            post_w + 6.0,
            band_h,
            Color::new(0.98, 0.98, 1.0, 1.0),
        );
        draw_rectangle(
            post_left - 3.0 + wobble_offset,
            top_y,
            post_w + 6.0,
            2.0,
            Color::new(0.9, 0.15, 0.15, 0.9), // Red antenna line
        );
    }
}
