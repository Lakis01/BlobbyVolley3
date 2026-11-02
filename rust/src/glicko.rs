// Blobby Volley 3 - Glicko-2 Rating & Skill-Based Matchmaking System
// Implements the official Glicko-2 algorithm by Dr. Mark E. Glickman
use serde::{Deserialize, Serialize};
use crate::ai::AiDifficulty;

const SCALE: f64 = 173.7178;
const TAU: f64 = 0.5; // System constant constraining volatility change (0.3 - 1.2)
const EPSILON: f64 = 0.000001;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlickoRating {
    pub rating: f64,      // r: rating (default 1500.0)
    pub deviation: f64,   // RD: rating deviation (default 350.0, unrated)
    pub volatility: f64,  // sigma: rating volatility (default 0.06)
}

impl Default for GlickoRating {
    fn default() -> Self {
        Self {
            rating: 1500.0,
            deviation: 350.0,
            volatility: 0.06,
        }
    }
}

impl GlickoRating {
    #[allow(dead_code)]
    pub fn new(rating: f64, deviation: f64, volatility: f64) -> Self {
        Self {
            rating,
            deviation,
            volatility,
        }
    }

    /// Competitive rank title based on Glicko-2 rating
    pub fn rank_title(&self) -> &'static str {
        if self.rating < 1100.0 {
            "Bronze I"
        } else if self.rating < 1250.0 {
            "Bronze II"
        } else if self.rating < 1400.0 {
            "Silver I"
        } else if self.rating < 1550.0 {
            "Silver II"
        } else if self.rating < 1700.0 {
            "Gold I"
        } else if self.rating < 1850.0 {
            "Gold II"
        } else if self.rating < 2000.0 {
            "Platinum I"
        } else if self.rating < 2150.0 {
            "Platinum II"
        } else if self.rating < 2300.0 {
            "Diamond"
        } else {
            "Grandmaster"
        }
    }

    /// Calibrated baseline ratings for offline AI difficulty tiers
    pub fn ai_baseline(difficulty: AiDifficulty) -> (f64, f64) {
        match difficulty {
            AiDifficulty::Novice => (1000.0, 100.0),
            AiDifficulty::Amateur => (1250.0, 80.0),
            AiDifficulty::Regular => (1500.0, 60.0),
            AiDifficulty::Master => (1750.0, 50.0),
            AiDifficulty::Pro => (2000.0, 40.0),
        }
    }

    /// Computes Glicko-2 rating update after a single head-to-head match
    /// score: 1.0 for Win, 0.0 for Loss, 0.5 for Draw
    pub fn update_match(&self, opponent_rating: f64, opponent_rd: f64, score: f64) -> GlickoRating {
        // Step 2: Convert to Glicko-2 scale
        let mu = (self.rating - 1500.0) / SCALE;
        let phi = self.deviation / SCALE;
        let mu_j = (opponent_rating - 1500.0) / SCALE;
        let phi_j = opponent_rd / SCALE;

        // Step 3: Compute g(phi_j) and E(mu, mu_j, phi_j)
        let g_j = 1.0 / (1.0 + 3.0 * phi_j.powi(2) / (std::f64::consts::PI.powi(2))).sqrt();
        let e_j = 1.0 / (1.0 + (-g_j * (mu - mu_j)).exp());

        // Step 3 & 4: Compute estimated variance v and delta
        let v = 1.0 / (g_j.powi(2) * e_j * (1.0 - e_j));
        let delta = v * g_j * (score - e_j);

        // Step 5: Determine new volatility sigma' using Illinois algorithm
        let a = (self.volatility.powi(2)).ln();
        let f = |x: f64| -> f64 {
            let ex = x.exp();
            let num = ex * (delta.powi(2) - phi.powi(2) - v - ex);
            let den = 2.0 * (phi.powi(2) + v + ex).powi(2);
            (num / den) - (x - a) / TAU.powi(2)
        };

        let mut big_a = a;
        let mut big_b = if delta.powi(2) > phi.powi(2) + v {
            (delta.powi(2) - phi.powi(2) - v).ln()
        } else {
            let mut k = 1.0;
            while f(a - k * TAU) < 0.0 {
                k += 1.0;
            }
            a - k * TAU
        };

        let mut fa = f(big_a);
        let mut fb = f(big_b);

        while (big_b - big_a).abs() > EPSILON {
            let c = big_a + (big_a - big_b) * fa / (fb - fa);
            let fc = f(c);
            if fc * fb < 0.0 {
                big_a = big_b;
                fa = fb;
            } else {
                fa /= 2.0;
            }
            big_b = c;
            fb = fc;
        }

        let sigma_prime = (big_a / 2.0).exp();

        // Step 6: Update rating deviation to pre-rating period value
        let phi_star = (phi.powi(2) + sigma_prime.powi(2)).sqrt();

        // Step 7: Update rating and RD to new values
        let phi_prime = 1.0 / ((1.0 / phi_star.powi(2)) + (1.0 / v)).sqrt();
        let mu_prime = mu + phi_prime.powi(2) * g_j * (score - e_j);

        // Step 8: Convert back to standard Glicko scale
        let r_prime = SCALE * mu_prime + 1500.0;
        let rd_prime = (SCALE * phi_prime).clamp(30.0, 350.0);

        GlickoRating {
            rating: r_prime.round(),
            deviation: rd_prime.round(),
            volatility: sigma_prime,
        }
    }

    /// Checks if two players' ratings are within an acceptable matchmaking tolerance window
    #[allow(dead_code)]
    pub fn is_match_acceptable(rating_a: f64, rating_b: f64, max_diff: f64) -> bool {
        (rating_a - rating_b).abs() <= max_diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glicko_win_and_loss() {
        let player = GlickoRating::new(1500.0, 200.0, 0.06);
        // Win against equal opponent
        let after_win = player.update_match(1500.0, 200.0, 1.0);
        assert!(after_win.rating > 1500.0, "Rating must increase after a win");
        assert!(after_win.deviation < 200.0, "RD must decrease after playing a match");

        // Loss against equal opponent
        let after_loss = player.update_match(1500.0, 200.0, 0.0);
        assert!(after_loss.rating < 1500.0, "Rating must decrease after a loss");
        assert!(after_loss.deviation < 200.0, "RD must decrease after playing a match");
    }

    #[test]
    fn test_rank_titles() {
        let bronze = GlickoRating::new(1050.0, 100.0, 0.06);
        assert_eq!(bronze.rank_title(), "Bronze I");
        let silver = GlickoRating::new(1450.0, 100.0, 0.06);
        assert_eq!(silver.rank_title(), "Silver II");
        let gold = GlickoRating::new(1750.0, 100.0, 0.06);
        assert_eq!(gold.rank_title(), "Gold II");
        let plat = GlickoRating::new(2050.0, 100.0, 0.06);
        assert_eq!(plat.rank_title(), "Platinum II");
        let gm = GlickoRating::new(2400.0, 100.0, 0.06);
        assert_eq!(gm.rank_title(), "Grandmaster");
    }

    #[test]
    fn test_ai_baselines() {
        assert_eq!(GlickoRating::ai_baseline(AiDifficulty::Novice), (1000.0, 100.0));
        assert_eq!(GlickoRating::ai_baseline(AiDifficulty::Amateur), (1250.0, 80.0));
        assert_eq!(GlickoRating::ai_baseline(AiDifficulty::Regular), (1500.0, 60.0));
        assert_eq!(GlickoRating::ai_baseline(AiDifficulty::Master), (1750.0, 50.0));
        assert_eq!(GlickoRating::ai_baseline(AiDifficulty::Pro), (2000.0, 40.0));
    }

    #[test]
    fn test_match_acceptability() {
        assert!(GlickoRating::is_match_acceptable(1500.0, 1550.0, 100.0));
        assert!(!GlickoRating::is_match_acceptable(1500.0, 1650.0, 100.0));
        assert!(GlickoRating::is_match_acceptable(1500.0, 1650.0, 160.0));
    }
}
