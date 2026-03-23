use std::collections::HashMap;

use eframe::egui;
use egui::Sense;
use egui::{Align2, Color32, FontId, Pos2, Rect, Rgba, Vec2};
use web_sys::{console, window};

use tinyhouse::get_bit;
use tinyhouse::move_gen::{GameState, BitBoard};
use tinyhouse::move_gen::Inventory;
use tinyhouse::move_gen::{Move, Piece, Side, Square};
use search::alphabeta_best_move;

struct MainMenuState {
    mode: PlayMode,
    side: Side
}

impl MainMenuState {
    fn new() -> Self {
        Self { mode: PlayMode::PassAndPlay, side: Side::White }
    }
    fn ui(&mut self, ctx: &egui::Context) -> Option<Screen> {
        let mut next_screen = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Main Menu");
            
            egui::ComboBox::from_label("Mode")
                .selected_text(self.mode.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.mode, PlayMode::PassAndPlay, "Pass and Play");
                    ui.selectable_value(&mut self.mode, PlayMode::Computer, "Computer");
                });

            if matches!(self.mode, PlayMode::Computer) {
                egui::ComboBox::from_label("Side")
                    .selected_text(format!("{}", self.side))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.side, Side::White, "White");
                        ui.selectable_value(&mut self.side, Side::Black, "Black");
                    }); 
            }

            if ui.button("Play").clicked() {
                match self.mode {
                    PlayMode::PassAndPlay => {
                        next_screen = Some(Screen::PassAndPlay(PassAndPlayState::new()));
                    },
                    PlayMode::Computer => {
                        next_screen = Some(Screen::PlayComputer(PlayComputerState::new(self.side)));
                    }
                }
            };
        });
        next_screen
    }
}

#[derive(PartialEq)]
enum PlayMode {
    PassAndPlay,
    Computer
}

impl PlayMode {
    fn as_str(&self) -> &'static str {
        match self {
            PlayMode::PassAndPlay => "Pass and Play",
            PlayMode::Computer => "Computer",
        }
    }
}

struct ChessBoardWidget {
    flip_board: bool,

    selected_square: Option<Square>,
    selected_piece: Option<Square>,
    
    promotion_pending: Option<Move>,
    promotion_selected: Option<Piece>,

    selected_inventory_piece: Option<Piece>,

    available_piece_moves: HashMap<Square, Vec<Move>>,
    available_placement_moves: HashMap<Piece, Vec<Move>>,
    player_side: Side,
    
    result: Option<(Side, bool)> // true is checkmate, false is stalemate
}

impl ChessBoardWidget {
    fn new(game_state: &GameState) -> Self {
        let mut widget = Self {
            flip_board: true, 

            selected_square: None,
            selected_piece: None,

            promotion_pending: None,
            promotion_selected: None,

            selected_inventory_piece: None,
            available_piece_moves: HashMap::new(),
            available_placement_moves: HashMap::new(),
            player_side: Side::White,

            result: None, 
        };
        widget.compute_available_moves(game_state);
        widget
    }
    fn map_square(&self, rank: u8, file: u8) -> (u8, u8) {
        match self.player_side {
            Side::White => (rank, file),
            Side::Black => (3 - rank, 3 - file),
        }
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
    fn handle_board_clicked(&mut self, response: &egui::Response, rect: egui::Rect, square_size: f32) {
        if let Some(pos) = response.interact_pointer_pos() {
            let relative_pos = pos - rect.min;
            let mut clicked_file = (relative_pos.x / square_size).floor() as u32;
            let mut clicked_rank = (relative_pos.y / square_size).floor() as u32;
            
            
            (clicked_file, clicked_rank) = match self.player_side {
                Side::White => (clicked_file, clicked_rank),
                Side::Black => (3 - clicked_file, 3 - clicked_rank)
            };

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
    }
    fn deselect_inventory(&mut self) {
        self.selected_inventory_piece = None;
    }
    fn deselect_promotion(&mut self) {
        self.promotion_pending = None;
        self.promotion_selected = None;
    }
    fn handle_inventory_clicked(&mut self, response: &egui::Response, rect: egui::Rect, smaller_square_size: f32, left: bool) {
        if let Some(pos) = response.interact_pointer_pos() {
            let relative_pos = pos - rect.min;
            let mut clicked_piece = (relative_pos.y / smaller_square_size).floor() as usize;

            if clicked_piece < 4 {
                clicked_piece = match left {
                    true => clicked_piece,
                    false => 3 - clicked_piece,
                };
                let clicked_piece = [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn][clicked_piece]; 
                    
                self.selected_inventory_piece = Some(clicked_piece);
            } 
        } 
    }
    fn draw_legal_moves(
        &mut self, 
        game_state: &GameState,
        painter: &mut egui::Painter, 
        rect: egui::Rect
    ) {
        let origin = rect.min;
        if let Some(square) = self.selected_square {
            if let Some(moves) = self.available_piece_moves.get(&square) {
                for r#move in moves {
                    let target_square = r#move.target as u8;
                    let rank = target_square / 4;
                    let file = target_square % 4;
                    let (rank, file) = self.map_square(rank, file);
                    let square_pos = origin + Vec2::new(file as f32 * SQUARE_SIZE, rank as f32 * SQUARE_SIZE);
                    let centre = square_pos + Vec2::splat(SQUARE_SIZE * 0.5);
                    let colour = Color32::from_rgba_unmultiplied(200, 200, 200, 128);
                    painter.circle_filled(centre, SQUARE_SIZE * 0.2, colour);
                }
            } 
        }

        if let Some(piece) = self.selected_inventory_piece {
            if let Some(moves) = self.available_placement_moves.get(&piece) {
                for r#move in moves {
                    let target_square = r#move.target as u8;
                    let rank = target_square / 4;
                    let file = target_square % 4;
                    let (rank, file) = self.map_square(rank, file);
                    let square_pos = origin + Vec2::new(file as f32 * SQUARE_SIZE, rank as f32 * SQUARE_SIZE);
                    let centre = square_pos + Vec2::splat(SQUARE_SIZE * 0.5);
                    let colour = Color32::from_rgba_unmultiplied(200, 200, 200, 128);
                    painter.circle_filled(centre, SQUARE_SIZE * 0.2, colour);
                }
            } 
        } 
    }
    fn draw_board(
        &mut self, 
        game_state: &GameState,
        painter: &mut egui::Painter, 
        rect: egui::Rect
    ) {
        let origin = rect.min;
        let mut x;
        let mut y;

        for rank in 0..4 {
            for file in 0..4 {
                let square = 4 * rank + file;
                
                let (rank, file) = self.map_square(rank, file); 
                (x, y) = (file as f32 * SQUARE_SIZE, rank as f32 * SQUARE_SIZE);
                    
                // Draw Square
                let square_rect = egui::Rect::from_min_size(
                    origin + Vec2::new(x, y),
                    Vec2::new(SQUARE_SIZE, SQUARE_SIZE),
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
                            origin.x + x + SQUARE_SIZE * 0.5,
                            origin.y + y + SQUARE_SIZE * 0.5,
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
                            FontId::proportional(SQUARE_SIZE * 0.8),
                            colour,
                        );
                    }
                }
            }
        }
    }
    fn draw_inventory(
        &mut self,
        game_state: &GameState,
        side: Side,
        painter: &mut egui::Painter,
        rect: egui::Rect,
        left: bool
    ) {
        painter.rect_filled(rect, 0.0, egui::Color32::PURPLE);
        
        let square_size = rect.size().x;
        // let smaller_size = rect.size().x * 0.8;
        let smaller_size = square_size;

        let colour = match side {
            Side::White => Color32::WHITE,
            Side::Black => Color32::BLACK
        };
        
        let (mut y, sign) = match left {
            true => {
                (rect.min.y, 1.)
            },
            false => {
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
    fn handle_promotion_input(&mut self, ui: &mut egui::Ui) {
        if self.promotion_pending.is_some() {
            egui::ComboBox::from_label("Choose piece")
                .selected_text(format!("{}", match self.promotion_selected { Some(piece) => piece.to_string(), None => "".to_string()}))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.promotion_selected, Some(Piece::Wazir), "Wazir");
                    ui.selectable_value(&mut self.promotion_selected, Some(Piece::Ferz), "Ferz");
                    ui.selectable_value(&mut self.promotion_selected, Some(Piece::Ma), "Ma");
                });
        }
    }
    fn try_parse_move(&mut self, game_state: &GameState) -> Option<&Move> {
        let mut selected_move = None;
        if let Some(target_square) = self.selected_square {
            if let Some(source_square) = self.selected_piece {
                if let Some(moves) = self.available_piece_moves.get(&source_square) {
                    for r#move in moves {
                        if r#move.promoted_piece == None {
                            if r#move.target == target_square {
                                selected_move = Some(r#move);
                                break;
                            }
                        } else {
                            if r#move.target == target_square {
                                self.promotion_pending = Some(r#move.clone());
                                break;
                            }
                        }
                    } 
                }
            }
            if let Some(piece) = self.selected_inventory_piece {
                if let Some(moves) = self.available_placement_moves.get(&piece) {
                    for r#move in moves {
                        if r#move.target == target_square {
                            selected_move = Some(r#move);
                            break;
                        }
                    }
                }
            }
        }
        if let (Some(r#move), Some(piece)) = (&mut self.promotion_pending, self.promotion_selected) {
            r#move.promoted_piece = Some(piece);
            selected_move = Some(r#move);
        }
        selected_move
    }
    fn setup(&self, ui: &mut egui::Ui) -> (egui::Response, Rect, Rect, Rect) {
        let size = Vec2::new(INVENTORY_WIDTH + BOARD_SIZE + INVENTORY_WIDTH, BOARD_SIZE);

        let (response, painter) = ui.allocate_painter(size, egui::Sense::click());
        let rect = response.rect;

        let left_inventory_rect =
            egui::Rect::from_min_size(rect.min, Vec2::new(INVENTORY_WIDTH, BOARD_SIZE));
        let board_rect = egui::Rect::from_min_size(
            Pos2::new(rect.min.x + INVENTORY_WIDTH, rect.min.y),
            Vec2::new(BOARD_SIZE, BOARD_SIZE),
        );
        let right_inventory_rect = egui::Rect::from_min_size(
            Pos2::new(rect.min.x + INVENTORY_WIDTH + BOARD_SIZE, rect.min.y),
            Vec2::new(INVENTORY_WIDTH, BOARD_SIZE),
        );
        (response, left_inventory_rect, board_rect, right_inventory_rect)
    }
    fn check_result(&mut self, game_state: &GameState) {
        self.result = game_state.detect_mate().map(
            |s| {
                match s {
                    true => (-*game_state.side(), true),
                    false => (*game_state.side(), false)
            } 
            }
        );

    }
}

const INVENTORY_WIDTH: f32 = 100.0;
const SQUARE_SIZE: f32 = 100.0;
const BOARD_SIZE: f32 = 4. * SQUARE_SIZE;

struct PassAndPlayState {
    game_state: GameState,
    chess_board: ChessBoardWidget
}

impl PassAndPlayState {
    fn new() -> Self {
        let game_state = GameState::default();
        let chess_board = ChessBoardWidget::new(&game_state);
        Self { game_state, chess_board }
    }
    fn ui(&mut self, ctx: &egui::Context) -> Option<Screen> {
        let mut next_screen = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tinyhouse");
            let chess_board = &mut self.chess_board;
            let game_state = &mut self.game_state;

            match chess_board.result {
                None => {
                    let (response, left_inventory_rect, board_rect, right_inventory_rect) = chess_board.setup(ui);
                    
                    let mut left_inventory_painter = ui.painter_at(left_inventory_rect);
                    let mut board_painter = ui.painter_at(board_rect);
                    let mut right_inventory_painter = ui.painter_at(right_inventory_rect);
                    
                    chess_board.handle_promotion_input(ui);
                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if left_inventory_rect.contains(pos) {
                                chess_board.handle_inventory_clicked(&response, left_inventory_rect, SQUARE_SIZE, true);
                                chess_board.deselect_promotion();
                            } 
                             
                            if board_rect.contains(pos) {
                                chess_board.handle_board_clicked(&response, board_rect, SQUARE_SIZE);
                                chess_board.deselect_promotion();
                            } else {
                                chess_board.deselect_board();
                            }
                            if !chess_board.selected_piece.is_none() {
                                chess_board.deselect_inventory();
                            }
                        }
                    } 

                    if let Some(selected_move) = chess_board.try_parse_move(game_state) {  
                        *game_state = game_state.make_move(selected_move);
                        chess_board.compute_available_moves(game_state);

                        if chess_board.flip_board {
                            chess_board.player_side = *game_state.side(); // board-flipping
                        }
                        chess_board.deselect_inventory();
                        chess_board.deselect_promotion();
                    } 
                    
                    chess_board.draw_inventory(
                        game_state,
                        chess_board.player_side,
                        &mut left_inventory_painter,
                        left_inventory_rect,
                        true
                    );
                    chess_board.draw_board(game_state, &mut board_painter, board_rect);
                    chess_board.draw_legal_moves(game_state, &mut board_painter, board_rect);
                    chess_board.draw_inventory(
                        game_state,
                        -chess_board.player_side,
                        &mut right_inventory_painter,
                        right_inventory_rect,
                        false
                    ); 
                    chess_board.check_result(game_state); 
                },
                Some((side, checkmate)) => {
                    if let Some(win) = window() {
                        if checkmate {
                            win.alert_with_message(&format!("CHECKMATE! {} wins", side));
                        } else {
                            win.alert_with_message(&format!("STALEMATE! {} wins", side));
                        }
                    } 
                    next_screen = Some(Screen::MainMenu(MainMenuState::new()));
                }
            } 
        });
        next_screen 
    }
}

struct PlayComputerState {
    game_state: GameState,
    chess_board: ChessBoardWidget
}

impl PlayComputerState {
    fn new(player_side: Side) -> Self {
        let game_state = GameState::default();
        let mut chess_board = ChessBoardWidget::new(&game_state);
        chess_board.flip_board = false;
        chess_board.player_side = player_side;
        Self { game_state, chess_board }
    }
    fn ui(&mut self, ctx: &egui::Context) -> Option<Screen> {
        let mut next_screen = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tinyhouse");
            let chess_board = &mut self.chess_board;
            let game_state = &mut self.game_state;
            
            match chess_board.result {
                None => {
                    let (response, left_inventory_rect, board_rect, right_inventory_rect) = chess_board.setup(ui);
                    
                    let mut left_inventory_painter = ui.painter_at(left_inventory_rect);
                    let mut board_painter = ui.painter_at(board_rect);
                    let mut right_inventory_painter = ui.painter_at(right_inventory_rect);
                    
                    let mut selected_move = None; 
                    if chess_board.player_side == *game_state.side() {
                        chess_board.handle_promotion_input(ui);
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if left_inventory_rect.contains(pos) {
                                    chess_board.handle_inventory_clicked(&response, left_inventory_rect, SQUARE_SIZE, true);
                                    chess_board.deselect_promotion();
                                } 
                                 
                                if board_rect.contains(pos) {
                                    chess_board.handle_board_clicked(&response, board_rect, SQUARE_SIZE);
                                    chess_board.deselect_promotion();
                                } else {
                                    chess_board.deselect_board();
                                }
                            }
                            if !chess_board.selected_piece.is_none() {
                                chess_board.deselect_inventory();
                            }
                        } 
                        selected_move = chess_board.try_parse_move(game_state).cloned();
                    } else {
                        selected_move = Some(alphabeta_best_move(game_state, 7).expect("returns None only if no moves possible"));
                    }
                    
                    if let Some(r#move) = selected_move {
                        *game_state = game_state.make_move(&r#move);
                        chess_board.compute_available_moves(game_state);

                        if chess_board.flip_board {
                            chess_board.player_side = *game_state.side(); // board-flipping
                        }
                        chess_board.deselect_inventory();
                        chess_board.deselect_promotion();
                        chess_board.deselect_board();
                    } 

                    chess_board.draw_inventory(
                        game_state,
                        chess_board.player_side,
                        &mut left_inventory_painter,
                        left_inventory_rect,
                        true
                    );
                    chess_board.draw_board(game_state, &mut board_painter, board_rect);
                    chess_board.draw_legal_moves(game_state, &mut board_painter, board_rect);
                    chess_board.draw_inventory(
                        game_state,
                        -chess_board.player_side,
                        &mut right_inventory_painter,
                        right_inventory_rect,
                        false
                    ); 
                    
                    chess_board.check_result(game_state); 
                },
                Some((side, checkmate)) => {
                    if let Some(win) = window() {
                        if checkmate {
                            win.alert_with_message(&format!("CHECKMATE! {} wins", side));
                        } else {
                            win.alert_with_message(&format!("STALEMATE! {} wins", side));
                        }
                    } 
                    next_screen = Some(Screen::MainMenu(MainMenuState::new()));
                }
            }
        });
        next_screen
    }
}

enum Screen {
    MainMenu(MainMenuState),
    PassAndPlay(PassAndPlayState),
    PlayComputer(PlayComputerState)
}

pub struct TinyhouseApp {
    screen: Screen
}

impl TinyhouseApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { screen: Screen::MainMenu(MainMenuState::new()) }
    }
}

impl eframe::App for TinyhouseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut next_screen = None;
        match &mut self.screen {
            Screen::MainMenu(state) => {
                next_screen = state.ui(ctx);
            }
            Screen::PassAndPlay(state) => {
                next_screen = state.ui(ctx);
            }
            Screen::PlayComputer(state) => {
                next_screen = state.ui(ctx);
            }
        }
        if let Some(screen) = next_screen {
            self.screen = screen;
        }
    }
}
