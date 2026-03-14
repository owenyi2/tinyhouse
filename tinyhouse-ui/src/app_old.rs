use std::collections::HashMap;

use eframe::egui;
use egui::{Rect, Pos2, Vec2, Color32, Align2, FontId, Rgba};
use egui::{Sense};
use web_sys::console;

use tinyhouse::move_gen::GameState;
use tinyhouse::move_gen::{Square, Piece, Side, Move};
use tinyhouse::get_bit;

pub struct TinyhouseApp {
    game_state: GameState,
    available_piece_moves: HashMap<Square, Vec<Move>>,
    available_placement_moves: HashMap<Piece, Vec<Move>>,
    selected_square: Option<Square>,
    selected_piece: Option<Square>,
}
impl TinyhouseApp {
    fn compute_available_moves(&mut self) {
        self.available_piece_moves.clear();
        self.available_placement_moves.clear();
        for r#move in self.game_state.generate_legal_moves() {
            match r#move.source {
                Some(source) => self.available_piece_moves.entry(source).or_insert(Vec::new()).push(r#move),
                None => self.available_placement_moves.entry(r#move.piece).or_insert(Vec::new()).push(r#move)
            }
        }
    }

    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut game_state = Self {
            game_state: GameState::default(),
            available_piece_moves: HashMap::new(),
            available_placement_moves: HashMap::new(),
            selected_square: None,
            selected_piece: None,
        };
        game_state.compute_available_moves();
        game_state
    }
    fn draw_board(&mut self, ui: &mut egui::Ui) {
        let square_size = 100.0;
        let (response, painter) =
            ui.allocate_painter(Vec2::new(4. * square_size, 4. * square_size), Sense::click());
        
        let origin = response.rect.min;
        let mut x; 
        let mut y;

        for rank in 0..4 {
            for file in 0..4 {
                let square = 4 * rank + file;
                x = file as f32 * square_size;
                y = rank as f32 * square_size;

                let square_rect = egui::Rect::from_min_size(origin + Vec2::new(x, y), Vec2::new(square_size, square_size));
                let color = if (rank + file) % 2 == 0 {
                    Color32::from_rgb(173, 120, 133)
                } else {
                    Color32::from_rgb(100, 120, 200)
                };
                painter.rect_filled(square_rect, 0.0, color);

                for piece in Piece::ALL {
                    if get_bit!(self.game_state.bitboards()[piece], square) {
                        let center = Pos2::new(
                            origin.x + x + square_size * 0.5,
                            origin.y + y + square_size * 0.5,
);
                        let color = if get_bit!(self.game_state.occupancies()[Side::White], square) {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        };
                        painter.text(
                            center,
                            Align2::CENTER_CENTER,
                            piece.to_string(),
                            FontId::proportional(square_size * 0.8),
                            color
                        );
                    }
                }
            }
        } 
        if let Some(square) = self.selected_square {
            if let Some(moves) = self.available_piece_moves.get(&square) {
                self.selected_piece = Some(square);
                for r#move in moves {
                    let target_square = r#move.target as u8;
                    let rank = target_square / 4;
                    let file = target_square % 4;
                    let square_pos = origin + Vec2::new(file as f32 * square_size, rank as f32 * square_size);
                    let centre = square_pos + Vec2::splat(square_size * 0.5);
                    let color = Color32::from_rgba_unmultiplied(200, 200, 200, 128);
                    painter.circle_filled(centre, square_size * 0.2, color);
                }
            } 
        }
        if let (Some(source_square), Some(target_square)) = (self.selected_piece, self.selected_square) {
            // these will only differ at this point if self.selected_square is Some but
            // self.available_piece_moves.get(&square) was None
            if let Some(moves) = self.available_piece_moves.get(&source_square) {
                let mut selected_move = None;
                for r#move in moves {
                    // TODO: For given source/target we cannot assume there is unique move
                    // Precisely because promotions exist
                    if r#move.target == target_square {
                        selected_move = Some(r#move);
                        break;
                    }
                } 
                if let Some(r#move) = selected_move {
                    self.game_state = self.game_state.make_move(r#move);
                    self.compute_available_moves();

                    for (square, moves) in &self.available_piece_moves {
                        console::log_1(&format!("{}", square).into());
                        for r#move in moves {
                            console::log_1(
                                &format!("{:?}", r#move).into()
                                );
                        }
                    } 
                }
            }  
        }
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let relative_pos = pos - origin;
                let clicked_file = (relative_pos.x / square_size).floor() as u8;
                let clicked_rank = (relative_pos.y / square_size).floor() as u8;
                if clicked_file < 4 && clicked_rank < 4 {
                    let clicked_square: u8 = 4 * clicked_rank + clicked_file;
                    self.selected_square = Some(Square::from(clicked_square as u32));
 
                    console::log_1(
                        &format!(
                            "Clicked square: file {}, rank {}, index {}",
                            clicked_file, clicked_rank, clicked_square
                        )
                        .into(),
                    );
                }
            }
        } 
        if response.clicked_elsewhere() {
            self.selected_square = None;
            self.selected_piece = None;
        }
    }
}

impl eframe::App for TinyhouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tinyhouse");
            self.draw_board(ui); 
        });
    }
}
