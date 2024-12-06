use crate::{bitboard::*, chess_move::*, gamestate::Gamestate, move_generation::generate_legal_moves, move_list::MoveList, piece::*};
use std::collections::HashMap;

pub struct Board {
    pieces_bb: [[u64; 6]; 2],
    colors_bb: [u64; 2],
    color_to_move: PieceColor,
    piece_matrix: [Option<Piece>; 64],
    legal_moves: MoveList,
    pub current_gamestate: Gamestate,
    gamestate_stack: Vec<Gamestate>,
}

impl Board {
    pub fn new(fen_string: &str) -> Board {
        let char_to_type: HashMap<char, PieceType> = HashMap::from([
            ('r', PieceType::Rook),
            ('n', PieceType::Knight),
            ('b', PieceType::Bishop),
            ('q', PieceType::Queen),
            ('k', PieceType::King),
            ('p', PieceType::Pawn),
        ]); // CONST

        let mut bb_pieces: [[u64; 6]; 2] = [[0; 6]; 2];
        let fen_string_splits: Vec<&str> = fen_string.split(' ').collect();

        let mut rank: u64 = 0;
        let mut file: u64 = 0;
        for c in fen_string_splits[0].chars() {
            match c {
                '/' => {
                    if file != 8 {
                        panic!("Error in fen string parsing")
                    }

                    rank += 1;
                    file = 0;
                }
                '1'..='8' => file += c.to_digit(10).unwrap() as u64,
                'a'..='z' => {
                    let bb_index = (7 - rank) * 8 + file;
                    let piece_type = char_to_type[&c];
                    bb_pieces[PieceColor::Black][piece_type].set_square(bb_index);
                    file += 1;
                }
                'A'..='Z' => {
                    let bb_index = (7 - rank) * 8 + file;
                    let piece_type = char_to_type[&c.to_ascii_lowercase()];
                    bb_pieces[PieceColor::White][piece_type].set_square(bb_index);
                    file += 1;
                }
                _ => {}
            };
        }

        let bb_colors = [
            bb_pieces[0].iter().copied().fold(0, |acc, bb| acc | bb),
            bb_pieces[1].iter().copied().fold(0, |acc, bb| acc | bb),
        ];

        let color_to_move = Self::read_color_to_move(fen_string_splits[1]);

        let mut piece_matrix = [None; 64];
        for (color_index, bb_ar) in bb_pieces.iter().enumerate() {
            for (piece_index, bb) in bb_ar.iter().enumerate() {
                let mut bb_copy = *bb;
                while bb_copy != 0 {
                    let index = bb_copy.bitscan_reset();
                    piece_matrix[index as usize] = Some(Piece::new(PieceType::from(piece_index), PieceColor::from(color_index)));
                }
            }
        }

        let mut res = Board {
            pieces_bb: bb_pieces,
            colors_bb: bb_colors,
            color_to_move,
            piece_matrix,
            legal_moves: MoveList::new(),
            current_gamestate: Gamestate::new(fen_string_splits[2], fen_string_splits[3]),
            gamestate_stack: Vec::with_capacity(50),
        };
        res.generate_legal_moves();

        res
    }

    fn read_color_to_move(fen_string: &str) -> PieceColor {
        match String::from(fen_string).chars().nth(0).unwrap() {
            'w' => PieceColor::White,
            'b' => PieceColor::Black,
            _ => panic!("color not found in fen string"),
        }
    }

    #[inline]
    pub fn get_pieces_bb(&self) -> [[u64; 6]; 2] {
        self.pieces_bb
    }

    #[inline]
    pub fn get_us_enemy_colors_bb(&self, us_color: PieceColor) -> (u64, u64) {
        (self.colors_bb[us_color], self.colors_bb[!us_color])
    }

    #[inline]
    pub fn get_color_to_move(&self) -> PieceColor {
        self.color_to_move
    }

    /// THIS METHOD CHANGES COLOR_TO_MOVE
    pub fn make_move(&mut self, move_to_make: Move) {
        let start_index = move_to_make.get_from();
        let land_index = move_to_make.get_to();
        let moved_piece = move_to_make.get_moved_piece();
        let moved_color = moved_piece.get_color();
        let moved_type = moved_piece.get_type();
        let enemy_color = !moved_piece.get_color();

        self.gamestate_stack.push(self.current_gamestate); // push old gamestate

        self.current_gamestate.last_piece_captured = None;
        self.current_gamestate.enpassant_square = None;

        self.pieces_bb[moved_color][moved_type].toggle_squares(start_index, land_index);
        self.colors_bb[moved_color].toggle_squares(start_index, land_index);

        // change castling rights
        if moved_type == PieceType::King {
            self.current_gamestate.remove_castle_kingside(moved_color);
            self.current_gamestate.remove_castle_queenside(moved_color);
        } else if moved_type == PieceType::Rook {
            if start_index == 0 || start_index == 56 {
                self.current_gamestate.remove_castle_queenside(moved_color);
            } else if start_index == 7 || start_index == 63 {
                self.current_gamestate.remove_castle_kingside(moved_color);
            }
        } else if move_to_make.is_promotion() {
            let promotion_type = move_to_make.get_promotion_type();
            self.pieces_bb[moved_color][moved_type].toggle_square(land_index);
            self.pieces_bb[moved_color][promotion_type].toggle_square(land_index);
            self.piece_matrix[start_index as usize] = Some(Piece::new(promotion_type, moved_color));
        }

        // double pawn push so change en passant target square
        if start_index.abs_diff(land_index) == 16 && moved_type == PieceType::Pawn {
            match moved_color {
                PieceColor::White => self.current_gamestate.enpassant_square = Some(start_index + 8),
                PieceColor::Black => self.current_gamestate.enpassant_square = Some(start_index - 8),
            }
        }
        // en passant
        else if move_to_make.is_enpassant() {
            let enemy_pawn_index = match moved_color {
                PieceColor::White => land_index - 8,
                PieceColor::Black => land_index + 8,
            };

            self.pieces_bb[enemy_color][PieceType::Pawn].toggle_square(enemy_pawn_index);
            self.colors_bb[enemy_color].toggle_square(enemy_pawn_index);
            self.piece_matrix[enemy_pawn_index as usize] = None;
            self.current_gamestate.last_piece_captured = Some(Piece::new(PieceType::Pawn, enemy_color));
        }
        // capture
        else if let Some(captured_piece) = self.get_piece_at(land_index) {
            self.pieces_bb[enemy_color][captured_piece.get_type()].toggle_square(land_index);
            self.colors_bb[enemy_color].toggle_square(land_index);
            self.current_gamestate.last_piece_captured = Some(captured_piece);
        } else if move_to_make.is_castle_kingside() {
            let rook_from = start_index + 3;
            let rook_to = start_index + 1;

            self.pieces_bb[moved_color][PieceType::Rook].toggle_squares(rook_from, rook_to);
            self.colors_bb[moved_color].toggle_squares(rook_from, rook_to);
            self.piece_matrix[rook_to as usize] = self.piece_matrix[rook_from as usize].take();
        } else if move_to_make.is_castle_queenside() {
            let rook_from = start_index - 4;
            let rook_to = start_index - 1;

            self.pieces_bb[moved_color][PieceType::Rook].toggle_squares(rook_from, rook_to);
            self.colors_bb[moved_color].toggle_squares(rook_from, rook_to);
            self.piece_matrix[rook_to as usize] = self.piece_matrix[rook_from as usize].take();
        }

        self.piece_matrix[land_index as usize] = self.piece_matrix[start_index as usize].take();
        self.color_to_move = !self.color_to_move;
    }

    /// THIS METHOD CHANGES COLOR_TO_MOVE
    pub fn unmake_move(&mut self, move_to_unmake: Move) {
        let start_index = move_to_unmake.get_from();
        let land_index = move_to_unmake.get_to();
        let moved_piece = move_to_unmake.get_moved_piece();
        let moved_color = moved_piece.get_color();

        self.pieces_bb[moved_color][moved_piece.get_type()].toggle_squares(start_index, land_index);
        self.colors_bb[moved_color].toggle_squares(start_index, land_index);
        self.piece_matrix[start_index as usize] = self.piece_matrix[land_index as usize].take();

        if move_to_unmake.is_promotion() {
            self.pieces_bb[moved_color][move_to_unmake.get_promotion_type()].toggle_square(land_index);
            self.pieces_bb[moved_color][PieceType::Pawn].toggle_square(land_index);
            self.piece_matrix[start_index as usize] = Some(Piece::new(PieceType::Pawn, moved_color));
        }

        if move_to_unmake.is_enpassant() {
            let enemy_pawn_index = match moved_color {
                PieceColor::White => land_index - 8,
                PieceColor::Black => land_index + 8,
            };

            let enemy_color = !moved_color;
            self.pieces_bb[enemy_color][PieceType::Pawn].toggle_square(enemy_pawn_index);
            self.colors_bb[enemy_color].toggle_square(enemy_pawn_index);
            self.piece_matrix[enemy_pawn_index as usize] = Some(Piece::new(PieceType::Pawn, enemy_color));
        } else if let Some(captured_piece) = self.current_gamestate.get_last_piece_captured() {
            let enemy_color = !moved_color;
            self.pieces_bb[enemy_color][captured_piece.get_type()].toggle_square(land_index);
            self.colors_bb[enemy_color].toggle_square(land_index);
            self.piece_matrix[land_index as usize] = self.current_gamestate.get_last_piece_captured();
        } else if move_to_unmake.is_castle_kingside() {
            let rook_from = start_index + 3;
            let rook_to = start_index + 1;

            self.pieces_bb[moved_color][PieceType::Rook].toggle_squares(rook_from, rook_to);
            self.colors_bb[moved_color].toggle_squares(rook_from, rook_to);
            self.piece_matrix[rook_from as usize] = self.piece_matrix[rook_to as usize].take();
        } else if move_to_unmake.is_castle_queenside() {
            let rook_from = start_index - 4;
            let rook_to = start_index - 1;

            self.pieces_bb[moved_color][PieceType::Rook].toggle_squares(rook_from, rook_to);
            self.colors_bb[moved_color].toggle_squares(rook_from, rook_to);
            self.piece_matrix[rook_from as usize] = self.piece_matrix[rook_to as usize].take();
        }
        self.current_gamestate = self.gamestate_stack.pop().unwrap();
        self.color_to_move = !self.color_to_move;
    }

    #[inline]
    pub fn get_legal_moves(&self) -> &MoveList {
        &self.legal_moves
    }

    pub fn generate_legal_moves(&mut self) {
        self.legal_moves = generate_legal_moves(self);
    }

    #[inline]
    pub fn get_piece_at(&self, index: u64) -> Option<Piece> {
        self.piece_matrix[index as usize]
    }

    #[inline]
    pub fn get_us_enemy_bitboards(&self, color_to_move: PieceColor) -> ([u64; 6], [u64; 6], u64, u64) {
        (
            self.pieces_bb[color_to_move],
            self.pieces_bb[!color_to_move],
            self.colors_bb[color_to_move],
            self.colors_bb[!color_to_move],
        )
    }
}
