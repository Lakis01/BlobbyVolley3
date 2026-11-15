// Blobby Volley 3 - Authentic Combot 1.1 & AI Engine
// Faithfully aligned with Blobby Volley 2 Combot 1.1 (com_11.lua) and Reduced bot (reduced.lua)

use glam::Vec2;
use quad_rand::gen_range;
use crate::physics::{PhysicWorld, PlayerInput, PlayerSide};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiDifficulty {
    Novice,
    Amateur,
    Regular,
    Master,
    Pro,
}

impl AiDifficulty {
    pub fn name(self) -> &'static str {
        match self {
            AiDifficulty::Novice => "Novice",
            AiDifficulty::Amateur => "Amateur",
            AiDifficulty::Regular => "Regular",
            AiDifficulty::Master => "Master",
            AiDifficulty::Pro => "Pro",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AiDifficulty::Novice => AiDifficulty::Amateur,
            AiDifficulty::Amateur => AiDifficulty::Regular,
            AiDifficulty::Regular => AiDifficulty::Master,
            AiDifficulty::Master => AiDifficulty::Pro,
            AiDifficulty::Pro => AiDifficulty::Novice,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            AiDifficulty::Novice => AiDifficulty::Pro,
            AiDifficulty::Amateur => AiDifficulty::Novice,
            AiDifficulty::Regular => AiDifficulty::Amateur,
            AiDifficulty::Master => AiDifficulty::Regular,
            AiDifficulty::Pro => AiDifficulty::Master,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => AiDifficulty::Novice,
            1 => AiDifficulty::Amateur,
            2 => AiDifficulty::Regular,
            3 => AiDifficulty::Master,
            _ => AiDifficulty::Pro,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            AiDifficulty::Novice => 0,
            AiDifficulty::Amateur => 1,
            AiDifficulty::Regular => 2,
            AiDifficulty::Master => 3,
            AiDifficulty::Pro => 4,
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
        let court_width = physics.court_width;
        let half_w = physics.net_x();

        // 1. Mirror state to canonical Left side
        let my_pos_x = if is_right { court_width - physics.blobs[my_idx].pos.x } else { physics.blobs[my_idx].pos.x };
        let my_pos_y = physics.blobs[my_idx].pos.y;
        let my_vel_y = physics.blobs[my_idx].vel.y;

        let ball_pos_x = if is_right { court_width - physics.ball.pos.x } else { physics.ball.pos.x };
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
            half_w,
            court_width,
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
        half_w: f32,
        court_width: f32,
    ) -> PlayerInput {
        let mut input = PlayerInput::default();

        // Delegate to chosen difficulty behavior
        match self.difficulty {
            AiDifficulty::Novice => {
                self.decide_novice(my_pos, my_vel_y, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input, half_w, court_width);
            }
            AiDifficulty::Amateur => {
                self.decide_amateur(my_pos, my_vel_y, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input, half_w, court_width);
            }
            AiDifficulty::Regular => {
                self.decide_combot_11(my_pos, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input, half_w, court_width);
            }
            AiDifficulty::Master => {
                self.decide_advanced(my_pos, my_vel_y, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input, half_w, court_width);
            }
            AiDifficulty::Pro => {
                self.decide_reduced(my_pos, ball_pos, ball_vel, touches, i_am_serving, is_game_running, &mut input, half_w, court_width);
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
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
        half_w: f32,
        court_width: f32,
    ) {
        let base_x = half_w * 0.32;
        if !is_game_running {
            if i_am_serving {
                let target_x = ball_pos.x - 3.0;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, base_x, input);
            }
            return;
        }

        let head_y = 379.5;
        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true, court_width);

        let target_x = if let Some((tx, _, _)) = impact_head {
            if tx > half_w - 10.0 { base_x } else { tx.clamp(45.0, half_w - 40.0) }
        } else {
            if ball_pos.x > half_w - 20.0 { base_x } else { ball_pos.x.clamp(45.0, half_w - 40.0) }
        };

        // Moveto with slight reaction deadband (gentle beginner tier)
        let deadband = 12.0;
        let hit_target = (target_x - 10.0).clamp(40.0, half_w - 40.0);
        if my_pos.x < hit_target - deadband {
            input.right = true;
        } else if my_pos.x > hit_target + deadband {
            input.left = true;
        }

        // Gentle hop if ball is low and close to head, or on touch 2
        let dist = (my_pos.x - ball_pos.x).abs();
        if (dist < 22.0 && ball_pos.y > 360.0 && my_vel_y >= 0.0) || (touches >= 2 && dist < 28.0 && ball_pos.y > 330.0) {
            input.up = true;
        }
    }

    /// Amateur AI: Intermediate tier strictly between Novice and Regular.
    /// Uses trajectory estimation with full BLOBBY_SPEED, returns cleanly over net,
    /// and pushes forward with a hop on touch 2 or floating net balls.
    fn decide_amateur(
        &mut self,
        my_pos: Vec2,
        my_vel_y: f32,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
        half_w: f32,
        court_width: f32,
    ) {
        let base_x = half_w * 0.32;
        if !is_game_running {
            if i_am_serving {
                let target_x = ball_pos.x - 4.0;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, base_x, input);
            }
            return;
        }

        let head_y = 379.5;
        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true, court_width);

        if let Some((target_x, _, time_to_hit)) = impact_head {
            if target_x > half_w - 5.0 && ball_pos.x > half_w - 20.0 {
                // Ball heading to opponent court: return to base
                moveto(my_pos.x, base_x, input);
            } else {
                // Position slightly behind the ball to hit cleanly forward over the net
                let offset = if target_x > half_w - 70.0 { 8.0 } else { 15.0 };
                let clamped_x = (target_x - offset).clamp(38.0, half_w - 38.0);
                moveto(my_pos.x, clamped_x, input);

                let dist_to_target = (my_pos.x - clamped_x).abs();

                // 1. On touch 2 (last hit!): ALWAYS jump forward to clear the net!
                if touches >= 2 && time_to_hit <= 24.0 && dist_to_target < 28.0 && my_vel_y >= 0.0 {
                    input.up = true;
                }
                // 2. Near net defense: jump push to prevent ball dropping on net tape
                else if target_x > half_w - 85.0 && time_to_hit <= 20.0 && dist_to_target < 24.0 && my_vel_y >= 0.0 {
                    input.up = true;
                }
                // 3. Floating ball attack: if ball is slow and hanging at jump height on touch 1
                else if touches == 1 && ball_vel.y.abs() < 3.0 && ball_pos.y < 330.0 && dist_to_target < 20.0 && my_vel_y >= 0.0 {
                    input.up = true;
                }
            }
        } else {
            let safe_x = if ball_pos.x > half_w - 20.0 { base_x } else { ball_pos.x.clamp(45.0, half_w - 40.0) };
            moveto(my_pos.x, safe_x, input);
        }
    }

    /// Master AI: Advanced tier between Regular (Combot 1.1) and Pro (Reduced).
    /// Combines lethal sprungattacke jump spikes with proactive panic dive saves.
    fn decide_advanced(
        &mut self,
        my_pos: Vec2,
        _my_vel_y: f32,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
        half_w: f32,
        court_width: f32,
    ) {
        let base_x = half_w * 0.32;
        if !is_game_running {
            if i_am_serving {
                self.next_smash = true;
                let target_x = ball_pos.x + self.serve_offset;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, base_x, input);
                self.reset_rally();
            }
            return;
        }

        let head_y = 379.5;
        let net_y = 276.0;
        let jump_y = 217.0;

        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true, court_width);
        let impact_net = estimate_x_at_y(ball_pos, ball_vel, net_y, true, court_width);
        let impact_jump = estimate_x_at_y(ball_pos, ball_vel, jump_y, true, court_width);

        if touches >= 3 || ball_pos.x > half_w {
            self.next_smash = false;
        } else if touches == 1 && ball_vel.x.abs() < 3.0 {
            self.next_smash = true;
        } else if touches == 2 {
            self.next_smash = true;
        }

        // Net tape roller defense
        if let Some((target_net_x, _, _)) = impact_net {
            if target_net_x > half_w - 45.0 && target_net_x <= half_w {
                self.next_smash = true;
            }
        }

        // If ball heading to opponent side, retreat and reset
        let target = impact_head.map(|(x, _, _)| x).unwrap_or(court_width);
        if target > half_w - 5.0 && ball_pos.x > half_w - 20.0 {
            moveto(my_pos.x, base_x, input);
            self.reset_rally();
            return;
        }

        // High smash attack
        if self.next_smash {
            if let Some((target_jump_x, target_speed, time_to_jump)) = impact_jump {
                if target_jump_x < half_w - 20.0 && target_jump_x > 40.0 && target_speed.abs() < 3.5 {
                    let net_dist_ratio = (target_jump_x / (half_w - 40.0)).clamp(0.0, 1.0);
                    let min_power = 28.0 + 10.0 * net_dist_ratio;
                    let max_power = 55.0 - 10.0 * net_dist_ratio;
                    let power = self.attack_power.clamp(min_power, max_power);

                    let attack_x = (target_jump_x - power).clamp(35.0, half_w - 35.0);
                    moveto(my_pos.x, attack_x, input);

                    if time_to_jump <= 31.5 && (my_pos.x - attack_x).abs() < 24.0 {
                        input.up = true;
                    }
                    return;
                }
            }
        }

        // Ground receive with Reduced-style panic dive saves
        if let Some((target_low, est_vx, timeto)) = impact_head {
            let naive_target = ball_pos.x + ball_vel.x * timeto;
            let balldir = if est_vx > 0.0 { 1.0 } else { -1.0 };
            let in_time = timeto > (balldir * (target_low - my_pos.x) - 12.0) / BLOBBY_SPEED;

            if in_time || naive_target >= half_w {
                moveto(my_pos.x, target_low.clamp(35.0, half_w - 35.0), input);
            } else if naive_target < half_w {
                // PANIC DIVE!
                moveto(my_pos.x, target_low, input);
                input.up = true;
            }
        } else {
            let safe_x = if ball_pos.x > half_w - 20.0 { base_x } else { ball_pos.x.clamp(45.0, half_w - 40.0) };
            moveto(my_pos.x, safe_x, input);
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
        half_w: f32,
        court_width: f32,
    ) {
        let base_x = half_w * 0.25;
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
                moveto(my_pos.x, base_x, input);
            }
            return;
        }

        // estimImpactHigh: destination y = 242.0 (CONST_BLOBBY_MAX_JUMP - 25 in BV2)
        let impact_high = estimate_x_at_y(ball_pos, ball_vel, 242.0, true, court_width);
        // estimImpactLow: destination y = 379.5 (CONST_BALL_BLOBBY_HEAD in BV2)
        let impact_low = estimate_x_at_y(ball_pos, ball_vel, 379.5, true, court_width);

        // Try High Play
        if let Some((target_high, est_vx, timeto)) = impact_high {
            let naive_target = ball_pos.x + ball_vel.x * timeto;
            let high_play_pos = if est_vx < 0.0 {
                target_high - 50.0 - est_vx * 5.0
            } else {
                target_high - 50.0
            }.clamp(35.0, half_w - 35.0);

            let can_reach = timeto > (my_pos.x - high_play_pos).abs() / BLOBBY_SPEED + 26.0;

            if naive_target < half_w && target_high < half_w && (self.mode_lock || can_reach) && touches < 3 {
                self.mode_lock = timeto < 30.0;
                if !self.mode_lock {
                    self.serve_rand = None;
                }

                // highPlay
                if target_high > half_w {
                    moveto(my_pos.x, base_x, input);
                } else {
                    moveto(my_pos.x, high_play_pos, input);
                    if self.serve_rand.is_none() {
                        self.serve_rand = Some(gen_range(0.0, 1.0));
                    }
                    if naive_target < half_w && timeto < (28.0 + self.serve_rand.unwrap()) {
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

            if in_time || naive_target >= half_w {
                // Low play: move to target, ground receive (no jump)
                if target_low > half_w {
                    moveto(my_pos.x, base_x, input);
                } else {
                    moveto(my_pos.x, target_low, input);
                }
            } else if naive_target < half_w {
                // PANIC SAVE! Dive/jump towards target
                moveto(my_pos.x, target_low, input);
                input.up = true;
            }
        } else {
            // Ball not reachable or high in air
            if ball_pos.x > half_w - 20.0 {
                moveto(my_pos.x, base_x, input);
            } else {
                moveto(my_pos.x, ball_pos.x.clamp(45.0, half_w - 40.0), input);
            }
        }
    }

    /// Regular AI: Authentic port of Blobby Volley 2 "com_11.lua" (Combot 1.1)
    fn decide_combot_11(
        &mut self,
        my_pos: Vec2,
        ball_pos: Vec2,
        ball_vel: Vec2,
        touches: i32,
        i_am_serving: bool,
        is_game_running: bool,
        input: &mut PlayerInput,
        half_w: f32,
        court_width: f32,
    ) {
        let base_x = half_w * 0.33;
        if !is_game_running {
            if i_am_serving {
                self.next_smash = true;
                let target_x = ball_pos.x + self.serve_offset;
                if moveto(my_pos.x, target_x, input) {
                    input.up = true;
                }
            } else {
                moveto(my_pos.x, base_x, input);
                self.reset_rally();
            }
            return;
        }

        let head_y = 379.5;
        let net_y = 276.0;
        let jump_y = 217.0; // Lua 383: 600 - 383 = 217

        let impact_head = estimate_x_at_y(ball_pos, ball_vel, head_y, true, court_width);
        let impact_net = estimate_x_at_y(ball_pos, ball_vel, net_y, true, court_width);
        let impact_jump = estimate_x_at_y(ball_pos, ball_vel, jump_y, true, court_width);

        // naechsterBallSchmetternFlagTesten:
        if touches >= 3 || ball_pos.x > half_w {
            self.next_smash = false;
        } else if touches == 1 && ball_vel.x.abs() < 2.5 {
            self.next_smash = true;
        } else if touches == 2 {
            self.next_smash = true;
        }

        // Net roller recovery (targetNetz > CONST_BALL_LEFT_NET - 10)
        if let Some((target_net_x, _, _)) = impact_net {
            if target_net_x > half_w - 40.0 && target_net_x <= half_w {
                self.next_smash = true;
            }
        }

        // Target at head height
        let target = impact_head.map(|(x, _, _)| x).unwrap_or(court_width);

        if target > half_w - 5.0 && ball_pos.x > half_w - 20.0 {
            // Ball belongs to opponent: wait at base and roll power
            moveto(my_pos.x, base_x, input);
            self.reset_rally();
            return;
        }

        // If next_smash is active, execute sprungattacke or weiterleiten
        if self.next_smash {
            if let Some((target_jump_x, target_speed, time_to_jump)) = impact_jump {
                if target_jump_x < half_w - 20.0 && target_jump_x > 40.0 {
                    if target_speed.abs() < 2.5 {
                        // sprungattacke:
                        let net_dist_ratio = (target_jump_x / (half_w - 40.0)).clamp(0.0, 1.0);
                        let min_power = 30.0 + 10.0 * net_dist_ratio;
                        let max_power = 55.0 - 10.0 * net_dist_ratio;
                        let power = self.attack_power.clamp(min_power, max_power);

                        let attack_x = (target_jump_x - power).clamp(35.0, half_w - 35.0);
                        moveto(my_pos.x, attack_x, input);

                        // jumpto(383): blobtimetoy(383) is ~31.6 ticks
                        if time_to_jump <= 31.0 && (my_pos.x - attack_x).abs() < 22.0 {
                            input.up = true;
                        }
                        return;
                    } else {
                        // weiterleiten: move to quarter court and pass forward
                        moveto(my_pos.x, half_w * 0.5, input);
                        if time_to_jump <= 28.0 {
                            input.up = true;
                        }
                        return;
                    }
                }
            }
        }

        // Low play: moveto(target) with firm ground receive (no jump!)
        let head_dest = if target < half_w { target } else { ball_pos.x.clamp(45.0, half_w - 40.0) };
        moveto(my_pos.x, head_dest, input);

        // Emergency dive jump if ball is dropping very fast out of reach
        if ball_pos.y > 430.0 && (my_pos.x - ball_pos.x).abs() > 22.0 && ball_pos.x < half_w - 10.0 {
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

/// Exact physical ball simulation in canonical space (left court = [0, half_w], right court = [half_w, court_width]).
/// Matches PhysicWorld ball simulation tick-for-tick up to 375 ticks (5 seconds at 75 Hz).
pub fn estimate_x_at_y(
    mut pos: Vec2,
    mut vel: Vec2,
    dest_y: f32,
    downward: bool,
    court_width: f32,
) -> Option<(f32, f32, f32)> {
    let max_ticks = 375;
    let net_x = court_width * 0.5;

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
        if pos.x + BALL_RADIUS >= court_width && vel.x > 0.0 {
            vel.x = -vel.x;
            pos.x = court_width - BALL_RADIUS;
        }

        // Net stem collision
        if pos.y > NET_SPHERE_POSITION {
            let left_bound = net_x - NET_RADIUS - BALL_RADIUS;
            let right_bound = net_x + NET_RADIUS + BALL_RADIUS;
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
        let net_pos = Vec2::new(net_x, NET_SPHERE_POSITION);
        let dist = (pos - net_pos).length();
        let total_radius = BALL_RADIUS + NET_RADIUS;
        if dist < total_radius {
            let normal = (net_pos - pos).try_normalize().unwrap_or(Vec2::ZERO);
            let perp = normal.dot(vel);
            let para_sq = vel.length_squared() - perp * perp;
            let speed = (perp * perp * 0.7 + para_sq * 0.9).max(0.0).sqrt();
            vel = (vel - normal * (2.0 * normal.dot(vel))).try_normalize().unwrap_or(Vec2::ZERO) * speed;
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
    estimate_x_at_y(ball_pos, ball_vel, target_y, true, 800.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_x_at_y_zero_length_vector_no_nan() {
        // Place ball directly at net cap center with zero velocity
        let net_center = Vec2::new(400.0, NET_SPHERE_POSITION);
        let res = estimate_x_at_y(net_center, Vec2::ZERO, 379.5, true, 800.0);
        if let Some((x, vx, t)) = res {
            assert!(!x.is_nan(), "x must not be NaN");
            assert!(!vx.is_nan(), "vx must not be NaN");
            assert!(!t.is_nan(), "t must not be NaN");
        }
    }
}
