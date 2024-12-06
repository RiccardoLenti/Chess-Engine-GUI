use crate::{Piece, PieceType};

#[derive(Clone, Copy, Debug)]
pub struct Move {
    move_code: u16,
    moved_piece: Piece,
}

impl Move {
    #[inline]
    pub fn new(start_square: u64, land_square: u64, piece: Piece) -> Move {
        Move {
            move_code: ((land_square << 6) + start_square) as u16,
            moved_piece: piece,
        }
    }

    #[inline]
    pub fn get_from(self) -> u64 {
        (self.move_code & 0x3f) as u64
    }

    #[inline]
    pub fn get_to(self) -> u64 {
        ((self.move_code >> 6) & 0x3f) as u64
    }

    #[inline]
    pub fn get_moved_piece(self) -> Piece {
        self.moved_piece
    }

    #[inline]
    pub fn add_enpassant(&mut self) {
        self.move_code += 8192;
    }

    #[inline]
    pub fn is_enpassant(self) -> bool {
        !self.is_promotion() && (self.move_code >> 13) & 1 == 1
    }

    #[inline]
    pub fn add_castle_kingside(&mut self) {
        self.move_code += 16384;
    }

    #[inline]
    pub fn is_castle_kingside(self) -> bool {
        !self.is_promotion() && (self.move_code >> 14) & 1 == 1
    }

    #[inline]
    pub fn add_castle_queenside(&mut self) {
        self.move_code += 32768;
    }

    #[inline]
    pub fn is_castle_queenside(self) -> bool {
        !self.is_promotion() && (self.move_code >> 15) & 1 == 1
    }
    
    #[inline]
    pub fn add_promotion(&mut self, piece_to_promote_to: PieceType) {
        self.move_code += 4096;
        self.move_code += (piece_to_promote_to as u16) << 13;
    }

    #[inline]
    pub fn is_promotion(self) -> bool {
        (self.move_code >> 12) & 1 == 1
    }

    #[inline]
    pub fn get_promotion_type(self) -> PieceType {
        PieceType::from(((self.move_code >> 13) & 7) as u8)
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        (self.move_code & 8191) == (other.move_code & 8191)
            && (!self.is_promotion() || (self.get_promotion_type() == other.get_promotion_type()))
    }
}
