// Blobby Volley 3 - Deterministic Physics Engine
use glam::Vec2;
use serde::{Serialize, Deserialize};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSide {
    Left = 0,
    Right = 1,
}

impl PlayerSide {
    pub fn opponent(self) -> Self {
        match self {
            PlayerSide::Left => PlayerSide::Right,
            PlayerSide::Right => PlayerSide::Left,
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub up: bool,
}

#[derive(Debug, Clone)]
pub enum MatchEvent {
    BallHitBlob { player: PlayerSide, intensity: f32 },
    BallHitGround { side: PlayerSide },
    BallHitWall { side: PlayerSide },
    BallHitNet { side: PlayerSide },
    BallHitNetTop,
    BallCrossedNet { new_side: PlayerSide },
}

#[derive(Debug, Clone)]
pub struct Blobby {
    pub pos: Vec2,
    pub vel: Vec2,
    pub anim_state: f32,
    pub anim_speed: f32,
    // Modern 2026 procedural squash & stretch
    pub squash_factor: f32,
    pub squash_vel: f32,
    // Eye look target
    pub eye_target: Vec2,
}

impl Blobby {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            anim_state: 0.0,
            anim_speed: 0.0,
            squash_factor: 1.0,
            squash_vel: 0.0,
            eye_target: Vec2::new(400.0, 300.0),
        }
    }

    pub fn start_animation(&mut self) {
        if self.anim_speed == 0.0 {
            self.anim_speed = 0.5;
        }
    }

    pub fn animation_step(&mut self) {
        if self.anim_state < 0.0 {
            self.anim_speed = 0.0;
            self.anim_state = 0.0;
        }
        if self.anim_state >= 4.5 {
            self.anim_speed = -0.5;
        }
        self.anim_state += self.anim_speed;
        if self.anim_state >= 5.0 {
            self.anim_state = 4.99;
        }
    }

    /// Procedural spring dynamics for 2026 organic jelly deformation
    pub fn update_procedural_squash(&mut self, dt: f32, on_ground: bool) {
        // Target squash based on movement and jump/fall
        let target_squash = if on_ground {
            1.0
        } else if self.vel.y < -3.0 {
            // Stretching upwards while rising
            1.18
        } else if self.vel.y > 4.0 {
            // Compressing slightly in anticipation of landing
            0.92
        } else {
            1.0
        };

        // Damped harmonic oscillator
        let spring_k = 180.0;
        let damping = 14.0;
        let force = -spring_k * (self.squash_factor - target_squash) - damping * self.squash_vel;
        self.squash_vel += force * dt;
        self.squash_factor += self.squash_vel * dt;

        // Clamp to prevent visual artifacts
        self.squash_factor = self.squash_factor.clamp(0.65, 1.45);
    }
}

#[derive(Debug, Clone)]
pub struct Ball {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f32,
    pub angular_velocity: f32,
}

impl Ball {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            rotation: 0.0,
            angular_velocity: STANDARD_BALL_ANGULAR_VELOCITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicWorld {
    pub blobs: [Blobby; 2],
    pub ball: Ball,
    pub events: Vec<MatchEvent>,
    pub prev_ball_pos: Vec2,
    pub prev_blob_pos: [Vec2; 2],
    pub current_ball_side: PlayerSide,
    pub blob_collision_cooldown: [i32; 2],
}

impl PhysicWorld {
    pub fn new() -> Self {
        Self {
            blobs: [
                Blobby::new(200.0, GROUND_PLANE_HEIGHT),
                Blobby::new(600.0, GROUND_PLANE_HEIGHT),
            ],
            ball: Ball::new(200.0, STANDARD_BALL_HEIGHT),
            events: Vec::new(),
            prev_ball_pos: Vec2::new(200.0, STANDARD_BALL_HEIGHT),
            prev_blob_pos: [
                Vec2::new(200.0, GROUND_PLANE_HEIGHT),
                Vec2::new(600.0, GROUND_PLANE_HEIGHT),
            ],
            current_ball_side: PlayerSide::Left,
            blob_collision_cooldown: [0, 0],
        }
    }

    pub fn reset_players(&mut self) {
        self.blobs[0].pos = Vec2::new(200.0, GROUND_PLANE_HEIGHT);
        self.blobs[0].vel = Vec2::ZERO;
        self.blobs[0].squash_factor = 1.0;
        self.blobs[0].squash_vel = 0.0;

        self.blobs[1].pos = Vec2::new(600.0, GROUND_PLANE_HEIGHT);
        self.blobs[1].vel = Vec2::ZERO;
        self.blobs[1].squash_factor = 1.0;
        self.blobs[1].squash_vel = 0.0;

        self.prev_blob_pos[0] = self.blobs[0].pos;
        self.prev_blob_pos[1] = self.blobs[1].pos;
    }

    pub fn reset_ball(&mut self, serving_side: Option<PlayerSide>) {
        let side = serving_side.unwrap_or(PlayerSide::Left);
        match side {
            PlayerSide::Left => {
                self.ball.pos = Vec2::new(200.0, STANDARD_BALL_HEIGHT);
                self.ball.angular_velocity = STANDARD_BALL_ANGULAR_VELOCITY;
            }
            PlayerSide::Right => {
                self.ball.pos = Vec2::new(600.0, STANDARD_BALL_HEIGHT);
                self.ball.angular_velocity = -STANDARD_BALL_ANGULAR_VELOCITY;
            }
        }
        self.ball.vel = Vec2::ZERO;
        self.prev_ball_pos = self.ball.pos;
        self.current_ball_side = side;
        self.blob_collision_cooldown = [0, 0];
    }

    pub fn blob_hit_ground(&self, side: PlayerSide) -> bool {
        self.blobs[side.index()].pos.y >= GROUND_PLANE_HEIGHT
    }

    fn player_top_ball_collision(&self, side: PlayerSide) -> bool {
        let idx = side.index();
        let blob_disp = self.blobs[idx].pos - self.prev_blob_pos[idx];
        let rel_prev_ball = self.prev_ball_pos - blob_disp;
        let blob_pos = Vec2::new(
            self.blobs[idx].pos.x,
            self.blobs[idx].pos.y - BLOBBY_UPPER_SPHERE,
        );
        let r = BALL_RADIUS + BLOBBY_UPPER_RADIUS;
        circle_circle_collision(
            self.ball.pos,
            BALL_RADIUS,
            blob_pos,
            BLOBBY_UPPER_RADIUS,
        ) || segment_circle_collision(rel_prev_ball, self.ball.pos, blob_pos, r)
    }

    fn player_bottom_ball_collision(&self, side: PlayerSide) -> bool {
        let idx = side.index();
        let blob_disp = self.blobs[idx].pos - self.prev_blob_pos[idx];
        let rel_prev_ball = self.prev_ball_pos - blob_disp;
        let blob_pos = Vec2::new(
            self.blobs[idx].pos.x,
            self.blobs[idx].pos.y + BLOBBY_LOWER_SPHERE,
        );
        let r = BALL_RADIUS + BLOBBY_LOWER_RADIUS;
        circle_circle_collision(
            self.ball.pos,
            BALL_RADIUS,
            blob_pos,
            BLOBBY_LOWER_RADIUS,
        ) || segment_circle_collision(rel_prev_ball, self.ball.pos, blob_pos, r)
    }

    fn handle_blob(&mut self, side: PlayerSide, input: PlayerInput) {
        let idx = side.index();
        let mut current_blobby_gravity = GRAVITATION;
        let on_ground = self.blob_hit_ground(side);

        if input.up {
            if on_ground {
                self.blobs[idx].vel.y = BLOBBY_JUMP_ACCELERATION;
                self.blobs[idx].start_animation();
                // Trigger stretch squash
                self.blobs[idx].squash_vel += 2.5;
            }
            current_blobby_gravity -= BLOBBY_JUMP_BUFFER;
        }

        if (input.left || input.right) && on_ground {
            self.blobs[idx].start_animation();
        }

        let speed_x = if input.right { BLOBBY_SPEED } else { 0.0 }
            - if input.left { BLOBBY_SPEED } else { 0.0 };
        self.blobs[idx].vel.x = speed_x;

        // dt = 1 in original physics simulation
        // ds = 0.5 * a * dt^2 + v * dt
        self.blobs[idx].pos += Vec2::new(0.0, 0.5 * current_blobby_gravity) + self.blobs[idx].vel;
        self.blobs[idx].vel.y += current_blobby_gravity;

        // Hitting the ground
        if self.blobs[idx].pos.y > GROUND_PLANE_HEIGHT {
            if self.blobs[idx].vel.y > 3.5 {
                self.blobs[idx].start_animation();
                // Land impact squash
                self.blobs[idx].squash_vel -= self.blobs[idx].vel.y * 0.45;
            }
            self.blobs[idx].pos.y = GROUND_PLANE_HEIGHT;
            self.blobs[idx].vel.y = 0.0;
        }

        self.blobs[idx].animation_step();
        self.blobs[idx].update_procedural_squash(PHYSICS_DT, on_ground);

        // Update eye tracking towards ball
        self.blobs[idx].eye_target = self.ball.pos;
    }

    fn handle_blobby_ball_collision(&mut self, side: PlayerSide) -> bool {
        let idx = side.index();
        if self.blob_collision_cooldown[idx] > 0 {
            return false;
        }

        let mut collision_center = self.blobs[idx].pos;

        if self.player_bottom_ball_collision(side) {
            collision_center.y += BLOBBY_LOWER_SPHERE;
        } else if self.player_top_ball_collision(side) {
            collision_center.y -= BLOBBY_UPPER_SPHERE;
        } else {
            return false;
        }

        // Hit intensity
        let relative_vel = self.ball.vel - self.blobs[idx].vel;
        let intensity = (relative_vel.length() / 25.0).min(1.0);

        // Normal points from collision center to ball
        let diff = self.ball.pos - collision_center;
        let normal = if diff.length_squared() > 1e-6 {
            diff.normalize()
        } else {
            Vec2::new(0.0, -1.0)
        };

        self.ball.vel = normal * BALL_COLLISION_VELOCITY;
        self.ball.pos += self.ball.vel;

        // Ball impact squash on blob
        self.blobs[idx].squash_vel -= intensity * 1.5;

        self.blob_collision_cooldown[idx] = 6;
        self.blob_collision_cooldown[side.opponent().index()] = 0;

        self.events.push(MatchEvent::BallHitBlob {
            player: side,
            intensity,
        });

        true
    }

    fn handle_ball_world_collisions(&mut self) {
        // Ball to ground collision
        if self.ball.pos.y + BALL_RADIUS > GROUND_PLANE_HEIGHT_MAX {
            self.ball.vel.y = -self.ball.vel.y * 0.95;
            self.ball.vel.x *= 0.95;
            self.ball.pos.y = GROUND_PLANE_HEIGHT_MAX - BALL_RADIUS;
            let side = if self.ball.pos.x > NET_POSITION_X {
                PlayerSide::Right
            } else {
                PlayerSide::Left
            };
            self.events.push(MatchEvent::BallHitGround { side });
        }

        // Border left
        if self.ball.pos.x - BALL_RADIUS <= LEFT_PLANE && self.ball.vel.x < 0.0 {
            self.ball.vel.x = -self.ball.vel.x;
            self.ball.pos.x = LEFT_PLANE + BALL_RADIUS;
            self.events.push(MatchEvent::BallHitWall {
                side: PlayerSide::Left,
            });
        }
        // Border right
        else if self.ball.pos.x + BALL_RADIUS >= RIGHT_PLANE && self.ball.vel.x > 0.0 {
            self.ball.vel.x = -self.ball.vel.x;
            self.ball.pos.x = RIGHT_PLANE - BALL_RADIUS;
            self.events.push(MatchEvent::BallHitWall {
                side: PlayerSide::Right,
            });
        }
        // Net vertical stem collision
        else if self.ball.pos.y > NET_SPHERE_POSITION
            && (self.ball.pos.x - NET_POSITION_X).abs() < BALL_RADIUS + NET_RADIUS
        {
            let right = self.ball.pos.x - NET_POSITION_X > 0.0;
            self.ball.vel.x = -self.ball.vel.x;
            self.ball.pos.x = NET_POSITION_X
                + if right {
                    BALL_RADIUS + NET_RADIUS
                } else {
                    -(BALL_RADIUS + NET_RADIUS)
                };
            let side = if right {
                PlayerSide::Right
            } else {
                PlayerSide::Left
            };
            self.events.push(MatchEvent::BallHitNet { side });
        } else {
            // Net rounded top cap collision
            let net_top = Vec2::new(NET_POSITION_X, NET_SPHERE_POSITION);
            let dist_to_top = (self.ball.pos - net_top).length();

            if dist_to_top < NET_RADIUS + BALL_RADIUS {
                // Vector pointing from ball towards net top
                let normal = if dist_to_top > 1e-4 {
                    (net_top - self.ball.pos).normalize()
                } else {
                    Vec2::new(0.0, 1.0)
                };

                let perp_dot = normal.dot(self.ball.vel);
                let perp_ekin = perp_dot * perp_dot;
                let para_ekin = self.ball.vel.length_squared() - perp_ekin;

                let perp_ekin_damped = perp_ekin * 0.7;
                let para_ekin_damped = para_ekin * 0.9;
                let new_speed = (perp_ekin_damped + para_ekin_damped).max(0.0).sqrt();

                // Reflect ball across normal
                let reflected = self.ball.vel - normal * (2.0 * normal.dot(self.ball.vel));
                let reflected_dir = if reflected.length_squared() > 1e-6 {
                    reflected.normalize()
                } else {
                    Vec2::new(0.0, -1.0)
                };
                self.ball.vel = reflected_dir * new_speed;

                // Push ball out of the net top circle
                self.ball.pos = net_top - normal * (NET_RADIUS + BALL_RADIUS);

                self.events.push(MatchEvent::BallHitNetTop);
            }
        }
    }

    pub fn step(
        &mut self,
        left_input: PlayerInput,
        right_input: PlayerInput,
        _is_ball_valid: bool,
        is_game_running: bool,
    ) {
        self.blob_collision_cooldown[0] = (self.blob_collision_cooldown[0] - 1).max(0);
        self.blob_collision_cooldown[1] = (self.blob_collision_cooldown[1] - 1).max(0);

        self.prev_ball_pos = self.ball.pos;
        self.prev_blob_pos[0] = self.blobs[0].pos;
        self.prev_blob_pos[1] = self.blobs[1].pos;

        // Blobby physical steps
        self.handle_blob(PlayerSide::Left, left_input);
        self.handle_blob(PlayerSide::Right, right_input);

        // Ball movement when in play
        if is_game_running {
            self.ball.pos += Vec2::new(0.0, 0.5 * BALL_GRAVITATION) + self.ball.vel;
            self.ball.vel.y += BALL_GRAVITATION;
        }

        // Ball blobby collision: always physically bounce so ball never phases through blob like air
        self.handle_blobby_ball_collision(PlayerSide::Left);
        self.handle_blobby_ball_collision(PlayerSide::Right);

        // Ball world collisions (walls, net, ground)
        self.handle_ball_world_collisions();

        // Check if ball crossed the net plane
        let new_side = if self.ball.pos.x < NET_POSITION_X {
            PlayerSide::Left
        } else {
            PlayerSide::Right
        };
        if new_side != self.current_ball_side {
            self.current_ball_side = new_side;
            self.events.push(MatchEvent::BallCrossedNet { new_side });
            self.blob_collision_cooldown = [0, 0];
        }

        // Blobby net constraints
        if self.blobs[0].pos.x + BLOBBY_LOWER_RADIUS > NET_POSITION_X - NET_RADIUS {
            self.blobs[0].pos.x = NET_POSITION_X - NET_RADIUS - BLOBBY_LOWER_RADIUS;
        }
        if self.blobs[1].pos.x - BLOBBY_LOWER_RADIUS < NET_POSITION_X + NET_RADIUS {
            self.blobs[1].pos.x = NET_POSITION_X + NET_RADIUS + BLOBBY_LOWER_RADIUS;
        }

        // Blobby border constraints
        if self.blobs[0].pos.x < LEFT_PLANE {
            self.blobs[0].pos.x = LEFT_PLANE;
        }
        if self.blobs[1].pos.x > RIGHT_PLANE {
            self.blobs[1].pos.x = RIGHT_PLANE;
        }

        // Ball rotation update
        if !is_game_running {
            self.ball.rotation -= self.ball.angular_velocity;
        } else if self.ball.vel.x > 0.0 {
            self.ball.rotation += self.ball.angular_velocity * (self.ball.vel.length() / 6.0);
        } else {
            self.ball.rotation -= self.ball.angular_velocity * (self.ball.vel.length() / 6.0);
        }

        let tau = std::f32::consts::TAU;
        if self.ball.rotation < 0.0 {
            self.ball.rotation += tau;
        } else if self.ball.rotation >= tau {
            self.ball.rotation -= tau;
        }
    }

    pub fn take_events(&mut self) -> Vec<MatchEvent> {
        std::mem::take(&mut self.events)
    }
}

#[inline]
pub fn circle_circle_collision(pos1: Vec2, rad1: f32, pos2: Vec2, rad2: f32) -> bool {
    let dist_sq = (pos1 - pos2).length_squared();
    let sum = rad1 + rad2;
    dist_sq < sum * sum
}

/// Continuous Collision Detection (CCD): Check whether a moving sphere swept from p1 to p2 intersects a circle
#[inline]
pub fn segment_circle_collision(p1: Vec2, p2: Vec2, center: Vec2, radius: f32) -> bool {
    let d = p2 - p1;
    let len_sq = d.length_squared();
    let t = if len_sq > 1e-6 {
        ((center - p1).dot(d) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let closest = p1 + d * t;
    (center - closest).length_squared() < radius * radius
}
