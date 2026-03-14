use std::collections::HashMap;

use eframe::egui;
use egui::Sense;
use egui::{Align2, Color32, FontId, Pos2, Rect, Rgba, Vec2};
use web_sys::console;

use tinyhouse::get_bit;
use tinyhouse::move_gen::{GameState, BitBoard};
use tinyhouse::move_gen::Inventory;
use tinyhouse::move_gen::{Move, Piece, Side, Square};

pub struct TinyhouseApp {
    game_state: GameState,

    chess_board: ChessBoardWidget,
}

struct ChessBoardWidget {
    selected_square: Option<Square>,
    selected_piece: Option<Square>,
    promotion_pending: Option<(Square, Square)>, // (source, target)
    
    selected_inventory_piece: Option<Piece>,

    available_piece_moves: HashMap<Square, Vec<Move>>,
    available_placement_moves: HashMap<Piece, Vec<Move>>,
}

impl ChessBoardWidget {
    fn new(game_state: &GameState) -> Self {
        let mut widget = Self {
            selected_square: None,
            selected_piece: None,
            promotion_pending: None,
            selected_inventory_piece: None,
            available_piece_moves: HashMap::new(),
            available_placement_moves: HashMap::new(),
        };
        widget.compute_available_moves(game_state);
        widget
    }
    fn compute_available_moves(&mut self, game_state: &GameState) {
        self.available_piece_moves.clear();
        self.available_placement_moves.clear();
        for r#move in game_state.generate_legal_moves() {
            match r#move.source {
                Some(source) => self
                    .available_piece_moves
                    .entry(source)
                    .or_insert(Vec::new())
                    .push(r#move),
                None => self
                    .available_placement_moves
                    .entry(r#move.piece)
                    .or_insert(Vec::new())
                    .push(r#move),
            }
        }
    }

    fn draw_inventory(
        &mut self,
        game_state: &GameState,
        side: Side,
        painter: &mut egui::Painter,
        rect: egui::Rect,
    ) {
        painter.rect_filled(rect, 0.0, egui::Color32::PURPLE);
        
        let square_size = rect.size().x;
        // let smaller_size = rect.size().x * 0.8;
        let smaller_size = square_size;

        let colour = match side {
            Side::White => Color32::WHITE,
            Side::Black => Color32::BLACK
        };
        
        let (mut y, sign) = match side {
            Side::White => {
                (rect.min.y, 1.)
            },
            Side::Black => {
                (rect.max.y - square_size, -1.)
            }
        };
        let inventory = game_state.inventory()[side as usize];
        // let inventory = Inventory::from(255);

        for piece in [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn] {
            let count = inventory.get(piece);
            if count == 0 {
                y += smaller_size * sign;
                continue;
            }
            let centre = Pos2::new(
                rect.min.x + square_size * 0.5,
                y + square_size * 0.5
                );
                painter.text(
                    centre,
                    Align2::CENTER_CENTER,
                    piece.to_string(),
                    FontId::proportional(smaller_size * 0.8),
                    colour
                    );
                painter.text(
                    centre + Vec2::new(smaller_size * 0.4, -smaller_size * 0.4),
                    Align2::CENTER_CENTER,
                    count.to_string(),
                    FontId::proportional(smaller_size * 0.2),
                    colour
                    );
                y += smaller_size * sign;
        }
    }
    fn draw_board(
        &mut self,
        game_state: &GameState,
        painter: &mut egui::Painter,
        square_size: f32,
        rect: egui::Rect,
    ) {
        let origin = rect.min;
        let mut x;
        let mut y;

        for rank in 0..4 {
            for file in 0..4 {
                let square = 4 * rank + file;
                x = file as f32 * square_size;
                y = rank as f32 * square_size;
                    
                // Draw Square
                let square_rect = egui::Rect::from_min_size(
                    origin + Vec2::new(x, y),
                    Vec2::new(square_size, square_size),
                );
                let colour = if (rank + file) % 2 == 0 {
                    Color32::from_rgb(173, 120, 133)
                } else {
                    Color32::from_rgb(100, 120, 200)
                };
                painter.rect_filled(square_rect, 0.0, colour);
                
                // Draw Pieces
                for piece in Piece::ALL {
                    if get_bit!(game_state.bitboards()[piece], square) {
                        let center = Pos2::new(
                            origin.x + x + square_size * 0.5,
                            origin.y + y + square_size * 0.5,
                        );
                        let colour = if get_bit!(game_state.occupancies()[Side::White], square) {
                            Color32::WHITE
                        } else {
                            Color32::BLACK
                        };
                        painter.text(
                            center,
                            Align2::CENTER_CENTER,
                            piece.to_string(),
                            FontId::proportional(square_size * 0.8),
                            colour,
                        );
                    }
                }
            }
        }

        // Draw legal moves for selected piece
        if let Some(square) = self.selected_square {
            if let Some(moves) = self.available_piece_moves.get(&square) {
                self.selected_piece = Some(square);
                for r#move in moves {
                    let target_square = r#move.target as u8;
                    let rank = target_square / 4;
                    let file = target_square % 4;
                    let square_pos = origin + Vec2::new(file as f32 * square_size, rank as f32 * square_size);
                    let centre = square_pos + Vec2::splat(square_size * 0.5);
                    let colour = Color32::from_rgba_unmultiplied(200, 200, 200, 128);
                    painter.circle_filled(centre, square_size * 0.2, colour);
                }
            } 
        }
        if let Some(piece) = self.selected_inventory_piece {
            let mut available_squares = !(game_state.occupancies()[Side::White] | game_state.occupancies()[Side::Black]);
            if piece == Piece::Pawn {
                match game_state.side() {
                    Side::White => {available_squares &= BitBoard(0b0000_1111_1111_1111);}
                    Side::Black => {available_squares &= BitBoard(0b1111_0000_0000_0000);}
                }
            }
            for square in available_squares {
                let rank = square as u32 / 4;
                let file = square as u32 % 4;
                let square_pos = origin + Vec2::new(file as f32 * square_size, rank as f32 * square_size);
                let centre = square_pos + Vec2::splat(square_size * 0.5);
                let colour = Color32::from_rgba_unmultiplied(200, 200, 200, 128);
                painter.circle_filled(centre, square_size * 0.2, colour);
            
            }
        } 
    }
    fn handle_board_clicked(&mut self, response: &egui::Response, rect: egui::Rect, square_size: f32) {
        if let Some(pos) = response.interact_pointer_pos() {
            let relative_pos = pos - rect.min;
            let clicked_file = (relative_pos.x / square_size).floor() as u32;
            let clicked_rank = (relative_pos.y / square_size).floor() as u32;

            if clicked_file < 4 && clicked_rank < 4 {
                let clicked_square = Square::from(4 * clicked_rank + clicked_file);
                self.selected_square = Some(clicked_square); 
                if let Some(moves) = self.available_piece_moves.get(&clicked_square) {
                    self.selected_piece = Some(clicked_square);
                } 
            } 
        }
    }
    fn deselect_board(&mut self) {
        self.selected_square = None;
        self.selected_piece = None;
        self.promotion_pending = None;
    }
    fn deselect_inventory(&mut self) {
        self.selected_inventory_piece = None;
    }
    fn handle_inventory_clicked(&mut self, response: &egui::Response, rect: egui::Rect, smaller_square_size: f32) {
        if let Some(pos) = response.interact_pointer_pos() {
            let relative_pos = pos - rect.min;
            let clicked_piece = (relative_pos.y / smaller_square_size).floor() as usize;

            if clicked_piece < 4 {
                let clicked_piece = [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn][clicked_piece]; 
                    
                self.selected_inventory_piece = Some(clicked_piece);
            } 
        } 
    }

    fn update(&mut self, game_state: &mut GameState) {
        if let Some(target_square) = self.selected_square {
            if let Some(source_square) = self.selected_piece {
                // these will only differ at this point if self.selected_square is Some but
                // self.available_piece_moves.get(&square) was None
                if let Some(moves) = self.available_piece_moves.get(&source_square) {
                    let mut selected_move = None;
                    for r#move in moves {
                        // TODO: For given source/target we cannot assume there is unique move
                        // Precisely because promotions exist
                        
                        if r#move.promoted_piece == None {
                            if r#move.target == target_square {
                                selected_move = Some(r#move);
                                break;
                            }
                        } else {
                            self.promotion_pending = Some((source_square, target_square));
                            break;
                        }
                    } 
                    if let Some(r#move) = selected_move {
                        *game_state = game_state.make_move(r#move);
                        self.compute_available_moves(game_state);

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
            if let Some(piece) = self.selected_inventory_piece {
                todo!()
            }
        }
    } 

    fn ui(&mut self, ui: &mut egui::Ui, game_state: &mut GameState) {
        let inventory_width = 100.0;
        let square_size = 100.0;
        let board_size = 4. * square_size;

        let size = Vec2::new(inventory_width + board_size + inventory_width, board_size);

        let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
        let rect = response.rect;

        let left_inventory_rect =
            egui::Rect::from_min_size(rect.min, Vec2::new(inventory_width, board_size));
        let left_inventory_rect =
            egui::Rect::from_min_size(rect.min, Vec2::new(inventory_width, board_size));
        let board_rect = egui::Rect::from_min_size(
            Pos2::new(rect.min.x + inventory_width, rect.min.y),
            Vec2::new(board_size, board_size),
        );
        let right_inventory_rect = egui::Rect::from_min_size(
            Pos2::new(rect.min.x + inventory_width + board_size, rect.min.y),
            Vec2::new(inventory_width, board_size),
        );

        let mut left_inventory_painter = ui.painter_at(left_inventory_rect);
        let mut board_painter = ui.painter_at(board_rect);
        let mut right_inventory_painter = ui.painter_at(right_inventory_rect);

        // --- input handling ---
        // self.handle_input(&response);
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if left_inventory_rect.contains(pos) {
                    self.handle_inventory_clicked(&response, left_inventory_rect, square_size);
                } else {
                    
                }
                if board_rect.contains(pos) {
                    self.handle_board_clicked(&response, board_rect, square_size)
                } else {
                    self.deselect_board();
                }
            }
        }
        if response.clicked_elsewhere() {
            self.deselect_board();
        }

        // --- state update ---
        self.update(game_state);

        // --- drawing ---
        self.draw_inventory(
            game_state,
            Side::White,
            &mut left_inventory_painter,
            left_inventory_rect,
        );
        self.draw_board(game_state, &mut board_painter, square_size, board_rect);
        self.draw_inventory(
            game_state,
            Side::Black,
            &mut right_inventory_painter,
            right_inventory_rect,
        );
    }
}

impl TinyhouseApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let game_state = GameState::default();

        let chess_board = ChessBoardWidget::new(&game_state);
        Self {
            game_state,
            chess_board,
        }
    }
}

impl eframe::App for TinyhouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tinyhouse");
            self.chess_board.ui(ui, &mut self.game_state);
        });
    }
}
