use crate::{board::Board, chess_move::Move, move_list::MoveList, piece::*};
use macroquad::prelude::*;

pub struct Gui {
    texture_rect: [[Rect; 6]; 2],
    sprite_sheet: Texture2D,
    moving_piece: Option<Piece>,
    moving_piece_index: u64,
    window_w: f32,
    window_h: f32,
}

impl Gui {
    pub async fn new() -> Gui {
        let mut texture_rect: [[Rect; 6]; 2] = [[Rect::default(); 6]; 2];
        let sprite_sheet = load_texture("./assets/texture.png").await.unwrap();
        sprite_sheet.set_filter(FilterMode::Linear);

        for (i, row) in texture_rect.iter_mut().enumerate() {
            for (j, rec) in row.iter_mut().enumerate() {
                let piece_color = match PieceColor::from(i) {
                    PieceColor::White => 1,
                    PieceColor::Black => 0,
                };

                let piece_type = match PieceType::from(j) {
                    PieceType::Rook => 0,
                    PieceType::Bishop => 2,
                    PieceType::Queen => 3,
                    PieceType::Knight => 1,
                    PieceType::Pawn => 5,
                    PieceType::King => 4,
                };

                *rec = Rect::new(piece_type as f32 * 100.0, piece_color as f32 * 100.0, 100.0, 100.0);
            }
        }

        Gui {
            texture_rect,
            sprite_sheet,
            moving_piece: None,
            moving_piece_index: 0,
            window_w: screen_width(),
            window_h: screen_height(),
        }
    }

    pub fn draw(&mut self, board: &Board) {
        self.window_w = screen_width();
        self.window_h = screen_height();

        self.draw_bg();

        if self.moving_piece.is_some() {
            self.draw_moves(board.get_legal_moves(), self.moving_piece_index);
        }

        for index in 0..64 {
            if let Some(piece) = board.get_piece_at(index) {
                self.draw_piece(index as usize / 8, index as usize % 8, piece);
            }
        }
    }

    fn draw_bg(&self) {
        const LIGHT_SQUARE_COLOR: Color = Color::new(0.945, 0.851, 0.753, 1.0);
        const DARK_SQUARE_COLOR: Color = Color::new(0.663, 0.478, 0.396, 1.0);
        let square_w = self.window_w / 8.0;
        let square_h = self.window_h / 8.0;

        for i in 0..8 {
            for j in 0..8 {
                let color = if (i + j) % 2 == 0 {
                    LIGHT_SQUARE_COLOR
                } else {
                    DARK_SQUARE_COLOR
                };
                draw_rectangle(j as f32 * square_w, i as f32 * square_h, square_w, square_h, color);
            }
        }
    }

    fn draw_moves(&self, moves: &MoveList, from: u64) {
        const MOVE_START_SQUARE_COLOR: Color = Color::new(1.0, 0.0, 0.0, 0.549);
        const MOVE_LAND_SQUARE_COLOR: Color = Color::new(1.0, 0.584, 0.110, 0.745);
        let square_w = self.window_w / 8.0;
        let square_h = self.window_h / 8.0;

        draw_rectangle(
            (from % 8) as f32 * square_w,
            (7 - from / 8) as f32 * square_h,
            square_w,
            square_h,
            MOVE_START_SQUARE_COLOR,
        );

        for m in moves.iter() {
            if m.get_from() == from {
                let to = m.get_to();
                draw_rectangle(
                    (to % 8) as f32 * square_w,
                    (7 - to / 8) as f32 * square_h,
                    square_w,
                    square_h,
                    MOVE_LAND_SQUARE_COLOR,
                );
            }
        }
    }

    fn draw_piece(&self, row_index: usize, col_index: usize, piece: Piece) {
        let square_size_x: f32 = self.window_w / 8.0;
        let square_size_y: f32 = self.window_h / 8.0;

        let (xpx, ypx) = if self.moving_piece.is_none() || self.moving_piece_index != (row_index * 8 + col_index) as u64 {
            (col_index as f32 * square_size_x, (7 - row_index) as f32 * square_size_y)
        } else {
            let mouse_pos = mouse_position();
            (mouse_pos.0 - square_size_x / 2.0, mouse_pos.1 - square_size_y / 2.0)
        };

        draw_texture_ex(
            &self.sprite_sheet,
            xpx,
            ypx,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(square_size_x, square_size_y)),
                source: Some(self.texture_rect[piece.get_color()][piece.get_type()]),
                ..Default::default()
            },
        );
    }

    pub fn handle_input(&mut self, board: &mut Board) {
        if is_mouse_button_pressed(MouseButton::Left) && self.moving_piece.is_none() {
            let clicked_index = self.mouse_pos_to_index(mouse_position());
            
            if clicked_index >= 64 {
                return;
            }

            if let Some(clicked_piece) = board.get_piece_at(clicked_index) {
                self.moving_piece = Some(clicked_piece);
                self.moving_piece_index = clicked_index;
            }
        } else if is_mouse_button_released(MouseButton::Left) && self.moving_piece.is_some() {
            let land_index = self.mouse_pos_to_index(mouse_position());

            let mut played_move = Move::new(self.moving_piece_index, land_index, self.moving_piece.unwrap());

            if (land_index / 8 == 0 || land_index / 8 == 7) && played_move.get_moved_piece().get_type() == PieceType::Pawn {
                played_move.add_promotion(PieceType::Queen);
            }

            if let Some(played_move_in_vec) = board.get_legal_moves().iter().find(|m| *m == played_move) {
                board.make_move(played_move_in_vec);
                board.generate_legal_moves();
            }

            self.moving_piece = None;
        }
    }

    fn mouse_pos_to_index(&self, mouse_pos: (f32, f32)) -> u64 {
        (8.0 - mouse_pos.1 * 8.0 / self.window_h) as u64 * 8 + (mouse_pos.0 * 8.0 / self.window_w) as u64
    }
}
