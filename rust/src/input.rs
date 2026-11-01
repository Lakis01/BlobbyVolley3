// Blobby Volley 3 - Input Management Subsystem
use macroquad::prelude::*;
use crate::physics::PlayerInput;

pub fn read_player1_input(is_single_player: bool) -> PlayerInput {
    let mut input = PlayerInput::default();

    // Standard Player 1 controls (WASD / Space)
    let left = is_key_down(KeyCode::A) || (is_single_player && is_key_down(KeyCode::Left));
    let right = is_key_down(KeyCode::D) || (is_single_player && is_key_down(KeyCode::Right));
    let up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space) || (is_single_player && is_key_down(KeyCode::Up));

    input.left = left;
    input.right = right;
    input.up = up;

    input
}

pub fn read_player2_input() -> PlayerInput {
    let mut input = PlayerInput::default();

    // Player 2 controls (Arrow keys / Enter)
    let left = is_key_down(KeyCode::Left) || is_key_down(KeyCode::J);
    let right = is_key_down(KeyCode::Right) || is_key_down(KeyCode::L);
    let up = is_key_down(KeyCode::Up) || is_key_down(KeyCode::I) || is_key_down(KeyCode::RightShift) || is_key_down(KeyCode::Enter);

    input.left = left;
    input.right = right;
    input.up = up;

    input
}

