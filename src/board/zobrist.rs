use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::defs::{Piece, Side, EMPTY, NR_OF_SQUARES, WHITE};

const ALL_SQUARES: usize = NR_OF_SQUARES as usize;
const ALL_PIECES: usize = 6;
const ALL_SIDES: usize = 2;
const ALL_CASTLING_PERMISSIONS: usize = 16;

/* Random number for all sides for all pieces on all squares */
type PieceRandoms = [[[u64; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES];
type CastlingRandoms = [u64; ALL_CASTLING_PERMISSIONS];

pub type ZobristKey = u64;

pub struct ZobristRandoms {
    rnd_pieces: PieceRandoms,
    rnd_castling: CastlingRandoms,
    rnd_white: u64,
    rnd_en_passant: u64,
}

impl ZobristRandoms {
    pub fn new() -> ZobristRandoms {
        let mut random = SmallRng::from_seed([128; 16]);
        let mut zobrist_randoms = ZobristRandoms {
            rnd_pieces: [[[EMPTY; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES],
            rnd_castling: [EMPTY; ALL_CASTLING_PERMISSIONS],
            rnd_en_passant: EMPTY,
            rnd_white: EMPTY,
        };

        for side in 0..ALL_SIDES {
            for piece in 0..ALL_PIECES {
                for square in 0..ALL_SQUARES {
                    zobrist_randoms.rnd_pieces[side][piece][square] = random.gen::<u64>();
                }
            }
        }

        for permission in 0..ALL_CASTLING_PERMISSIONS {
            zobrist_randoms.rnd_castling[permission] = random.gen::<u64>();
        }

        zobrist_randoms.rnd_en_passant = random.gen::<u64>();
        zobrist_randoms.rnd_white = random.gen::<u64>();

        zobrist_randoms
    }

    pub fn piece(&self, side: Side, piece: Piece, square: u8) -> u64 {
        self.rnd_pieces[side][piece][square as usize]
    }

    pub fn castling(&self, castling_permissions: u8) -> u64 {
        self.rnd_castling[castling_permissions as usize]
    }

    pub fn white(&self, active_color: u8) -> u64 {
        if (active_color as usize) == WHITE {
            self.rnd_white
        } else {
            0
        }
    }

    pub fn en_passant(&self, en_passant: Option<u8>) -> u64 {
        if let Some(ep) = en_passant {
            self.rnd_en_passant
        } else {
            0
        }
    }
}
