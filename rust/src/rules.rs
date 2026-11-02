// Blobby Volley 3 - Game Rules and Scoring Engine
use crate::physics::{PhysicWorld, PlayerSide};
use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleMode {
    Classic,
    Modern,
}

impl RuleMode {
    pub fn name(self) -> &'static str {
        match self {
            RuleMode::Modern => "Modern (Rally Points)",
            RuleMode::Classic => "Classic (Side-Out)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            RuleMode::Modern => RuleMode::Classic,
            RuleMode::Classic => RuleMode::Modern,
        }
    }

    pub fn prev(self) -> Self {
        self.next()
    }

    pub fn from_index(idx: usize) -> Self {
        if idx == 1 { RuleMode::Classic } else { RuleMode::Modern }
    }

    pub fn to_index(self) -> usize {
        match self {
            RuleMode::Modern => 0,
            RuleMode::Classic => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchStats {
    pub total_rallies: u32,
    pub max_rally_hits: u32,
    pub current_rally_hits: u32,
    pub match_time_seconds: f32,
}

#[derive(Debug, Clone)]
pub struct GameLogic {
    pub rule_mode: RuleMode,
    pub score_to_win: i32,
    pub scores: [i32; 2],
    pub touches: [i32; 2],
    pub squish: [i32; 2],
    pub squish_ground: i32,
    pub squish_wall: i32,
    pub serving_player: PlayerSide,
    pub winning_player: Option<PlayerSide>,
    pub is_ball_valid: bool,
    pub is_game_running: bool,
    pub stats: MatchStats,
    pub round_reset_timer: f32,
}

impl GameLogic {
    pub fn new(rule_mode: RuleMode, score_to_win: i32) -> Self {
        Self {
            rule_mode,
            score_to_win,
            scores: [0, 0],
            touches: [0, 0],
            squish: [0, 0],
            squish_ground: 0,
            squish_wall: 0,
            serving_player: PlayerSide::Left,
            winning_player: None,
            is_ball_valid: true,
            is_game_running: false,
            stats: MatchStats {
                total_rallies: 0,
                max_rally_hits: 0,
                current_rally_hits: 0,
                match_time_seconds: 0.0,
            },
            round_reset_timer: 0.0,
        }
    }

    pub fn reset_match(&mut self) {
        self.scores = [0, 0];
        self.touches = [0, 0];
        self.squish = [0, 0];
        self.squish_ground = 0;
        self.squish_wall = 0;
        self.serving_player = PlayerSide::Left;
        self.winning_player = None;
        self.is_ball_valid = true;
        self.is_game_running = false;
        self.stats.total_rallies = 0;
        self.stats.max_rally_hits = 0;
        self.stats.current_rally_hits = 0;
        self.stats.match_time_seconds = 0.0;
        self.round_reset_timer = 0.0;
    }

    pub fn step(&mut self, dt: f32) {
        if self.is_game_running {
            self.stats.match_time_seconds += dt;
        }

        self.squish[0] = (self.squish[0] - 1).max(0);
        self.squish[1] = (self.squish[1] - 1).max(0);
        self.squish_wall = (self.squish_wall - 1).max(0);
        self.squish_ground = (self.squish_ground - 1).max(0);

        if !self.is_ball_valid {
            self.round_reset_timer += dt;
        }
    }

    pub fn on_ball_hits_blob(&mut self, player: PlayerSide) {
        if self.squish[player.index()] > 0 || !self.is_ball_valid {
            return;
        }

        self.squish[player.index()] = SQUISH_TOLERANCE;
        self.squish[player.opponent().index()] = 0;
        self.is_game_running = true;

        self.touches[player.index()] += 1;
        self.stats.current_rally_hits += 1;
        if self.stats.current_rally_hits > self.stats.max_rally_hits {
            self.stats.max_rally_hits = self.stats.current_rally_hits;
        }

        // More than 3 consecutive touches is a fault!
        if self.touches[player.index()] > 3 {
            self.trigger_mistake(player);
        }

        // Reset other player's touch counter
        self.touches[player.opponent().index()] = 0;
    }

    pub fn on_ball_crosses_net(&mut self, _new_side: PlayerSide) {
        self.touches = [0, 0];
        self.squish = [0, 0];
    }

    pub fn on_ball_hits_ground(&mut self, side: PlayerSide) {
        if self.squish_ground > 0 || !self.is_ball_valid {
            return;
        }
        self.squish_ground = SQUISH_TOLERANCE;
        self.touches[side.opponent().index()] = 0;

        // Ground hit on player's side is a fault for that player
        self.trigger_mistake(side);
    }

    pub fn on_ball_hits_wall(&mut self, _side: PlayerSide) {
        if self.squish_wall > 0 || !self.is_ball_valid {
            return;
        }
        self.squish_wall = SQUISH_TOLERANCE;
    }

    pub fn on_ball_hits_net(&mut self, _side: PlayerSide) {
        if self.squish_wall > 0 || !self.is_ball_valid {
            return;
        }
        self.squish_wall = SQUISH_TOLERANCE;
    }

    fn trigger_mistake(&mut self, mistake_side: PlayerSide) {
        if !self.is_ball_valid {
            return;
        }

        let opponent = mistake_side.opponent();
        let points_awarded = match self.rule_mode {
            RuleMode::Modern => 1,
            RuleMode::Classic => {
                // In classic volleyball: point only if opponent was serving!
                if opponent == self.serving_player {
                    1
                } else {
                    0
                }
            }
        };

        self.scores[opponent.index()] += points_awarded;
        self.is_ball_valid = false;
        self.round_reset_timer = 0.0;
        self.stats.total_rallies += 1;
        self.stats.current_rally_hits = 0;

        // The rally winner serves next
        self.serving_player = opponent;

        self.touches = [0, 0];
        self.squish = [0, 0];
        self.squish_wall = 0;
        self.squish_ground = 0;

        self.winning_player = self.check_win();
    }

    pub fn check_win(&self) -> Option<PlayerSide> {
        let left = self.scores[0];
        let right = self.scores[1];

        if left >= self.score_to_win && left >= right + 2 {
            Some(PlayerSide::Left)
        } else if right >= self.score_to_win && right >= left + 2 {
            Some(PlayerSide::Right)
        } else {
            None
        }
    }

    pub fn can_start_round(&self, physics: &PhysicWorld) -> bool {
        if self.is_ball_valid {
            return false;
        }

        // Must wait at least 0.6 seconds after a point/fault for visual impact
        if self.round_reset_timer < 0.6 {
            return false;
        }

        let ball_settled = physics.blob_hit_ground(self.serving_player)
            && physics.ball.vel.y.abs() < 2.0
            && physics.ball.pos.y > 420.0;

        // Or timeout after 2.0s to avoid getting stuck
        ball_settled || self.round_reset_timer > 2.0
    }

    pub fn on_serve(&mut self) {
        self.is_ball_valid = true;
        self.is_game_running = false;
        self.round_reset_timer = 0.0;
        self.touches = [0, 0];
    }
}

