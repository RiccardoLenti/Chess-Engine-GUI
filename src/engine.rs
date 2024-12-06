use std::time::Instant;

use crate::{board::Board, chess_move::Move, move_generation::generate_legal_moves, move_list::MoveList};

const PIECE_WEIGHTS: [i32; 6] = [500, 330, 900, 300, 100, 0];
const COLOR_MULTIPLIERS: [i32; 2] = [1, -1];

pub fn play_next_move(board: &mut Board) {
    let mut best_move: Option<Move> = None;
    let mut max_eval = i32::MIN;

    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX - 1;

    let now = Instant::now();

    let mut legal_moves = generate_legal_moves(board);
    order_moves(&mut legal_moves, board);

    for m in legal_moves.iter() {
        board.make_move(m);
        let this_move_eval = -alpha_beta(board, -beta, -alpha, crate::ENGINE_DEPTH - 1);
        board.unmake_move(m);
        if this_move_eval > max_eval {
            max_eval = this_move_eval;
            best_move = Some(m);
            alpha = this_move_eval; // don't think the if is needed
        }
    }

    println!("Current eval: {} | {}s", max_eval, now.elapsed().as_secs_f32());
    board.make_move(best_move.unwrap());
}

fn alpha_beta(board: &mut Board, mut alpha: i32, beta: i32, depth: u8) -> i32 {
    if depth == 0 {
        return eval(board);
    }

    let mut max_eval = i32::MIN + 1;

    let mut legal_moves = generate_legal_moves(board);
    if legal_moves.len() == 0 {
        return max_eval;
    }
    order_moves(&mut legal_moves, board);

    for m in legal_moves.iter() {
        board.make_move(m);
        let this_move_eval = -alpha_beta(board, -beta, -alpha, depth - 1);
        board.unmake_move(m);

        if this_move_eval > max_eval {
            max_eval = this_move_eval;
            if this_move_eval > alpha {
                alpha = this_move_eval;
            }
        }

        if this_move_eval >= beta {
            return max_eval;
        }
    }
    max_eval
}

fn eval(board: &Board) -> i32 {
    let mut res = 0;

    for (piece_color, bbs_ar) in board.get_pieces_bb().iter().enumerate() {
        for (piece_type, bb) in bbs_ar.iter().enumerate() {
            res += PIECE_WEIGHTS[piece_type] * COLOR_MULTIPLIERS[piece_color] * bb.count_ones() as i32;
        }
    }

    res * COLOR_MULTIPLIERS[board.get_color_to_move()]
}

fn order_moves(moves: &mut MoveList, board: &Board) {
    let mut scores: Vec<i32> = vec![0; moves.len() as usize];

    for (i, m) in moves.iter().enumerate() {
        if let Some(captured_piece) = board.get_piece_at(m.get_to()) {
            scores[i] = PIECE_WEIGHTS[captured_piece.get_type()] - PIECE_WEIGHTS[m.get_moved_piece().get_type()];
        }

        if m.is_promotion() {
            scores[i] += PIECE_WEIGHTS[m.get_promotion_type()];
        }
    }

    quick_sort(moves, &mut scores, 0, moves.len() as isize - 1);
}

fn quick_sort(moves: &mut MoveList, scores: &mut Vec<i32>, low: isize, high: isize) {
    if low < high {
        let pivot_index = partition(moves, scores, low, high);
        quick_sort(moves, scores, low, pivot_index - 1);
        quick_sort(moves, scores, pivot_index + 1, high);
    }
}

fn partition(moves: &mut MoveList, scores: &mut Vec<i32>, low: isize, high: isize) -> isize {
    let pivot_score = scores[high as usize];
    let mut i = low - 1;

    for j in low..high {
        if scores[j as usize] > pivot_score {
            i += 1;
            moves.swap(i as usize, j as usize);
            scores.swap(i as usize, j as usize);
        }
    }

    moves.swap((i + 1) as usize, high as usize);
    scores.swap((i + 1) as usize, high as usize);

    i + 1
}