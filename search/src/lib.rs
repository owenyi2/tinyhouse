use std::cmp;

use tinyhouse::move_gen::{GameState, Move, Piece, Side, Square};

fn heuristic_eval(game_state: &GameState) -> f64 {
    let values = [
        (Piece::Wazir, 3.),
        (Piece::Ma, 2.),
        (Piece::Ferz, 2.),
        (Piece::Pawn, 1.),
    ];

    let inventory_multiplier = 1.0;

    let mut eval = 0.0;
    // White
    for (piece, value) in values {
        eval += (game_state.bitboards()[piece] & game_state.occupancies()[Side::White])
            .0
            .count_ones() as f64
            * value;

        eval += game_state.inventory()[0].get(piece) as f64 * value * inventory_multiplier;
    }

    // Black
    for (piece, value) in values {
        eval -= (game_state.bitboards()[piece] & game_state.occupancies()[Side::Black])
            .0
            .count_ones() as f64
            * value;
        eval -= game_state.inventory()[1].get(piece) as f64 * value * inventory_multiplier;
    }
    eval
}

fn terminal_eval(game_state: &GameState) -> f64 {
    let eval = match game_state.side() {
        Side::White => f64::INFINITY,
        Side::Black => f64::NEG_INFINITY,
    };
    if game_state.detect_check(*game_state.side()) {
        return -eval; // White got checkmated => -inf. Black got checkmated => inf
    } else {
        // Getting stalemated is a win
        return eval; // White got stalemated => inf. Black got stalemated => -inf
    }
}

fn minimax(state: &GameState, depth: i8, max: bool) -> f64 {
    let moves: Vec<Move> = state.generate_legal_moves().collect();

    if moves.is_empty() {
        return terminal_eval(state);
    }
    if depth == 0 {
        return heuristic_eval(state);
    }
    if max {
        let mut value = f64::NEG_INFINITY;
        for r#move in moves {
            let v = minimax(&state.make_move(&r#move), depth - 1, false);
            value = value.max(v);            

            // println!("{}move: {} {}", "\t".repeat((4 - depth).try_into().unwrap()), r#move, v);

        }
        return value;
    } else {
        let mut value = f64::INFINITY;
        for r#move in moves {
            let v = minimax(&state.make_move(&r#move), depth - 1, true);
            value = value.min(v);
            // println!("{}move: {} {}", "\t".repeat((4 - depth).try_into().unwrap()), r#move, v);
        }
        return value;
    }
}

fn alphabeta(state: &GameState, depth: i8, mut alpha: f64, mut beta: f64, max: bool) -> f64 {
    
    let moves: Vec<Move> = state.generate_legal_moves().collect();

    if moves.is_empty() {
        return terminal_eval(state);
    }
    if depth == 0 {
        return heuristic_eval(state);
    }

    if max {
        let mut value = f64::NEG_INFINITY;
        for r#move in moves {
            let v = alphabeta(&state.make_move(&r#move), depth - 1, alpha, beta, false);
            value = value.max(v);            

            if value >= beta { 
                break;
            }
            alpha = alpha.max(value)
        }
        return value; 
    } else {
        let mut value = f64::INFINITY;
        for r#move in moves {
            let v = alphabeta(&state.make_move(&r#move), depth - 1, alpha, beta, true);
            value = value.min(v);

            if value <= alpha {
                break;
            }
            beta = beta.min(value);
        }
        return value;
    }
}

fn minimax_best_move(state: &GameState, depth: i8) -> Option<Move> {
    let mut moves: Vec<Move> = state.generate_legal_moves().collect(); 

    match state.side() {
        Side::White => {
            let mut best_move_idx = None;
            let mut best_score = None;

            for (idx, r#move) in moves.iter().enumerate() {
                let score = minimax(&state.make_move(r#move), depth, false);
         
                if best_score.map_or(true, |b| score > b) {
                    best_move_idx = Some(idx);
                    best_score = Some(score);
                }
            }
            return best_move_idx.map(|idx| moves.swap_remove(idx))
        },
        Side::Black => {
            let mut best_move_idx = None;
            let mut best_score = None;

            for (idx, r#move) in moves.iter().enumerate() {
                let score = minimax(&state.make_move(r#move), depth, true);
                if best_score.map_or(true, |b| score < b) {
                    best_move_idx = Some(idx);
                    best_score = Some(score);
                }
            }
            return best_move_idx.map(|idx| moves.swap_remove(idx))
        }
    }
}

pub fn alphabeta_best_move(state: &GameState, depth: i8) -> Option<Move> {
    let mut moves: Vec<Move> = state.generate_legal_moves().collect(); 

    match state.side() {
        Side::White => {
            let mut best_move_idx = None;
            let mut best_score = None;

            for (idx, r#move) in moves.iter().enumerate() {
                let score = alphabeta(&state.make_move(r#move), depth, f64::NEG_INFINITY, f64::INFINITY, false);
         
                if best_score.map_or(true, |b| score > b) {
                    best_move_idx = Some(idx);
                    best_score = Some(score);
                }
            }
            return best_move_idx.map(|idx| moves.swap_remove(idx))
        },
        Side::Black => {
            let mut best_move_idx = None;
            let mut best_score = None;

            for (idx, r#move) in moves.iter().enumerate() {
                let score = alphabeta(&state.make_move(r#move), depth, f64::NEG_INFINITY, f64::INFINITY, true);
                if best_score.map_or(true, |b| score < b) {
                    best_move_idx = Some(idx);
                    best_score = Some(score);
                }
            }
            return best_move_idx.map(|idx| moves.swap_remove(idx))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn starting_position() {
        let game_state = GameState::default();

        println!("{}", game_state);
        let best_move = alphabeta_best_move(&game_state, 8).unwrap();
        dbg!(&best_move);
    } 

    #[test]
    fn mate_in_one_1() {
        let game_state = GameState::new(vec![
            (Piece::King, Side::Black, Square::d2),
            (Piece::Ma, Side::Black, Square::c3),
            (Piece::King, Side::White, Square::a1),
        ], Vec::new(), 
        [[0, 0, 0, 0].into(), [1, 0, 0, 0].into()],
        Side::Black
        );

        println!("{}", game_state);
        let best_move = minimax_best_move(&game_state, 1).unwrap();
        dbg!(&best_move);
        assert_eq!(
            best_move.source, None
            );
        assert_eq!(
            best_move.target, Square::a2
            );
        assert_eq!(
            best_move.piece, Piece::Wazir
            );
    } 
    
    #[test]
    fn mate_in_two_1() {
        let game_state = GameState::new(vec![
            (Piece::King, Side::White, Square::a1),
            (Piece::Pawn, Side::White, Square::a2),
            (Piece::Wazir, Side::White, Square::c3),
            (Piece::Pawn, Side::Black, Square::d2),
            (Piece::King, Side::Black, Square::c1),
            (Piece::Ferz, Side::Black, Square::d4),
            (Piece::Ferz, Side::Black, Square::a4),
        ], Vec::new(), 
        [[1, 2, 0, 0].into(), [0, 0, 0, 0].into()],
        Side::White
        );
        
        minimax(&game_state, 4, true);
        println!("{}", game_state);
        let best_move = minimax_best_move(&game_state, 2).unwrap();
        assert_eq!(best_move, Move { source: None, target: Square::b1, piece: Piece::Wazir, promoted_piece: None, capture: false});

        let game_state = game_state.make_move(&best_move);

        let best_move = minimax_best_move(&game_state, 2).unwrap();
        assert_eq!(best_move, Move { source: Some(Square::c1), target: Square::d1, piece: Piece::King, promoted_piece: None, capture: false});
        
        let game_state = game_state.make_move(&best_move);

        let best_move = minimax_best_move(&game_state, 2).unwrap();
        assert_eq!(best_move, Move { source: None, target: Square::b2, piece: Piece::Ma, promoted_piece: None, capture: false});
    }
    #[test]
    fn mate_in_two_2() {
        let game_state = GameState::new(vec![
            (Piece::King, Side::White, Square::b2),
            (Piece::Ma, Side::White, Square::a3),
            (Piece::King, Side::Black, Square::d3),
            (Piece::Pawn, Side::Black, Square::d2),
        ], Vec::new(), 
        [[1, 0, 0, 1].into(), [1, 1, 2, 1].into()],
        Side::White
        ); // many mate in 2's here e.g.
           // 1. P@c2+ Kd4
           // 2. W@d3#
           
           // Some mate in 3's here e.g.
           // 1. W@c3+ Kd4
           // 2. Wc4+ Kd3
           // 3. P@c2#
        let minimax_eval = minimax(&game_state, 3, false);
        dbg!(minimax_eval);
        let alphabeta_eval = alphabeta(&game_state, 7, f64::NEG_INFINITY, f64::INFINITY, false);
        dbg!(alphabeta_eval);

        assert_eq!(minimax_eval, alphabeta_eval);

        
        let best_move = minimax_best_move(&game_state, 3).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);

        let best_move = minimax_best_move(&game_state, 3).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);

        let best_move = minimax_best_move(&game_state, 3).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);    
    }

    #[test]
    fn mate_in_four() {
        let game_state = GameState::new(vec![
            (Piece::King, Side::White, Square::b1),
            (Piece::Ferz, Side::White, Square::d1),
            (Piece::Pawn, Side::White, Square::b3),
            (Piece::Ma, Side::Black, Square::b4),
            (Piece::Ma, Side::Black, Square::c1),
            (Piece::Wazir, Side::Black, Square::c2),
            (Piece::King, Side::Black, Square::d3),
        ], Vec::new(), 
        [[1, 0, 0, 0].into(), [1, 0, 1, 0].into()],
        Side::Black
        ); 
        // 1. ... W@b2+ 
        // 2. Ka1 Wa2+
        // 3. Kb1 Wcb2+
        // 4. Kxc1 F@d2#

        let minimax_eval = minimax(&game_state, 7, false);
        dbg!(minimax_eval);

        println!("{}", game_state); 
        let best_move = minimax_best_move(&game_state, 6).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = minimax_best_move(&game_state, 5).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = minimax_best_move(&game_state, 4).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = minimax_best_move(&game_state, 3).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);

        let best_move = minimax_best_move(&game_state, 2).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move); 
        println!("{}", game_state);

        let best_move = minimax_best_move(&game_state, 1).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move); 
        println!("{}", game_state);

        let best_move = minimax_best_move(&game_state, 0).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);    
        println!("{}", game_state);
    }
    
    #[test]
    fn mate_in_four_alphabeta() {
        let game_state = GameState::new(vec![
            (Piece::King, Side::White, Square::b1),
            (Piece::Ferz, Side::White, Square::d1),
            (Piece::Pawn, Side::White, Square::b3),
            (Piece::Ma, Side::Black, Square::b4),
            (Piece::Ma, Side::Black, Square::c1),
            (Piece::Wazir, Side::Black, Square::c2),
            (Piece::King, Side::Black, Square::d3),
        ], Vec::new(), 
        [[1, 0, 0, 0].into(), [1, 0, 1, 0].into()],
        Side::Black
        ); 
        // 1. ... W@b2+ 
        // 2. Ka1 Wa2+
        // 3. Kb1 Wcb2+
        // 4. Kxc1 F@d2#

        let alphabeta_eval = alphabeta(&game_state, 7, f64::NEG_INFINITY, f64::INFINITY, false);
        dbg!(alphabeta_eval);

        println!("{}", game_state); 
        let best_move = alphabeta_best_move(&game_state, 6).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = alphabeta_best_move(&game_state, 5).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = alphabeta_best_move(&game_state, 4).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);
        
        let best_move = alphabeta_best_move(&game_state, 3).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);
        println!("{}", game_state);

        let best_move = alphabeta_best_move(&game_state, 2).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move); 
        println!("{}", game_state);

        let best_move = alphabeta_best_move(&game_state, 1).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move); 
        println!("{}", game_state);

        let best_move = alphabeta_best_move(&game_state, 0).unwrap();
        dbg!(&best_move);
        let game_state = game_state.make_move(&best_move);    
        println!("{}", game_state);
    }
}
