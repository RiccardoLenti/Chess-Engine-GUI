use std::mem::MaybeUninit;

use crate::{bitboard::Bitmanip, chess_move::Move, consts::CONSTS, move_generation::generate_promotion_moves, Piece};

const MAX_LEGAL_MOVES: u8 = 255;

#[derive(Copy, Clone)]
pub struct MoveList {
    list: [Move; MAX_LEGAL_MOVES as usize],
    size: u8,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            list: unsafe {
                let block = MaybeUninit::uninit();
                block.assume_init()
            },
            size: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, m: Move) {
        self.list[self.size as usize] = m;
        self.size += 1;
    }

    #[inline]
    pub fn len(&self) -> u8 {
        self.size
    }

    pub fn iter(&self) -> MoveListIter {
        MoveListIter {
            movelist: self,
            index: 0,
        }
    }

    pub fn swap(&mut self, index1: usize, index2: usize) {
        unsafe {
            let ptr_a: *mut Move = &mut self.list[index1];
            let ptr_b: *mut Move = &mut self.list[index2];

            std::ptr::swap(ptr_a, ptr_b);
        }
    }

    pub fn append_bb(&mut self, mut bb: u64, from: u64, piece: Piece) {
        while bb != 0 {
            let to = bb.bitscan_reset();
            self.push(Move::new(from, to, piece));
        }
    }

    pub fn append_bb_pawn_pushes(&mut self, mut bb: u64, offset: i64, piece: Piece) {
        let promotions = bb & (CONSTS::MASKS[0].rank_mask | CONSTS::MASKS[63].rank_mask);
        bb &= !(CONSTS::MASKS[0].rank_mask | CONSTS::MASKS[63].rank_mask);

        while bb != 0 {
            let to = bb.bitscan_reset();
            self.push(Move::new((offset + to as i64) as u64, to, piece));
        }

        generate_promotion_moves(promotions, offset, piece, self);
    }

    pub fn append_bb_pawn_attacks(&mut self, mut bb: u64, offset: i64, opt_enpassant: Option<u64>, piece: Piece) {
        if let Some(enpassant_index) = opt_enpassant {
            let promotions = bb & (CONSTS::MASKS[0].rank_mask | CONSTS::MASKS[63].rank_mask);
            bb &= !(CONSTS::MASKS[0].rank_mask | CONSTS::MASKS[63].rank_mask);

            generate_promotion_moves(promotions, offset, piece, self);

            while bb != 0 {
                let to = bb.bitscan_reset();
                let from = (to as i64 + offset) as u64;
                let mut m = Move::new(from, to, piece);
                if to == enpassant_index {
                    m.add_enpassant();
                }
                self.push(m);
            }
        } else {
            self.append_bb_pawn_pushes(bb, offset, piece);
        }
    }
}

pub struct MoveListIter<'a> {
    movelist: &'a MoveList,
    index: u8,
}

impl<'a> Iterator for MoveListIter<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.movelist.size {
            let item = unsafe { *self.movelist.list.get_unchecked(self.index as usize) };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
