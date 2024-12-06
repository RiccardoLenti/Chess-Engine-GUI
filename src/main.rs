mod bitboard;
mod board;
mod chess_move;
mod consts;
mod engine;
mod gamestate;
mod gui;
mod move_generation;
mod move_list;
mod piece;

use std::time::Instant;

use crate::board::Board;
use crate::gui::Gui;

use macroquad::prelude::*;
use move_generation::generate_legal_moves;
use piece::*;

const _INITIAL_FEN_STRING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"; //KQkq -";
const _TEST_FEN_STRING: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ";

const ENGINE_DEPTH: u8 = 7;

#[macroquad::main("Chess")]

async fn main() {
    request_new_screen_size(900.0, 900.0);
    let mut gui = Gui::new().await;
    let mut board = Board::new(_INITIAL_FEN_STRING);
    let cpu_color = !board.get_color_to_move();

    //_perft_test(8, &mut board);

    loop {
        if board.get_color_to_move() == cpu_color {
            engine::play_next_move(&mut board);
            board.generate_legal_moves();
        }

        gui.handle_input(&mut board);

        gui.draw(&board);

        next_frame().await
    }
}

fn _perft_test(max_depth: u8, board: &mut Board) {
    println!("---------------------------------------");
    println!("|                                     |");
    println!("|  Running Perft test with depth: {:>2}  |", max_depth);
    println!("|                                     |");
    println!("---------------------------------------");
    for i in 1..=max_depth {
        let now = Instant::now();
        println!(
            "Depth: {:>2} | Nodes: {:>12} | Time: {}s",
            i,
            _perft_test_r(i, board, i),
            now.elapsed().as_secs_f32()
        );
    }
}

fn _perft_test_r(depth: u8, board: &mut Board, max_depth: u8) -> u128 {
    if depth == 0 {
        return 1;
    }

    let legal_moves = generate_legal_moves(board);
    if depth == 1 {
        return legal_moves.len() as u128;
    }

    let mut res = 0;
    for m in legal_moves.iter() {
        board.make_move(m);
        let positions_after_this_move = _perft_test_r(depth - 1, board, max_depth);
        board.unmake_move(m);

        #[cfg(debug_assertions)]
        {
            if depth == max_depth {
                println!(
                    "{}{} : {}",
                    _square_to_str(m.get_from()),
                    _square_to_str(m.get_to()),
                    positions_after_this_move
                );
            }
        }

        res += positions_after_this_move;
    }

    res
}

fn _square_to_str(index: u64) -> String {
    let y = index / 8;
    let x = index % 8;

    (('a' as u8 + x as u8) as char).to_string() + &((y as u8 + '1' as u8) as char).to_string()
}

fn str_to_square(name: &str) -> u64 {
    let mut chars = name.chars();

    ((chars.next().unwrap() as u32 - 'a' as u32) + (chars.next().unwrap().to_digit(10).unwrap() - 1) * 8) as u64
}
