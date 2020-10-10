pub mod defs;
pub mod material;
pub mod psqt;

use crate::{board::Board, defs::Sides};

pub fn evaluate_position(board: &Board) -> i16 {
    let w_material = board.game_state.material[Sides::WHITE];
    let b_material = board.game_state.material[Sides::BLACK];

    // Base evaluation, by counting material.
    let mut value = (w_material - b_material) as i16;

    // Add PSQT values
    value += board.game_state.psqt[Sides::WHITE] - board.game_state.psqt[Sides::BLACK];

    value
}
