// Blobby Volley 3 - Authentic Combot 1.1 & AI Engine
// Faithfully aligned with Blobby Volley 2 Combot 1.1 (com_11.lua) and Reduced bot (reduced.lua)

use glam::Vec2;
use quad_rand::gen_range;
use crate::physics::{PhysicWorld, PlayerInput, PlayerSide};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    Novice,
    Pro,
    Master,
}

impl AiDifficulty {
    pub fn name(self) -> &'static str {
        match self {
            AiDifficulty::Novice => "Novice",
            AiDifficulty::Pro => "Pro (Reduced)",
            AiDifficulty::Master => "Master (Combot 1.1)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AiDifficulty::Novice => AiDifficulty::Pro,
            AiDifficulty::Pro => AiDifficulty::Master,
            AiDifficulty::Master => AiDifficulty::Novice,
        }
    }
}

pub struct BotController {
    pub side: PlayerSide,
    pub difficulty: AiDifficulty,
    // Combot 1.1 parameters
    attack_power: f32,
    serve_offset: f32,
    next_smash: bool,
    // Reduced bot parameters
    serve_rand: Option<f32>,
    mode_lock: bool,
}

impl BotController {
    pub fn new(side: PlayerSide, difficulty: AiDifficulty) -> Self {
        Self {
            side,
            difficulty,
            attack_power: 42.0,
            serve_offset: -6.0,
            next_smash: false,
            serve_rand: None,
            mode_lock: false,
        }
    }

    pub fn reset_rally(&mut self) {
        // Randomize attack power between 30 and 55 (matching CONST_ANGRIFFSGRUNDWERT_MIN/MAX in com_11.lua)
        self.attack_power = gen_range(30.0, 55.0);
        self.serve_offset = gen_range(-8.0, -4.0);
        self.next_smash = false;
        self.serve_rand = None;
        self.mode_lock = false;
    }

    /// Updates the bot using canonical left-side coordinates (mirroring for right side)
    pub fn update(
        &mut self,
        physics: &PhysicWorld,
        touches: i32,
        serving_player: PlayerSide,
        is_game_running: bool,
    ) -> PlayerInput {
        let is_right = self.side == PlayerSide::Right;
        let my_idx = self.side.index();

        // 1. Mirror state to canonical Left side
        let my_pos_x = if is_right { RIGHT_PLANE - physics.blobs[my_idx].pos.x } else { physics.blobs[my_idx].pos.x };
        let my_pos_y = physics.blobs[my_idx].pos.y;
        let my_vel_y = physics.blobs[my_idx].vel.y;

        let ball_pos_x = if is_right { RIGHT_PLANE - physics.ball.pos.x } else { physics.ball.pos.x };
        let ball_pos_y = physics.ball.pos.y;
        let ball_vel_x = if is_right { -physics.ball.vel.x } else { physics.ball.vel.x };
        let ball_vel_y = physics.ball.vel.y;

        let ball_pos = Vec2::new(ball_pos_x, ball_pos_y);
        let ball_vel = Vec2::new(ball_vel_x, ball_vel_y);
        let my_pos = Vec2::new(my_pos_x, my_pos_y);

        let i_am_serving = serving_player == self.side;

        // 2. Decide actions in canonical space
        let canonical_input = self.decide_canonical(
            my_pos,
            my_vel_y,
            ball_pos,
            ball_vel,
            touches,
            i_am_serving,
            is_game_running,
        );

        // 3. Mirror actions back if on Right side
        if is_right {
            PlayerInput {
                left: canonical_input.right,
                right: canonical_input.left,
                up: canonical_input.up,
            }
        } else {
            canonical_input
        }
    }

    fn decide_canonical(
        &mut self,
        my_pos: Vec2,
        my_vel_y: f32,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
    ) -> PlayerInput {
        let mut input = PlayerInput::default();

        // Delegate to chosen difficulty behavior
        match self.difficulty {
            AiDifficulty::Novice => {
                self.decide_novice(my_pos, my_vel_y, ball_pos, ball_vel, i_am_serving, is_game_running, &mut input);
            }
            AiDifficulty::Pro => {
                self.decide_reduced(my_pos, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input);
            }
            AiDifficulty::Master => {
                self.decide_combot_11(my_pos, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input);
            }
        }

        input
    }

    /// Novice AI: relaxed reaction, ground returns, accessible for new players
    fn decide_novice(
        &mut self,
        my_pos: Vec2,
        my_vel_y: f32,
        ball_pos: Vec2,
        ball_vel: Vec2,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
    ) {
        if !is_game_running {
            if i_am_serving {
                let target_x = ball_pos.x - 3.0;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, 120.0, input);
            }
            return;
        }

        let head_y = 379.5;
        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true);

        let target_x = if let Some((tx, _, _)) = impact_head {
            if tx > 390.0 { 120.0 } else { tx.clamp(45.0, 360.0) }
        } else {
            if ball_pos.x > 380.0 { 120.0 } else { ball_pos.x.clamp(45.0, 360.0) }
        };

        // Moveto with slight reaction deadband
        let deadband = 12.0;
        if my_pos.x < target_x - deadband {
            input.right = true;
        } else if my_pos.x > target_x + deadband {
            input.left = true;
        }

        // Gentle hop if ball is low and close to head
        let dist = (my_pos.x - ball_pos.x).abs();
        if dist < 25.0 && ball_pos.y > 350.0 && my_vel_y >= 0.0 {
            input.up = true;
        }
    }

    /// Pro AI: Authentic port of Blobby Volley 2 "reduced.lua"
    fn decide_reduced(
        &mut self,
        my_pos: Vec2,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
    ) {
        if !is_game_running {
            if i_am_serving {
                if self.serve_rand.is_none() {
                    self.serve_rand = Some(gen_range(0.0, 5.0));
                }
                let target_x = ball_pos.x + self.serve_rand.unwrap();
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                    self.serve_rand = None;
                }
            } else {
                moveto(my_pos.x, 100.0, input);
            }
            return;
        }

        // estimImpactHigh: destination y = 242.0 (CONST_BLOBBY_MAX_JUMP - 25 in BV2)
        let impact_high = estimate_x_at_y(ball_pos, ball_vel, 242.0, true);
        // estimImpactLow: destination y = 379.5 (CONST_BALL_BLOBBY_HEAD in BV2)
        let impact_low = estimate_x_at_y(ball_pos, ball_vel, 379.5, true);

        // Try High Play
        if let Some((target_high, est_vx, timeto)) = impact_high {
            let naive_target = ball_pos.x + ball_vel.x * timeto;
            let high_play_pos = if est_vx < 0.0 {
                target_high - 50.0 - est_vx * 5.0
            } else {
                target_high - 50.0
            }.clamp(35.0, 365.0);

            let can_reach = timeto > (my_pos.x - high_play_pos).abs() / BLOBBY_SPEED + 26.0;

            if naive_target < 400.0 && target_high < 400.0 && (self.mode_lock || can_reach) && touches < 3 {
                self.mode_lock = timeto < 30.0;
                if !self.mode_lock {
                    self.serve_rand = None;
                }

                // highPlay
                if target_high > 400.0 {
                    moveto(my_pos.x, 100.0, input);
                } else {
                    moveto(my_pos.x, high_play_pos, input);
                    if self.serve_rand.is_none() {
                        self.serve_rand = Some(gen_range(0.0, 1.0));
                    }
                    if naive_target < 400.0 && timeto < (28.0 + self.serve_rand.unwrap()) {
                        input.up = true;
                    }
                }
                return;
            }
        }

        // Low Play
        self.mode_lock = false;
        self.serve_rand = None;

        if let Some((target_low, est_vx, timeto)) = impact_low {
            let naive_target = ball_pos.x + ball_vel.x * timeto;
            let balldir = if est_vx > 0.0 { 1.0 } else { -1.0 };
            let in_time = timeto > (balldir * (target_low - my_pos.x) - 10.0) / BLOBBY_SPEED;

            if in_time || naive_target >= 400.0 {
                // Low play: move to target, ground receive (no jump)
                if target_low > 400.0 {
                    moveto(my_pos.x, 100.0, input);
                } else {
                    moveto(my_pos.x, target_low, input);
                }
            } else if naive_target < 400.0 {
                // PANIC SAVE! Dive/jump towards target
                moveto(my_pos.x, target_low, input);
                input.up = true;
            }
        } else {
            // Ball not reachable or high in air
            if ball_pos.x > 380.0 {
                moveto(my_pos.x, 100.0, input);
            } else {
                moveto(my_pos.x, ball_pos.x.clamp(45.0, 360.0), input);
            }
        }
    }

    /// Master AI: Authentic port of Blobby Volley 2 "com_11.lua" (Combot 1.1)
    fn decide_combot_11(
        &mut self,
        my_pos: Vec2,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
    ) {
        if !is_game_running {
            if i_am_serving {
                self.next_smash = true;
                let target_x = ball_pos.x + self.serve_offset;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, 130.0, input);
                self.reset_rally();
            }
            return;
        }

        let head_y = 379.5;
        let net_y = 276.0;
        let jump_y = 217.0; // Lua 383: 600 - 383 = 217

        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true);
        let impact_net = estimate_x_at_y(ball_pos, ball_vel, net_y, true);
        let impact_jump = estimate_x_at_y(ball_pos, ball_vel, jump_y, true);

        // naechsterBallSchmetternFlagTesten:
        if touches >= 3 || ball_pos.x > 400.0 {
            self.next_smash = false;
        } else if touches == 1 && ball_vel.x.abs() < 2.5 {
            self.next_smash = true;
        } else if touches == 2 {
            self.next_smash = true;
        }

        // Net roller recovery (targetNetz > CONST_BALL_LEFT_NET - 10)
        if let Some((target_net_x, _, _)) = impact_net {
            if target_net_x > 360.0 && target_net_x <= 400.0 {
                self.next_smash = true;
            }
        }

        // Target at head height
        let target = impact_head.map(|(x, _, _)| x).unwrap_or(800.0);

        if target > 395.0 && ball_pos.x > 380.0 {
            // Ball belongs to opponent: wait at base 135 and roll power
            moveto(my_pos.x, 135.0, input);
            self.reset_rally();
            return;
        }

        // If next_smash is active, execute sprungattacke or weiterleiten
        if self.next_smash {
            if let Some((target_jump_x, target_speed, time_to_jump)) = impact_jump {
                if target_jump_x < 380.0 && target_jump_x > 40.0 {
                    if target_speed.abs() < 2.5 {
                        // sprungattacke:
                        let net_dist_ratio = (target_jump_x / 360.0).clamp(0.0, 1.0);
                        let min_power = 30.0 + 10.0 * net_dist_ratio;
                        let max_power = 55.0 - 10.0 * net_dist_ratio;
                        let power = self.attack_power.clamp(min_power, max_power);

                        let attack_x = (target_jump_x - power).clamp(35.0, 365.0);
                        moveto(my_pos.x, attack_x, input);

                        // jumpto(383): blobtimetoy(383) is ~31.6 ticks
                        if time_to_jump <= 31.0 && (my_pos.x - attack_x).abs() < 22.0 {
                            input.up = true;
                        }
                        return;
                    } else {
                        // weiterleiten: move to 200 and pass forward
                        moveto(my_pos.x, 200.0, input);
                        if time_to_jump <= 28.0 {
                            input.up = true;
                        }
                        return;
                    }
                }
            }
        }

        // Low play: moveto(target) with firm ground receive (no jump!)
        let head_dest = if target < 400.0 { target } else { ball_pos.x.clamp(45.0, 360.0) };
        moveto(my_pos.x, head_dest, input);

        // Emergency dive jump if ball is dropping very fast out of reach
        if ball_pos.y > 430.0 && (my_pos.x - ball_pos.x).abs() > 22.0 && ball_pos.x < 390.0 {
            input.up = true;
        }
    }
}

/// Precise moveto matching bot_api.lua:
/// if x < target - SPEED/2: right; if x > target + SPEED/2: left; else in position
#[inline]
fn moveto(current_x: f32, target_x: f32, input: &mut PlayerInput) -> bool {
    let half_speed = BLOBBY_SPEED * 0.5;
    if current_x < target_x - half_speed {
        input.right = true;
        input.left = false;
        false
    } else if current_x > target_x + half_speed {
        input.left = true;
        input.right = false;
        false
    } else {
        input.left = false;
        input.right = false;
        true
    }
}

/// Exact physical ball simulation in canonical space (left court = [0, 400], right court = [400, 800]).
/// Matches PhysicWorld ball simulation tick-for-tick up to 375 ticks (5 seconds at 75 Hz).
pub fn estimate_x_at_y(
    mut pos: Vec2,
    mut vel: Vec2,
    dest_y: f32,
    downward: bool,
) -> Option<(f32, f32, f32)> {
    let max_ticks = 375;

    for tick in 1..=max_ticks {
        // dt = 1
        pos += Vec2::new(0.0, 0.5 * BALL_GRAVITATION) + vel;
        vel.y += BALL_GRAVITATION;

        // Ground collision
        if pos.y + BALL_RADIUS > GROUND_PLANE_HEIGHT_MAX {
            vel.y = -vel.y * 0.95;
            pos.y = GROUND_PLANE_HEIGHT_MAX - BALL_RADIUS;
        }

        // Left wall
        if pos.x - BALL_RADIUS <= LEFT_PLANE && vel.x < 0.0 {
            vel.x = -vel.x;
            pos.x = LEFT_PLANE + BALL_RADIUS;
        }

        // Right wall
        if pos.x + BALL_RADIUS >= RIGHT_PLANE && vel.x > 0.0 {
            vel.x = -vel.x;
            pos.x = RIGHT_PLANE - BALL_RADIUS;
        }

        // Net stem collision
        if pos.y > NET_SPHERE_POSITION {
            let left_bound = NET_POSITION_X - NET_RADIUS - BALL_RADIUS;
            let right_bound = NET_POSITION_X + NET_RADIUS + BALL_RADIUS;
            if pos.x >= left_bound && pos.x <= right_bound {
                if vel.x > 0.0 {
                    vel.x = -vel.x;
                    pos.x = left_bound;
                } else if vel.x < 0.0 {
                    vel.x = -vel.x;
                    pos.x = right_bound;
                }
            }
        }

        // Net round top cap
        let net_pos = Vec2::new(NET_POSITION_X, NET_SPHERE_POSITION);
        let dist = (pos - net_pos).length();
        let total_radius = BALL_RADIUS + NET_RADIUS;
        if dist < total_radius {
            let normal = (net_pos - pos).normalize();
            let perp = normal.dot(vel);
            let para_sq = vel.length_squared() - perp * perp;
            let speed = (perp * perp * 0.7 + para_sq * 0.9).max(0.0).sqrt();
            vel = (vel - normal * (2.0 * normal.dot(vel))).normalize() * speed;
            pos = net_pos - normal * total_radius;
        }

        // Destination condition
        if downward {
            if pos.y >= dest_y && vel.y > 0.0 {
                return Some((pos.x, vel.x, tick as f32));
            }
        } else {
            if (pos.y - dest_y).abs() < 12.0 {
                return Some((pos.x, vel.x, tick as f32));
            }
        }
    }

    None
}

#[allow(dead_code)]
pub fn estimate_ball_trajectory(ball_pos: Vec2, ball_vel: Vec2, target_y: f32) -> Option<(f32, f32, f32)> {
    estimate_x_at_y(ball_pos, ball_vel, target_y, true)
}
