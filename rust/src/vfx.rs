// Blobby Volley 3 - 2026 Remastered Particle Systems, Screen Shake & Visual Juice
use macroquad::prelude::*;
use glam::Vec2;
use quad_rand::gen_range;

#[derive(Debug, Clone)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub size: f32,
    pub life: f32,
    pub max_life: f32,
    pub is_confetti: bool,
    pub rotation: f32,
    pub rot_speed: f32,
}

#[derive(Debug, Clone)]
pub struct Shockwave {
    pub pos: Vec2,
    pub radius: f32,
    pub max_radius: f32,
    pub color: Color,
    pub life: f32,
    pub max_life: f32,
    pub width: f32,
}

pub struct VfxManager {
    pub particles: Vec<Particle>,
    pub shockwaves: Vec<Shockwave>,
    pub screen_shake: f32,
}

impl VfxManager {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(256),
            shockwaves: Vec::with_capacity(16),
            screen_shake: 0.0,
        }
    }

    pub fn trigger_screen_shake(&mut self, intensity: f32) {
        self.screen_shake = (self.screen_shake + intensity).min(10.0);
    }

    pub fn spawn_sand_puff(&mut self, pos: Vec2, count: usize) {
        for _ in 0..count {
            let vx = gen_range(-3.5, 3.5);
            let vy = gen_range(-4.0, -0.5);
            let size = gen_range(3.0, 6.5);
            let life = gen_range(0.3, 0.65);
            let alpha = gen_range(0.4, 0.8);

            self.particles.push(Particle {
                pos: pos + Vec2::new(gen_range(-12.0, 12.0), gen_range(-2.0, 2.0)),
                vel: Vec2::new(vx, vy),
                color: Color::new(0.92, 0.82, 0.58, alpha),
                size,
                life,
                max_life: life,
                is_confetti: false,
                rotation: 0.0,
                rot_speed: 0.0,
            });
        }
    }

    pub fn spawn_hit_shockwave(&mut self, pos: Vec2, intensity: f32) {
        // Expanding shockwave halo
        let max_r = 35.0 + intensity * 45.0;
        let color = if intensity > 0.7 {
            Color::new(1.0, 0.6, 0.2, 0.85) // Fiery smash
        } else {
            Color::new(0.4, 0.85, 1.0, 0.8) // Crisp hit
        };

        self.shockwaves.push(Shockwave {
            pos,
            radius: 5.0,
            max_radius: max_r,
            color,
            life: 0.25,
            max_life: 0.25,
            width: 4.0 + intensity * 3.0,
        });

        // Sparks & speed bursts
        let spark_count = (8.0 + intensity * 16.0) as usize;
        for _ in 0..spark_count {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(3.0, 10.0 + intensity * 8.0);
            let life = gen_range(0.2, 0.45);

            self.particles.push(Particle {
                pos,
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                color: Color::new(1.0, 0.95, 0.4, 0.95),
                size: gen_range(2.0, 4.5),
                life,
                max_life: life,
                is_confetti: false,
                rotation: 0.0,
                rot_speed: 0.0,
            });
        }

        if intensity > 0.5 {
            self.trigger_screen_shake(intensity * 4.0);
        }
    }

    pub fn spawn_confetti_celebration(&mut self, center: Vec2, count: usize) {
        for _ in 0..count {
            let angle = gen_range(0.0, std::f32::consts::TAU);
            let speed = gen_range(4.0, 14.0);
            let life = gen_range(1.2, 2.8);

            let r = gen_range(0.2, 1.0);
            let g = gen_range(0.2, 1.0);
            let b = gen_range(0.2, 1.0);

            self.particles.push(Particle {
                pos: center + Vec2::new(gen_range(-40.0, 40.0), gen_range(-10.0, 10.0)),
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed - 6.0),
                color: Color::new(r, g, b, 1.0),
                size: gen_range(4.0, 8.0),
                life,
                max_life: life,
                is_confetti: true,
                rotation: gen_range(0.0, std::f32::consts::TAU),
                rot_speed: gen_range(-8.0, 8.0),
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        // Screen shake decay
        self.screen_shake *= (-10.0 * dt).exp();
        if self.screen_shake < 0.05 {
            self.screen_shake = 0.0;
        }

        // Update shockwaves
        for sw in &mut self.shockwaves {
            sw.life -= dt;
            let progress = 1.0 - (sw.life / sw.max_life).max(0.0);
            sw.radius = 5.0 + (sw.max_radius - 5.0) * progress;
        }
        self.shockwaves.retain(|sw| sw.life > 0.0);

        // Update particles
        for p in &mut self.particles {
            p.life -= dt;
            p.pos += p.vel;
            p.rotation += p.rot_speed * dt;

            // Gravity on particles
            if p.is_confetti {
                p.vel.y += 4.5 * dt;
                p.vel.x *= 0.98;
            } else {
                p.vel.y += 8.0 * dt;
                p.vel.x *= 0.95;
            }
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    pub fn render(&self) {
        // Render shockwaves
        for sw in &self.shockwaves {
            let alpha = (sw.life / sw.max_life).clamp(0.0, 1.0);
            let mut c = sw.color;
            c.a *= alpha;
            draw_circle_lines(sw.pos.x, sw.pos.y, sw.radius, sw.width * alpha, c);
        }

        // Render particles
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut c = p.color;
            c.a *= alpha;

            if p.is_confetti {
                draw_rectangle_ex(
                    p.pos.x,
                    p.pos.y,
                    p.size,
                    p.size * 0.6,
                    DrawRectangleParams {
                        rotation: p.rotation,
                        color: c,
                        ..Default::default()
                    },
                );
            } else {
                draw_circle(p.pos.x, p.pos.y, p.size * alpha, c);
            }
        }
    }

    pub fn get_shake_offset(&self) -> Vec2 {
        if self.screen_shake <= 0.0 {
            return Vec2::ZERO;
        }
        Vec2::new(
            gen_range(-self.screen_shake, self.screen_shake),
            gen_range(-self.screen_shake, self.screen_shake),
        )
    }
}
