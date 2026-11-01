// Blobby Volley 3 - Smart AI Bot Engine
// Ported and enhanced from reduced.lua, com_11.lua, and axji.lua

use glam::Vec2;
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
            AiDifficulty::Pro => "Pro",
            AiDifficulty::Master => "Master",
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
    mode_lock: bool,
    serve_rand: f32,
}

impl BotController {
    pub fn new(side: PlayerSide, difficulty: AiDifficulty) -> Self {
        Self {
            side,
            difficulty,
            mode_lock: false,
            serve_rand: 0.5,
        }
    }

    pub fn update(
        &mut self,
        physics: &PhysicWorld,
        serving_player: PlayerSide,
        is_game_running: bool,
    ) -> PlayerInput {
        let mut input = PlayerInput::default();
        let my_idx = self.side.index();
        let my_pos = physics.blobs[my_idx].pos;
        let ball_pos = physics.ball.pos;
        let is_my_court = match self.side {
            PlayerSide::Left => ball_pos.x < NET_POSITION_X,
            PlayerSide::Right => ball_pos.x > NET_POSITION_X,
        };

        // When serving: waiting to launch
        if !is_game_running {
            if serving_player == self.side {
                // I am serving: move under ball and jump
                let target_x = ball_pos.x + (self.serve_rand - 0.5) * 8.0;
                if (my_pos.x - target_x).abs() < 12.0 {
                    input.up = true;
                } else if my_pos.x < target_x {
                    input.right = true;
                } else {
                    input.left = true;
                }
            } else {
                // Opponent is serving: return to home base
                let home_x = match self.side {
                    PlayerSide::Left => 200.0,
                    PlayerSide::Right => 600.0,
                };
                if (my_pos.x - home_x).abs() > 10.0 {
                    if my_pos.x < home_x {
                        input.right = true;
                    } else {
                        input.left = true;
                    }
                }
            }
            return input;
        }

        // Active game play
        match self.difficulty {
            AiDifficulty::Novice => self.update_novice(physics, &mut input),
            AiDifficulty::Pro => self.update_pro(physics, &mut input, is_my_court),
            AiDifficulty::Master => self.update_master(physics, &mut input, is_my_court),
        }

        input
    }

    fn update_novice(&mut self, physics: &PhysicWorld, input: &mut PlayerInput) {
        let my_pos = physics.blobs[self.side.index()].pos;
        let ball_pos = physics.ball.pos;

        let home_x = match self.side {
            PlayerSide::Left => 220.0,
            PlayerSide::Right => 580.0,
        };

        let target_x = if match self.side {
            PlayerSide::Left => ball_pos.x < NET_POSITION_X + 50.0,
            PlayerSide::Right => ball_pos.x > NET_POSITION_X - 50.0,
        } {
            ball_pos.x
        } else {
            home_x
        };

        // Slightly sluggish positioning
        if my_pos.x < target_x - 14.0 {
            input.right = true;
        } else if my_pos.x > target_x + 14.0 {
            input.left = true;
        }

        // Jump if ball is close and above head
        let dx = (my_pos.x - ball_pos.x).abs();
        if dx < 40.0 && ball_pos.y > 300.0 && ball_pos.y < 420.0 && physics.ball.vel.y > 0.0 {
            input.up = true;
        }
    }

    fn update_pro(
        &mut self,
        physics: &PhysicWorld,
        input: &mut PlayerInput,
        is_my_court: bool,
    ) {
        let my_pos = physics.blobs[self.side.index()].pos;
        let ball_pos = physics.ball.pos;
        let ball_vel = physics.ball.vel;

        let default_home = match self.side {
            PlayerSide::Left => 200.0,
            PlayerSide::Right => 600.0,
        };

        // Estimate where ball reaches head height (y = 390.0)
        let impact_opt = estimate_ball_trajectory(ball_pos, ball_vel, 395.0);

        if let Some((target_x, _est_vx, time_to_impact)) = impact_opt {
            let court_bound_ok = match self.side {
                PlayerSide::Left => target_x < NET_POSITION_X - 25.0,
                PlayerSide::Right => target_x > NET_POSITION_X + 25.0,
            };

            if is_my_court || court_bound_ok {
                let dist_x = (my_pos.x - target_x).abs();
                let _time_to_reach = dist_x / BLOBBY_SPEED;

                // Move towards target
                let offset = match self.side {
                    PlayerSide::Left => -15.0,
                    PlayerSide::Right => 15.0,
                };
                let aim_x = (target_x + offset).clamp(
                    match self.side {
                        PlayerSide::Left => 30.0,
                        PlayerSide::Right => NET_POSITION_X + 35.0,
                    },
                    match self.side {
                        PlayerSide::Left => NET_POSITION_X - 35.0,
                        PlayerSide::Right => RIGHT_PLANE - 30.0,
                    },
                );

                if my_pos.x < aim_x - 6.0 {
                    input.right = true;
                } else if my_pos.x > aim_x + 6.0 {
                    input.left = true;
                }

                // Jump timing: jump if in position and ball is falling
                if dist_x < 45.0 && time_to_impact < 28.0 && ball_vel.y > 0.5 {
                    input.up = true;
                }
                return;
            }
        }

        // Return to home base
        if (my_pos.x - default_home).abs() > 15.0 {
            if my_pos.x < default_home {
                input.right = true;
            } else {
                input.left = true;
            }
        }
    }

    fn update_master(
        &mut self,
        physics: &PhysicWorld,
        input: &mut PlayerInput,
        is_my_court: bool,
    ) {
        let my_pos = physics.blobs[self.side.index()].pos;
        let ball_pos = physics.ball.pos;
        let ball_vel = physics.ball.vel;

        let home_x = match self.side {
            PlayerSide::Left => 180.0,
            PlayerSide::Right => 620.0,
        };

        // Try High Play: spike at peak jump (y ~ 240)
        let high_dest_y = 250.0;
        let low_dest_y = 390.0;

        let high_est = estimate_ball_trajectory(ball_pos, ball_vel, high_dest_y);
        let low_est = estimate_ball_trajectory(ball_pos, ball_vel, low_dest_y);

        // Can we do a high smash?
        if let Some((target_x, est_vx, timeto)) = high_est {
            let on_my_side = match self.side {
                PlayerSide::Left => target_x < NET_POSITION_X - 30.0 && target_x > 40.0,
                PlayerSide::Right => target_x > NET_POSITION_X + 30.0 && target_x < RIGHT_PLANE - 40.0,
            };

            let dist_to_target = (my_pos.x - target_x).abs();
            let time_needed = dist_to_target / BLOBBY_SPEED + 22.0;

            if on_my_side && (self.mode_lock || timeto > time_needed) {
                self.mode_lock = timeto < 30.0;

                // Position behind ball to spike it forwards across the net
                let spike_offset = match self.side {
                    PlayerSide::Left => -35.0,
                    PlayerSide::Right => 35.0,
                };
                let high_target = target_x + spike_offset - est_vx * 2.0;

                if my_pos.x < high_target - 5.0 {
                    input.right = true;
                } else if my_pos.x > high_target + 5.0 {
                    input.left = true;
                }

                // Jump to smash
                if timeto < 26.0 && dist_to_target < 65.0 {
                    input.up = true;
                }
                return;
            }
        }

        self.mode_lock = false;

        // Low play: clean return
        if let Some((target_x, _est_vx, timeto)) = low_est {
            let on_my_side = match self.side {
                PlayerSide::Left => target_x < NET_POSITION_X - 15.0,
                PlayerSide::Right => target_x > NET_POSITION_X + 15.0,
            };

            if on_my_side || is_my_court {
                let hit_offset = match self.side {
                    PlayerSide::Left => -12.0,
                    PlayerSide::Right => 12.0,
                };
                let play_x = target_x + hit_offset;

                if my_pos.x < play_x - 5.0 {
                    input.right = true;
                } else if my_pos.x > play_x + 5.0 {
                    input.left = true;
                }

                // Panic jump if ball is low and close to ground
                if timeto < 12.0 && (my_pos.x - target_x).abs() < 40.0 {
                    input.up = true;
                }
                return;
            }
        }

        // Return to defensive base
        if (my_pos.x - home_x).abs() > 12.0 {
            if my_pos.x < home_x {
                input.right = true;
            } else {
                input.left = true;
            }
        }
    }
}

/// Fast numerical raymarch / trajectory simulation matching Blobby Volley ball physics
pub fn estimate_ball_trajectory(
    mut pos: Vec2,
    mut vel: Vec2,
    target_y: f32,
) -> Option<(f32, f32, f32)> {
    let max_ticks = 180;

    for tick in 0..max_ticks {
        // Move ball ds = a/2 + v
        pos += Vec2::new(0.0, 0.5 * BALL_GRAVITATION) + vel;
        vel.y += BALL_GRAVITATION;

        // Left / right wall reflections
        if pos.x - BALL_RADIUS <= LEFT_PLANE && vel.x < 0.0 {
            vel.x = -vel.x;
            pos.x = LEFT_PLANE + BALL_RADIUS;
        } else if pos.x + BALL_RADIUS >= RIGHT_PLANE && vel.x > 0.0 {
            vel.x = -vel.x;
            pos.x = RIGHT_PLANE - BALL_RADIUS;
        }

        // Net vertical stem reflection
        if pos.y > NET_SPHERE_POSITION && (pos.x - NET_POSITION_X).abs() < BALL_RADIUS + NET_RADIUS {
            let right = pos.x - NET_POSITION_X > 0.0;
            vel.x = -vel.x;
            pos.x = NET_POSITION_X + if right { BALL_RADIUS + NET_RADIUS } else { -(BALL_RADIUS + NET_RADIUS) };
        }

        // Check if ball crossed target_y while moving downwards
        if pos.y >= target_y && vel.y > 0.0 {
            return Some((pos.x, vel.x, tick as f32));
        }

        // Ground hit -> ball dead
        if pos.y + BALL_RADIUS >= GROUND_PLANE_HEIGHT_MAX {
            break;
        }
    }

    None
}
