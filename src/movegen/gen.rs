/**
 * The movegen.rs module is the part of the engine that generates chess moves to be searched
 * and evaluate later, in the search and evaluation modules of the program. Note that the move
 * generator is pseudo-legal: that means that it generates all possible moves, regardless if
 * they leave the king in check after that move. The reason is twofold:
 *
 * 1. This speeds up move generation, as _MOST_ moves will be legal. Checking every move for
 * legality would greatly slow down the move generation process.
 * 2. The search might decide to focus only on a subset of moves and discard the rest. Those
 * discarded moves will not be executed or evaluated. If legality checking had been done on
 * those moves, that time would have been wasted.
 */
use super::information::square_attacked;
use super::movedefs::{Move, MoveList, Shift};
use super::MoveGenerator;
use crate::board::representation::Board;
use crate::board::square_on_rank;
use crate::defs::{
    Bitboard, Piece, B1, B8, BISHOP, BLACK, C1, C8, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, D1,
    D8, E1, E8, F1, F8, G1, G8, KING, KNIGHT, PAWN, PNONE, QUEEN, RANK_1, RANK_4, RANK_5, RANK_8,
    ROOK, WHITE,
};

/**
 * This function actually generates the moves, using other functions in this module.
 * It takes the following parameters:
 *
 * board: a reference to the board/position
 * side: the side to move
 * mg: The movegenerator, which provides all possible piece attacks on all squares.
 *          It uses precalculated moves for each piece on each square, so the move
 *          generator does not have to calculate this over and aover again.
 * list: a mutable reference to a list that will contain the moves.
*/
pub fn all_moves(board: &Board, mg: &MoveGenerator, list: &mut MoveList) {
    list.clear();
    piece(KING, board, mg, list);
    piece(KNIGHT, board, mg, list);
    piece(ROOK, board, mg, list);
    piece(BISHOP, board, mg, list);
    piece(QUEEN, board, mg, list);
    pawns(board, mg, list);
    castling(board, mg, list);
}

/**
 * Generates moves for pieces.
 * Basically:
 * - It gets the "from" square.
 * - It gets all the targets for the piece from the Movements object.
 * - The piece can move to all squares that do not contain our own pieces.
 * - Add those moves to the move list.
 */
fn piece(piece: Piece, board: &Board, mg: &MoveGenerator, list: &mut MoveList) {
    let side = board.active_color as usize;
    let bb_occupancy = board.occupancy();
    let bb_own_pieces = board.bb_pieces[side];
    let mut bb_pieces = board.get_pieces(piece, side);
    while bb_pieces > 0 {
        let from = next(&mut bb_pieces);
        let bb_target = match piece {
            QUEEN | ROOK | BISHOP => mg.get_slider_attacks(piece, from, bb_occupancy),
            KING | KNIGHT => mg.get_non_slider_attacks(piece, from),
            _ => 0,
        };
        let bb_moves = bb_target & !bb_own_pieces;
        add_move(board, piece, from, bb_moves, list);
    }
}

/**
 * Pawns are a bit more complicated, because their attacks and moves are different,
 * but also because they have en-passant and promotion capabilities.
 * It works as such:
 * - Get the "from" square for each pawn.
 * - Push the pawn forward one rank.
 * - If the destination square is empty, "one_step" contains a move. Otherwise, it's 0.
 * - Two_step is a pawn moving two steps. It contains a move if:
 *      * One_step also contains a move
 *      * The next rank is empty
 *      * and the next rank is the fourth (from either WHITE or BLACK's point of view).
 * Then the capture moves are generated; same way as the king/knight moves.
 * An en_passant capture is generated, if the en_passant square in the board position is set,
 * and if the pawn currently being investigated has this square as an attack target.
 * Combine all the moves, and add them to the move list. The add_move function will take care
 * of promotions, adding four possible moves (Q, R, B, and N) to the list instead of one move.
 */
fn pawns(board: &Board, mg: &MoveGenerator, list: &mut MoveList) {
    let side = board.active_color as usize;
    let direction = if side == WHITE { 8 } else { -8 };
    let bb_opponent_pieces = board.bb_pieces[side ^ 1];
    let bb_empty = !board.occupancy();
    let bb_fourth = if side == WHITE {
        board.bb_ranks[RANK_4]
    } else {
        board.bb_ranks[RANK_5]
    };
    let mut bb_pawns = board.get_pieces(PAWN, side);
    while bb_pawns > 0 {
        let from = next(&mut bb_pawns);
        let bb_push = 1u64 << (from as i8 + direction);
        let bb_one_step = bb_push & bb_empty;
        let bb_two_step = bb_one_step.rotate_left((64 + direction) as u32) & bb_empty & bb_fourth;
        let bb_targets = mg.get_pawn_attacks(side, from);
        let bb_captures = bb_targets & bb_opponent_pieces;
        let bb_ep_capture = if let Some(ep) = board.en_passant {
            bb_targets & (1u64 << ep)
        } else {
            0
        };
        let moves = bb_one_step | bb_two_step | bb_captures | bb_ep_capture;
        add_move(board, PAWN, from, moves, list);
    }
}

/** The castling function is long, but fortunately not hard to understand.
 * The length is due to having four parts; each side can castle either kingside or queenside.
 * Step by step description:
 * First, determine the opponent, which is "not our side".
 * "has_castling_rights" is checked against the board, for either white or black.
 * "in_check" is either checked for white or black.
 * Then there are two big parts: one for white castling, and one for black castling.
 * A part can be executed, if the side is correct for that part, the side has at least one
 * castling right, and the king is not in check.
 * Inside the part, we try to either castle kingside or queenside. To be able to determine
 * if castling is possible, we first determine if there are any blocking pieces between the
 * king and the rook of the side we're castling to. We also check if the square directly next
 * to the king is not attacked; it's not permitted to castle across check. If there are no
 * blockers and the squares just next to the king are not attacked, castling is possible.
 * Note: we MUST verify if the king does not castle ACROSS check. We DON'T verify if the king
 * castles INTO check on the landing square (gi, c1, g8 or c8). This verification is left up
 * to makemove/unmake move outside of the move generator.
 */
fn castling(board: &Board, mg: &MoveGenerator, list: &mut MoveList) {
    let side = board.active_color as usize;
    let opponent = side ^ 1;
    let has_castling_rights = if side == WHITE {
        (board.castling & (CASTLE_WK + CASTLE_WQ)) > 0
    } else {
        (board.castling & (CASTLE_BK + CASTLE_BQ)) > 0
    };
    let in_check = if side == WHITE {
        square_attacked(board, opponent, mg, E1)
    } else {
        square_attacked(board, opponent, mg, E8)
    };

    if side == WHITE && has_castling_rights && !in_check {
        let mut bb_king = board.get_pieces(KING, side);
        let from = next(&mut bb_king);
        let bb_occupancy = board.occupancy();

        // Kingside
        if board.castling & CASTLE_WK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << F1) | (1u64 << G1);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;
            let f1_attacked = square_attacked(board, opponent, mg, F1);

            if !is_kingside_blocked && !f1_attacked {
                let to = (1u64 << from) << 2;
                add_move(board, KING, from, to, list);
            }
        }

        // Queenside
        if board.castling & CASTLE_WQ > 0 {
            let bb_queenside_blockers: u64 = (1u64 << B1) | (1u64 << C1) | (1u64 << D1);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;
            let d1_attacked = square_attacked(board, opponent, mg, D1);

            if !is_queenside_blocked && !d1_attacked {
                let to = (1u64 << from) >> 2;
                add_move(board, KING, from, to, list);
            }
        }
    }

    if side == BLACK && has_castling_rights && !in_check {
        let mut bb_king = board.get_pieces(KING, side);
        let from = next(&mut bb_king);
        let bb_occupancy = board.occupancy();

        // Kingside
        if board.castling & CASTLE_BK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << F8) | (1u64 << G8);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;
            let f8_attacked = square_attacked(board, opponent, mg, F8);

            if !is_kingside_blocked && !f8_attacked {
                let to = (1u64 << from) << 2;
                add_move(board, KING, from, to, list);
            }
        }

        // Queenside
        if board.castling & CASTLE_BQ > 0 {
            let bb_queenside_blockers: u64 = (1u64 << B8) | (1u64 << C8) | (1u64 << D8);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;
            let d8_attacked = square_attacked(board, opponent, mg, D8);

            if !is_queenside_blocked && !d8_attacked {
                let to = (1u64 << from) >> 2;
                add_move(board, KING, from, to, list);
            }
        }
    }
}

/**
 * Get the next set bit from a bitboard.
 * This is used to get the square locations of each piece.
 * For example, the PAWNS bitboard could have 8 bits set.
 * This function returns the index (= square) from that bitboard,
 * and then removes the bit. All pieces/squares (whatver is in
 * the bitboard) have been handled when the bitboard becomes 0.
 * */
fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1u64 << location;
    location as u8
}

/** Determine if the move is a capture; this is the case if there's an opponent piece on the
 * target square of the moving piece. If so, return which piece is on the target square. If
 * there is no piece, PNONE (no piece) will will be returned.
 */
fn captured_piece(board: &Board, to_square: u8) -> Piece {
    let side = board.active_color as usize;
    let bb_target_square = 1u64 << (to_square as u64);
    let bb_opponent_pieces = board.bb_pieces[side ^ 1];
    if bb_target_square & bb_opponent_pieces > 0 {
        return board.which_piece(to_square);
    };
    PNONE
}

/** Adds moves and the data belonging to those moves to a move list.
 * This function also takes care of promotions, by adding four moves
 * to the list instead of one; one move for each promotion possibility.
*/
fn add_move(board: &Board, piece: Piece, from: u8, to: Bitboard, list: &mut MoveList) {
    let side = board.active_color as usize;
    let mut bb_to = to;
    let promotion_rank = if side == WHITE {
        RANK_8 as u8
    } else {
        RANK_1 as u8
    };
    let promotion_pieces: [usize; 4] = [QUEEN, ROOK, BISHOP, KNIGHT];

    while bb_to > 0 {
        let to_square = next(&mut bb_to);
        let capture = captured_piece(board, to_square);
        let promotion = (piece == PAWN) && square_on_rank(to_square, promotion_rank);
        let ep_square = if let Some(square) = board.en_passant {
            square
        } else {
            0
        };
        let en_passant = (piece == PAWN) && (ep_square != 0) && (to_square == ep_square);
        let castling = (piece == KING) && ((to_square as i8 - from as i8).abs() == 2);
        let move_data = (piece as u64)
            | ((from as u64) << Shift::FromSq as u64)
            | ((to_square as u64) << Shift::ToSq as u64)
            | ((capture as u64) << Shift::Capture as u64)
            | ((en_passant as u64) << Shift::EnPassant as u64)
            | ((castling as u64) << Shift::Castling as u64);

        if !promotion {
            let m = Move {
                data: move_data | ((PNONE as u64) << Shift::Promotion as u64),
            };
            list.push(m);
        }

        if promotion {
            for piece in promotion_pieces.iter() {
                let m = Move {
                    data: move_data | ((*piece as u64) << Shift::Promotion as u64),
                };
                list.push(m);
            }
        }
    }
}