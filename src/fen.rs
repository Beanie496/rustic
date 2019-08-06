use crate::board::*;
use crate::defs::*;

fn part_0(part: &str, board: &mut Board) {
    const PART: u8 = 0;
    let mut rank = RANK_8;
    let mut file = FILE_A;

    for c in part.chars() {
        let square = (rank * 8) + file;
        match c {
            'k' => board.bb_b[BB_K] += 1 << square,
            'q' => board.bb_b[BB_Q] += 1 << square,
            'r' => board.bb_b[BB_R] += 1 << square,
            'b' => board.bb_b[BB_B] += 1 << square,
            'n' => board.bb_b[BB_N] += 1 << square,
            'p' => board.bb_b[BB_P] += 1 << square,
            'K' => board.bb_w[BB_K] += 1 << square,
            'Q' => board.bb_w[BB_Q] += 1 << square,
            'R' => board.bb_w[BB_R] += 1 << square,
            'B' => board.bb_w[BB_B] += 1 << square,
            'N' => board.bb_w[BB_N] += 1 << square,
            'P' => board.bb_w[BB_P] += 1 << square,
            '1'..='8' => {
                if let Some(x) = c.to_digit(10) {
                    file += x as u8;
                }
            }
            SPLITTER => {
                assert!(file == 8, "FEN {}: Counting incorrect: {}", PART, part);
                rank -= 1;
                file = 0;
            }
            _ => assert!(false, "FEN {}: Illegal character found: {}", PART, part),
        }
        if LIST_OF_PIECES.contains(c) {
            file += 1;
        }
    }
}

fn part_1(part: &str, board: &mut Board) {
    const PART: u8 = 1;
    let mut step = if part.len() == 1 { 1 } else { 0 };

    if step == 1 {
        if let Some(x) = part.chars().next() {
            step += if WHITE_OR_BLACK.contains(x) { 1 } else { 0 };
            match x {
                'w' => board.turn = Color::WHITE,
                'b' => board.turn = Color::BLACK,
                _ => (),
            }
        }
    }
    assert_eq!(step, 2, "FEN {}: Must be 'w' or 'b'. {}", PART, part);
}

fn part_2(part: &str, board: &mut Board) {
    const PART: u8 = 2;
    let length = part.len();
    let mut char_ok = 0;

    if length == 1 {
        if let Some(x) = part.chars().next() {
            if x == DASH || CASTLE_RIGHTS.contains(x) {
                char_ok += 1
            }
        }
    }

    if length > 1 && length <= 4 {
        for c in part.chars() {
            if CASTLE_RIGHTS.contains(c) {
                char_ok += 1;
                match c {
                    'K' => board.castling += CASTLE_WK,
                    'Q' => board.castling += CASTLE_WQ,
                    'k' => board.castling += CASTLE_BK,
                    'q' => board.castling += CASTLE_BQ,
                    _ => (),
                }
            }
        }
    }
    assert_eq!(char_ok, length, "FEN {}: Castling rights: {}", PART, part);
}

fn part_3(part: &str, board: &mut Board) {
    const PART: u8 = 3;
    let length = part.len();
    let mut char_ok = 0;

    if length == 1 {
        if let Some(x) = part.chars().next() {
            if x == DASH {
                char_ok += 1
            }
        }
    }

    if length == 2 {
        const ASCII_VALUE_OF_SMALL_A: i8 = 97;
        const ASCII_VALUE_OF_1: i8 = 49;
        let mut char_nr = 0;
        let mut file = -1;
        let mut rank = -1;
        for c in part.chars() {
            char_nr += 1;
            if char_nr == 1 && LETTERS.contains(c) {
                file = (c as i8) - ASCII_VALUE_OF_SMALL_A;
                char_ok += 1;
            }
            if char_nr == 2 && EN_PASSANT_RANKS.contains(c) {
                rank = (c as i8) - ASCII_VALUE_OF_1;
                char_ok += 1;
            }
        }
        if file != -1 && rank != -1 {
            let square_nr = (rank * 8) + file;
            board.en_passant = square_nr;
        }
    }
    assert_eq!(char_ok, length, "FEN {}: En Passant Target: {}", PART, part);
}

fn part_4(part: &str, board: &mut Board) {
    const PART: u8 = 4;
    let length = part.len();
    let mut is_ok = false;

    if length == 1 || length == 2 {
        match part.parse::<u8>() {
            Ok(x) => {
                if x <= 50 {
                    board.fifty_moves = x;
                    is_ok = true;
                }
            }
            _ => (),
        }
    }
    assert_eq!(is_ok, true, "FEN {}: 50-move count: {}", PART, part);
}

fn part_5(part: &str, board: &mut Board) {
    const PART: u8 = 5;
    let length = part.len();
    let mut is_ok = false;

    if length >= 1 || length <= 4 {
        match part.parse::<u16>() {
            Ok(x) => {
                if x <= MAX_FULL_MOVES {
                    board.full_moves = x;
                    is_ok = true;
                }
            }
            _ => (),
        }
    }
    assert_eq!(is_ok, true, "FEN {}: Full move count: {}", PART, part);
}

pub fn fen_read(fen_string: &str, board: &mut Board) {
    let fen_parts: Vec<String> = fen_string.split(SPACE).map(|s| s.to_string()).collect();
    let fen_parse: [FunctionPointerFenPartHandler; 6] =
        [part_0, part_1, part_2, part_3, part_4, part_5];

    for x in 0..fen_parse.len() {
        fen_parse[x](&fen_parts[x], board);
    }
}