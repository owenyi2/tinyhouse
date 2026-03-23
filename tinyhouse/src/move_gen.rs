use std::fmt::{self, Display};
use std::ops::{Index, IndexMut, Neg, Not};

#[derive(Default, Copy, Clone)]
pub struct BitBoard(pub u16);

#[macro_export]
macro_rules! get_bit {
    ($bitmap:expr, $square:expr) => {
        $bitmap.0 & (1 << $square as u16) != 0
    };
}

#[macro_export]
macro_rules! pop_bit {
    ($bitmap:expr, $square:expr) => {
        $bitmap.0 &= !(1 << $square as u16)
    };
}

#[macro_export]
macro_rules! set_bit {
    ($bitmap:expr, $square:expr) => {
        $bitmap.0 |= (1 << $square as u16)
    };
}

macro_rules! impl_bin_ops {
    (
        $ty:ty :
        $(
            $trait:ident :: $method:ident => |$lhs:ident, $rhs:ident| $body:expr
        ),* $(,)?
    ) => {
        $(
            impl std::ops::$trait for $ty {
                type Output = Self;
                fn $method(self, rhs: Self) -> Self {
                    let $lhs = self.0;
                    let $rhs = rhs.0;
                    Self($body)
                }
            }
        )*
    };
}

impl_bin_ops!(BitBoard:
    BitOr  :: bitor  => |a, b| a | b,
    BitAnd :: bitand => |a, b| a & b,
    BitXor :: bitxor => |a, b| a ^ b,
    Mul    :: mul    => |a, b| a.wrapping_mul(b),
);

macro_rules! impl_bit_assign_ops {
    (
        $ty:ty :
        $(
            $trait:ident :: $method:ident => $op:tt
        ),* $(,)?
    ) => {
        $(
            impl std::ops::$trait for $ty {
                fn $method(&mut self, rhs: Self) {
                    self.0 $op rhs.0;
                }
            }
        )*
    };
}

impl_bit_assign_ops!(BitBoard:
    BitOrAssign  :: bitor_assign  => |=,
    BitAndAssign :: bitand_assign => &=,
    BitXorAssign :: bitxor_assign => ^=,
);

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

fn get_ls1b_index(bitboard: BitBoard) -> Square {
    assert!(bitboard.0 != 0);
    let idx = ((bitboard.0 & bitboard.0.wrapping_neg()).wrapping_sub(1)).count_ones() as u32;
    Square::from(idx)
}

impl Iterator for BitBoard {
    type Item = Square; // square index

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let idx = get_ls1b_index(*self);
        pop_bit!(self, idx);

        Some(Square::from(idx))
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in 0..4 {
            for file in 0..4 {
                let square = 4 * rank + file;
                if file == 0 {
                    write!(f, "  {} ", 4 - rank)?;
                }
                write!(f, " {}", get_bit!(self, square) as u8)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "\n     a b c d")?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct AttackTable([BitBoard; 16]);

impl Index<Square> for AttackTable {
    type Output = BitBoard;

    #[inline]
    fn index(&self, square: Square) -> &Self::Output {
        &self.0[square as usize]
    }
}

const WAZIR_ATTACKS: AttackTable = AttackTable([
    BitBoard(0b0000000000010010),
    BitBoard(0b0000000000100101),
    BitBoard(0b0000000001001010),
    BitBoard(0b0000000010000100),
    BitBoard(0b0000000100100001),
    BitBoard(0b0000001001010010),
    BitBoard(0b0000010010100100),
    BitBoard(0b0000100001001000),
    BitBoard(0b0001001000010000),
    BitBoard(0b0010010100100000),
    BitBoard(0b0100101001000000),
    BitBoard(0b1000010010000000),
    BitBoard(0b0010000100000000),
    BitBoard(0b0101001000000000),
    BitBoard(0b1010010000000000),
    BitBoard(0b0100100000000000),
]);

const FERZ_ATTACKS: AttackTable = AttackTable([
    BitBoard(0b0000000000100000),
    BitBoard(0b0000000001010000),
    BitBoard(0b0000000010100000),
    BitBoard(0b0000000001000000),
    BitBoard(0b0000001000000010),
    BitBoard(0b0000010100000101),
    BitBoard(0b0000101000001010),
    BitBoard(0b0000010000000100),
    BitBoard(0b0010000000100000),
    BitBoard(0b0101000001010000),
    BitBoard(0b1010000010100000),
    BitBoard(0b0100000001000000),
    BitBoard(0b0000001000000000),
    BitBoard(0b0000010100000000),
    BitBoard(0b0000101000000000),
    BitBoard(0b0000010000000000),
]);

const PAWN_ATTACKS: [AttackTable; 2] = [
    AttackTable([
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000010),
        BitBoard(0b0000000000000101),
        BitBoard(0b0000000000001010),
        BitBoard(0b0000000000000100),
        BitBoard(0b0000000000100000),
        BitBoard(0b0000000001010000),
        BitBoard(0b0000000010100000),
        BitBoard(0b0000000001000000),
        BitBoard(0b0000001000000000),
        BitBoard(0b0000010100000000),
        BitBoard(0b0000101000000000),
        BitBoard(0b0000010000000000),
    ]),
    AttackTable([
        BitBoard(0b0000000000100000),
        BitBoard(0b0000000001010000),
        BitBoard(0b0000000010100000),
        BitBoard(0b0000000001000000),
        BitBoard(0b0000001000000000),
        BitBoard(0b0000010100000000),
        BitBoard(0b0000101000000000),
        BitBoard(0b0000010000000000),
        BitBoard(0b0010000000000000),
        BitBoard(0b0101000000000000),
        BitBoard(0b1010000000000000),
        BitBoard(0b0100000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
        BitBoard(0b0000000000000000),
    ]),
];
const MAGICS: [u16; 16] = [
    17424, 8768, 8724, 8320, 4736, 544, 1042, 521, 2080, 1040, 544, 160, 130, 65, 20, 8202,
];
const MA_ATTACKS: [[u16; 4]; 16] = [
    [576, 64, 512, 0],
    [1408, 128, 1280, 0],
    [2576, 2560, 16, 0],
    [1056, 32, 1024, 0],
    [9220, 8192, 1028, 0],
    [22536, 2056, 20480, 0],
    [41217, 257, 40960, 0],
    [16898, 514, 16384, 0],
    [16450, 2, 16448, 0],
    [32901, 5, 32896, 0],
    [4122, 10, 4112, 0],
    [8228, 8224, 4, 0],
    [1056, 32, 1024, 0],
    [2128, 80, 2048, 0],
    [416, 256, 160, 0],
    [576, 512, 64, 0],
];

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
#[repr(u8)]
pub enum Square {
    a4,
    b4,
    c4,
    d4,
    a3,
    b3,
    c3,
    d3,
    a2,
    b2,
    c2,
    d2,
    a1,
    b1,
    c1,
    d1,
}
impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = *self as i8;
        let rank = 4 - (i / 4);
        let file = (i % 4) as usize;
        let letters = ['a', 'b', 'c', 'd'];
        write!(f, "{}{}", letters[file], rank)
    }
}
impl From<u32> for Square {
    fn from(item: u32) -> Self {
        match item % 16 {
            0 => Square::a4,
            1 => Square::b4,
            2 => Square::c4,
            3 => Square::d4,
            4 => Square::a3,
            5 => Square::b3,
            6 => Square::c3,
            7 => Square::d3,
            8 => Square::a2,
            9 => Square::b2,
            10 => Square::c2,
            11 => Square::d2,
            12 => Square::a1,
            13 => Square::b1,
            14 => Square::c1,
            15 => Square::d1,
            _ => Square::a1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    King,
    Wazir,
    Ma,
    Ferz,
    Pawn,
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Piece::King => write!(f, "K"),
            Piece::Wazir => write!(f, "W"),
            Piece::Ma => write!(f, "M"),
            Piece::Ferz => write!(f, "F"),
            Piece::Pawn => write!(f, "P"),
        }
    }
}

impl Piece {
    pub const ALL: [Self; 5] = [
        Piece::King,
        Piece::Wazir,
        Piece::Ma,
        Piece::Ferz,
        Piece::Pawn,
    ];
}

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    White,
    Black,
}

impl Neg for Side {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::White => write!(f, "White"),
            Side::Black => write!(f, "Black"),
        }
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Inventory {
    inventory: u8,
}

impl From<u8> for Inventory {
    fn from(item: u8) -> Self {
        Inventory { inventory: item }
    }
}

impl Inventory {
    pub fn get(&self, piece: Piece) -> u8 {
        assert!(piece != Piece::King);
        use Piece::*;
        match piece {
            Wazir => (self.inventory >> 6) & 0b11,
            Ma => (self.inventory >> 4) & 0b11,
            Ferz => (self.inventory >> 2) & 0b11,
            Pawn => (self.inventory) & 0b11,
            _ => unreachable!(),
        }
    }
    fn increment(&mut self, piece: Piece) {
        assert!(piece != Piece::King);
        use Piece::*;
        match piece {
            Wazir => self.inventory += 1 << 6,
            Ma => self.inventory += 1 << 4,
            Ferz => self.inventory += 1 << 2,
            Pawn => self.inventory += 1,
            _ => unreachable!(),
        }
    }
    fn decrement(&mut self, piece: Piece) {
        assert!(piece != Piece::King);
        use Piece::*;
        match piece {
            Wazir => self.inventory -= 1 << 6,
            Ma => self.inventory -= 1 << 4,
            Ferz => self.inventory -= 1 << 2,
            Pawn => self.inventory -= 1,
            _ => unreachable!(),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct PieceBoards {
    pub king: BitBoard,
    pub wazir: BitBoard,
    pub ma: BitBoard,
    pub ferz: BitBoard,
    pub pawn: BitBoard,
}

impl Index<Piece> for PieceBoards {
    type Output = BitBoard;

    #[inline]
    fn index(&self, piece: Piece) -> &Self::Output {
        match piece {
            Piece::King => &self.king,
            Piece::Wazir => &self.wazir,
            Piece::Ma => &self.ma,
            Piece::Ferz => &self.ferz,
            Piece::Pawn => &self.pawn,
        }
    }
}
impl IndexMut<Piece> for PieceBoards {
    #[inline]
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        match piece {
            Piece::King => &mut self.king,
            Piece::Wazir => &mut self.wazir,
            Piece::Ma => &mut self.ma,
            Piece::Ferz => &mut self.ferz,
            Piece::Pawn => &mut self.pawn,
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Occupancies {
    pub white: BitBoard,
    pub black: BitBoard,
}
impl Index<Side> for Occupancies {
    type Output = BitBoard;

    fn index(&self, side: Side) -> &Self::Output {
        match side {
            Side::White => &self.white,
            Side::Black => &self.black,
        }
    }
}
impl IndexMut<Side> for Occupancies {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        match side {
            Side::White => &mut self.white,
            Side::Black => &mut self.black,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub source: Option<Square>,
    pub target: Square,
    pub piece: Piece,
    pub promoted_piece: Option<Piece>,
    pub capture: bool,
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let promoted = match self.promoted_piece {
            None => String::from(""),
            Some(piece) => format!("={}", piece),
        };
        let capture = match self.capture {
            true => String::from("x"),
            false => String::from(""),
        };
        match self.source {
            None => write!(f, "{}@{}{}", self.piece, self.target, promoted)?,
            Some(_) => write!(f, "{}{}{}{}", self.piece, capture, self.target, &promoted)?,
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct GameState {
    bitboards: PieceBoards, // King, Wazir, Ma, Ferz, Pawn
    occupancies: Occupancies,
    former_pawns: BitBoard,
    inventory: [Inventory; 2],
    side: Side,
}

impl GameState {
    pub fn new(
        piece_map: Vec<(Piece, Side, Square)>,
        former_pawns_list: Vec<Square>,
        inventory_map: [(u8, u8, u8, u8); 2],
        side: Side,
    ) -> Self {
        let mut bitboards = PieceBoards::default();
        let mut occupancies = Occupancies::default();
        for (piece, side, square) in piece_map {
            set_bit!(bitboards[piece], square);
            set_bit!(occupancies[side], square);
        }
        let mut former_pawns = BitBoard(0);
        for square in former_pawns_list {
            set_bit!(former_pawns, square)
        }
        let mut inventory = [Inventory::from(0), Inventory::from(0)];
        for _ in 0..inventory_map[0].0 {
            inventory[0].increment(Piece::Wazir)
        }
        for _ in 0..inventory_map[0].1 {
            inventory[0].increment(Piece::Ma)
        }
        for _ in 0..inventory_map[0].2 {
            inventory[0].increment(Piece::Ferz)
        }
        for _ in 0..inventory_map[0].3 {
            inventory[0].increment(Piece::Pawn)
        }

        for _ in 0..inventory_map[1].0 {
            inventory[1].increment(Piece::Wazir)
        }
        for _ in 0..inventory_map[1].1 {
            inventory[1].increment(Piece::Ma)
        }
        for _ in 0..inventory_map[1].2 {
            inventory[1].increment(Piece::Ferz)
        }
        for _ in 0..inventory_map[1].3 {
            inventory[1].increment(Piece::Pawn)
        }

        GameState {
            bitboards,
            occupancies,
            former_pawns,
            inventory,
            side,
        }
    }

    pub fn bitboards(&self) -> &PieceBoards {
        &self.bitboards
    }
    pub fn occupancies(&self) -> &Occupancies {
        &self.occupancies
    }
    pub fn inventory(&self) -> &[Inventory; 2] {
        &self.inventory
    }
    pub fn side(&self) -> &Side {
        &self.side
    }

    fn placement_moves(&self, output: &mut Vec<Move>) {
        let available_squares = !(self.occupancies[Side::White] | self.occupancies[Side::Black]);
        let inventory = self.inventory[self.side as usize];

        for piece in [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn] {
            if inventory.get(piece) == 0 {
                continue;
            }

            for square in available_squares {
                if self.side == Side::White && piece == Piece::Pawn && square <= Square::d4 {
                    continue;
                }
                if self.side == Side::Black && piece == Piece::Pawn && square >= Square::a1 {
                    continue;
                }

                output.push(Move {
                    source: None,
                    target: square,
                    piece: piece,
                    promoted_piece: None,
                    capture: false,
                });
            }
        }
    }

    fn enemy_attack_mask(self, side: Side) -> BitBoard {
        let enemy = -side;
        let enemies = self.occupancies[enemy];
        let mut attack_mask = BitBoard(0);

        // KING
        let bitboard = self.bitboards[Piece::King] & enemies;
        for source_square in bitboard {
            let attacks = WAZIR_ATTACKS[source_square] | FERZ_ATTACKS[source_square];
            attack_mask |= attacks;
        }
        // WAZIR
        let bitboard = self.bitboards[Piece::Wazir] & enemies;
        for source_square in bitboard {
            let attacks = WAZIR_ATTACKS[source_square];
            attack_mask |= attacks;
        }
        // FERZ
        let bitboard = self.bitboards[Piece::Ferz] & enemies;
        for source_square in bitboard {
            let attacks = FERZ_ATTACKS[source_square];
            attack_mask |= attacks;
        }
        // PAWN
        let bitboard = self.bitboards[Piece::Pawn] & enemies;
        for source_square in bitboard {
            let attacks = PAWN_ATTACKS[enemy as usize][source_square];
            attack_mask |= attacks;
        }
        // MA
        let both = self.occupancies[Side::White] | self.occupancies[Side::Black];
        let king = self.bitboards[Piece::King] & self.occupancies[side];
        let bitboard = self.bitboards[Piece::Ma] & enemies;
        for source_square in bitboard {
            let attacks = Self::lookup_ma_attacks(source_square, both & !king);
            attack_mask |= attacks;
        }
        attack_mask
    }
    fn king_moves(&self, output: &mut Vec<Move>) {
        let bitboard = self.bitboards[Piece::King] & self.occupancies[self.side];
        for source_square in bitboard {
            let attacks = (FERZ_ATTACKS[source_square] | WAZIR_ATTACKS[source_square])
                & !self.occupancies[self.side]
                & !self.enemy_attack_mask(self.side);
            for target_square in attacks {
                output.push(Move {
                    source: Some(source_square),
                    target: target_square,
                    piece: Piece::King,
                    promoted_piece: None,
                    capture: get_bit!(self.occupancies[-self.side], target_square),
                });
            }
        }
    }
    fn wazir_ferz_moves(&self, piece: Piece, attack_table: AttackTable, output: &mut Vec<Move>) {
        let bitboard = self.bitboards[piece] & self.occupancies[self.side];

        for source_square in bitboard {
            let attacks = attack_table[source_square] & !self.occupancies[self.side];
            for target_square in attacks {
                output.push(Move {
                    source: Some(source_square),
                    target: target_square,
                    piece: piece,
                    promoted_piece: None,
                    capture: get_bit!(self.occupancies[-self.side], target_square),
                });
            }
        }
    }
    fn lookup_ma_attacks(square: Square, block_mask: BitBoard) -> BitBoard {
        let magic_index = ((block_mask & WAZIR_ATTACKS[square])
            * BitBoard(MAGICS[square as usize]))
        .0 >> (16 - 2);

        BitBoard(MA_ATTACKS[square as usize][magic_index as usize])
    }
    fn ma_moves(&self, output: &mut Vec<Move>) {
        let bitboard = self.bitboards[Piece::Ma] & self.occupancies[self.side];
        let both = self.occupancies[Side::White] | self.occupancies[Side::Black];

        for source_square in bitboard {
            let attacks =
                Self::lookup_ma_attacks(source_square, both) & !self.occupancies[self.side];
            for target_square in attacks {
                output.push(Move {
                    source: Some(source_square),
                    target: target_square,
                    piece: Piece::Ma,
                    promoted_piece: None,
                    capture: get_bit!(self.occupancies[-self.side], target_square),
                });
            }
        }
    }
    fn pawn_moves(&self, output: &mut Vec<Move>) {
        let bitboard = self.bitboards[Piece::Pawn] & self.occupancies[self.side];
        let both = self.occupancies[Side::White] | self.occupancies[Side::Black];

        for source_square in bitboard {
            // Quiet moves
            let target_square: i16 = source_square as i16 - 4;

            let (promotion, target_square, forward_available) = match self.side {
                Side::White => {
                    let promotion = (Square::a3 <= source_square) && (source_square <= Square::d3);
                    let target_square: i16 = source_square as i16 - 4;
                    let forward_available =
                        (target_square >= 0) && !get_bit!(both, Square::from(target_square as u32));
                    (promotion, target_square, forward_available)
                }
                Side::Black => {
                    let promotion = (Square::a2 <= source_square) && (source_square <= Square::d2);
                    let target_square: i16 = source_square as i16 + 4;
                    let forward_available =
                        (target_square < 16) && !get_bit!(both, Square::from(target_square as u32));
                    (promotion, target_square, forward_available)
                }
            };
            let target_square = Square::from(target_square as u32);
            if forward_available {
                if promotion {
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Wazir),
                        capture: false,
                    });
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Ferz),
                        capture: false,
                    });
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Ma),
                        capture: false,
                    });
                } else {
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: None,
                        capture: false,
                    });
                }
            }
            // Captures
            let attacks =
                PAWN_ATTACKS[self.side as usize][source_square] & self.occupancies[-self.side];
            for target_square in attacks {
                if promotion {
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Wazir),
                        capture: true,
                    });
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Ferz),
                        capture: true,
                    });
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: Some(Piece::Ma),
                        capture: true,
                    });
                } else {
                    output.push(Move {
                        source: Some(source_square),
                        target: target_square,
                        piece: Piece::Pawn,
                        promoted_piece: None,
                        capture: true,
                    });
                }
            }
        }
    }
    fn generate_moves(&self, output: &mut Vec<Move>) {
        self.placement_moves(output);
        self.king_moves(output);
        self.wazir_ferz_moves(Piece::Wazir, WAZIR_ATTACKS, output);
        self.wazir_ferz_moves(Piece::Ferz, FERZ_ATTACKS, output);
        self.ma_moves(output);
        self.pawn_moves(output);
    }
    pub fn make_move(&self, move_: &Move) -> Self {
        let mut new_state = self.clone();

        if let Some(source_square) = move_.source {
            if move_.capture {
                let mut captured_piece = None;
                for piece in Piece::ALL {
                    if get_bit!(new_state.bitboards[piece], move_.target) {
                        assert!(piece != Piece::King);
                        captured_piece = Some(piece);
                        break;
                    }
                }
                let captured_piece = captured_piece.expect("`move_.capture == true`");
                if get_bit!(new_state.former_pawns, move_.target) {
                    // captured a former pawn
                    new_state.inventory[new_state.side as usize].increment(Piece::Pawn);
                    pop_bit!(new_state.former_pawns, move_.target);
                } else {
                    new_state.inventory[new_state.side as usize].increment(captured_piece);
                }
                // additionally remove enemy piece
                pop_bit!(new_state.bitboards[captured_piece], move_.target);
                // additionally remove enemy occupancy from target
                pop_bit!(new_state.occupancies[-new_state.side], move_.target);
            }
            // handle promotions
            if let Some(promoted_piece) = move_.promoted_piece {
                pop_bit!(new_state.bitboards[move_.piece], source_square);
                set_bit!(new_state.bitboards[promoted_piece], move_.target);
                set_bit!(new_state.former_pawns, move_.target);
            } else {
                pop_bit!(new_state.bitboards[move_.piece], source_square); // move piece
                set_bit!(new_state.bitboards[move_.piece], move_.target); // move piece
            }
            // remove occupancy from source
            pop_bit!(new_state.occupancies[new_state.side], source_square);
            // set occupancy on target
            set_bit!(new_state.occupancies[new_state.side], move_.target);

            // move former pawn
            if get_bit!(new_state.former_pawns, source_square) {
                pop_bit!(new_state.former_pawns, source_square);
                set_bit!(new_state.former_pawns, move_.target);
            }
        } else {
            // placement moves

            assert!(new_state.inventory[new_state.side as usize].get(move_.piece) > 0);
            new_state.inventory[new_state.side as usize].decrement(move_.piece);
            // place piece
            set_bit!(new_state.bitboards[move_.piece], move_.target);
            // set occupancy on target
            set_bit!(new_state.occupancies[new_state.side], move_.target);
        }
        new_state.side = -new_state.side;
        new_state
    }
    pub fn detect_check(&self, side: Side) -> bool {
        let attack_mask = self.enemy_attack_mask(side);
        return (attack_mask & self.bitboards[Piece::King] & self.occupancies[side]).0 != 0;
    }

    pub fn generate_legal_moves(&self) -> impl Iterator<Item = Move> {
        let mut moves = Vec::with_capacity(32);
        self.generate_moves(&mut moves);

        moves
            .into_iter()
            .filter(move |move_| !self.make_move(move_).detect_check(self.side))
    }
    pub fn detect_mate(&self) -> Option<bool> {
        if self.generate_legal_moves().peekable().next().is_none() {
            Some(self.detect_check(self.side)) // checkmate if true else stalemate
        } else {
            None
        }
    }

}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            bitboards: PieceBoards {
                king: BitBoard(1 << Square::a1 as u8) | BitBoard(1 << Square::d4 as u8),
                wazir: BitBoard(1 << Square::b1 as u8) | BitBoard(1 << Square::c4 as u8),
                ma: BitBoard(1 << Square::c1 as u8) | BitBoard(1 << Square::b4 as u8),
                ferz: BitBoard(1 << Square::d1 as u8) | BitBoard(1 << Square::a4 as u8),
                pawn: BitBoard(1 << Square::a2 as u8) | BitBoard(1 << Square::d3 as u8),
            },
            occupancies: Occupancies {
                white: BitBoard(0b1111_0001_0000_0000u16),
                black: BitBoard(0b0000_0000_1000_1111u16),
            },
            former_pawns: BitBoard(0),
            inventory: [Inventory::from(0), Inventory::from(0)],
            side: Side::White,
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?;
        for rank in 0..4 {
            for file in 0..4 {
                let square = 4 * rank + file;
                if file == 0 {
                    write!(f, "  {} ", 4 - rank)?;
                }
                let mut occupied = false;
                for piece in Piece::ALL {
                    if get_bit!(self.bitboards[piece], square) {
                        if get_bit!(self.occupancies[Side::White], square) {
                            write!(f, " {}", piece.to_string())?;
                        } else {
                            write!(f, " {}", piece.to_string().to_lowercase())?;
                        }
                        occupied = true;
                        break;
                    }
                }
                if !occupied {
                    write!(f, " .")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n     a b c d\n")?;
        write!(f, "\n")?;
        write!(f, "{} to play", self.side)?;
        write!(f, "\n")?;

        for piece in [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn] {
            let count = self.inventory[0].get(piece);
            if count > 0 {
                write!(f, "  {}: {}\n", piece.to_string(), count)?;
            }
        }
        for piece in [Piece::Wazir, Piece::Ma, Piece::Ferz, Piece::Pawn] {
            let count = self.inventory[1].get(piece);
            if count > 0 {
                write!(f, "  {}: {}\n", piece.to_string().to_lowercase(), count)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perft(game_state: GameState, depth: i32) -> usize {
        if depth == 0 {
            return 1;
        }
        let mut count = 0;
        let mut moves: Vec<Move> = game_state.generate_legal_moves().collect();

        moves.sort_by_key(|m| m.to_string());
        for move_ in moves {
            let temp_count = perft(game_state.make_move(&move_), depth - 1);
            count += temp_count;
            // println!("{}{}: {}", "\t".repeat((8 - depth).try_into().unwrap()), move_, temp_count);
        }
        return count;
        // rewrite Move::Display to match your python one
        // create a better perft function that lists out the moves in alphabetical order
        // and the number of moves that follow
        //
        // wb2: 10
        //   Kb2: 3
        //     pa1: 2
        //     pb1: 1
        //   P@a1: 4
        //
        //   Fa2: 3
        // kb1: 5
        // ...
    }
    fn recursive(game_state: GameState, total_depth: i32, remaining_depth: i32) {
        if remaining_depth == -1 {
            return
        }
        for move_ in game_state.generate_legal_moves() {
            let new_state = game_state.make_move(&move_);
            println!("{}move: {} {}", "\t".repeat((total_depth - remaining_depth).try_into().unwrap()), move_, perft(new_state, remaining_depth));
            recursive(new_state, total_depth, remaining_depth - 1);
        }
    }

    #[test]
    fn detailed_test() {
        let game_state = GameState::default();
        recursive(game_state, 3, 3);
    }


    #[test]
    fn performance_test() {
        let game_state = GameState::default();
        // for move_ in game_state.generate_legal_moves() {
        //     println!("move: {}", move_);
        //     let new_state = game_state.make_move(&move_);
        //     println!("resulting game state:\n{}", new_state);
        //     for move_ in new_state.generate_legal_moves() {
        //         println!("{}", move_);
        //     }
        // }
        for depth in 0..=7 {
            let count = perft(game_state, depth);
            println!("{} moves at depth {}", count, depth);
        }
    }

    #[test]
    fn test_1() {
        let mut game_state = GameState::default();
        println!("{}", game_state);
        println!("{}", game_state.occupancies[Side::White]);
        println!("{}", game_state.occupancies[Side::Black]);

        // 1. Kb2 Fb3
        let r#move = Move {
            source: Some(Square::a1),
            target: Square::b2,
            piece: Piece::King,
            promoted_piece: None,
            capture: false,
        };
        game_state = game_state.make_move(&r#move);
        println!("{}", game_state);
        println!("{}", game_state.occupancies[Side::White]);
        println!("{}", game_state.occupancies[Side::Black]);
        let r#move = Move {
            source: Some(Square::a4),
            target: Square::b3,
            piece: Piece::Ferz,
            promoted_piece: None,
            capture: false,
        };
        game_state = game_state.make_move(&r#move);
        println!("{}", game_state);
        println!("{}", game_state.occupancies[Side::White]);
        println!("{}", game_state.occupancies[Side::Black]);

        // 2.
        for r#move in game_state.generate_legal_moves() {
            println!("{}", r#move);
        }
    }
}
