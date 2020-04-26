use crate::movegen::movedefs::Move;

// This is simply a struct that collects all the variables holding the game sate.
// It makes it very easy to make a backup of the game state during make(), and
// restore it when performing unmake(). It prevents having to backup and restore
// each game state variable one by one.

#[derive(Clone, Copy)]
pub struct GameState {
    pub active_color: u8,
    pub castling: u8,
    pub halfmove_clock: u8,
    pub en_passant: Option<u8>,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub this_move: Move,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            active_color: 0,
            castling: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 0,
            zobrist_key: 0,
            this_move: Move { data: 0 },
        }
    }
}