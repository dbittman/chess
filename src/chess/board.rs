use std::fmt::Display;

use colored::Colorize;
use fen::{BoardState, FenError};

use crate::ab::{AlphaBeta, AlphaBetaResult, SearchSettings};

use super::{
    bitboard::BitBoard,
    moves::Move,
    piece::{Piece, ALL_PIECES, NR_PIECE_TYPES},
    side::Side,
    square::{File, Rank, Square, ALL_FILES, ALL_RANKS},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct CastleRights {
    val: u8,
}

impl Display for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.kingside() { "k" } else { "" },
            if self.queenside() { "q" } else { "" }
        )
    }
}

impl CastleRights {
    pub fn queenside(&self) -> bool {
        self.val & (1 << 1) == 0
    }

    pub fn kingside(&self) -> bool {
        self.val & (1 << 0) == 0
    }

    pub fn remove_kingside(&mut self) {
        self.val |= 1 << 0;
    }

    pub fn remove_queenside(&mut self) {
        self.val |= 1 << 1;
    }

    pub fn is_empty(&self) -> bool {
        self.val == 3
    }

    fn from_boardstate(value: &BoardState) -> [Self; 2] {
        let mut wcr = CastleRights::default();
        let mut bcr = CastleRights::default();
        if !value.black_can_oo {
            bcr.remove_kingside();
        }
        if !value.black_can_ooo {
            bcr.remove_queenside();
        }
        if !value.white_can_oo {
            wcr.remove_kingside();
        }
        if !value.white_can_ooo {
            wcr.remove_queenside();
        }
        [wcr, bcr]
    }
}

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct Board {
    pieces: [BitBoard; NR_PIECE_TYPES],
    sides: [BitBoard; 2],
    castle_rights: [CastleRights; 2],
    to_move: Side,
    enpassant: BitBoard,
    halfmove_clock: u64,
    fullmoves: u64,
}

impl Board {
    #[allow(dead_code)]
    pub fn color_pieces(&self, side: Side) -> BitBoard {
        self.sides[side]
    }

    pub fn adv_ply(&mut self, half: bool) {
        //TODO
        if self.to_move == Side::Black {
            self.fullmoves += 1;
        }
        if half {
            self.halfmove_clock += 1;
        }
        self.to_move = self.to_move.other();
    }

    #[allow(dead_code)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.pieces[piece]
    }

    pub fn check_piece(&self, sq: Square) -> Option<Piece> {
        ALL_PIECES.into_iter().find(|&p| self.pieces[p].get(sq))
    }

    pub fn piece(&self, sq: Square) -> Option<(Piece, Side)> {
        if self.sides[Side::White].get(sq) {
            Some((self.check_piece(sq).unwrap(), Side::White))
        } else if self.sides[Side::Black].get(sq) {
            Some((self.check_piece(sq).unwrap(), Side::Black))
        } else {
            None
        }
    }

    pub fn clear_square(&mut self, sq: Square) {
        for i in 0..NR_PIECE_TYPES {
            self.pieces[i].set(sq, false);
        }

        for i in 0..2 {
            self.sides[i].set(sq, false);
        }
    }

    pub fn set_square(&mut self, sq: Square, piece: Piece, side: Side) {
        self.clear_square(sq);
        self.pieces[piece].set(sq, true);
        self.sides[side].set(sq, true);
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let bs = fen::BoardState::from_fen(match fen {
            "startpos" => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            _ => fen,
        })?;
        let b = Board::from(bs);
        Ok(b)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in ALL_RANKS.into_iter().rev() {
            let mut empty = 0;
            for file in ALL_FILES {
                let sq = Square::from_rank_and_file(rank, file);
                if let Some((piece, side)) = self.piece(sq) {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    fen.push_str(&piece.to_char(side).to_string());
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank != Rank::new(1) {
                fen.push('/');
            }
        }
        fen.push(' ');
        fen.push_str(&self.to_move.to_char().to_string());
        fen.push(' ');
        fen.push_str(&self.castle_rights[Side::White].to_string().to_uppercase());
        fen.push_str(&self.castle_rights[Side::Black].to_string());
        if self.castle_rights(Side::White).is_empty() && self.castle_rights(Side::Black).is_empty()
        {
            fen.push('-');
        }
        fen.push(' ');
        fen.push_str(&self.enpassant.to_fen_string());
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmoves.to_string());
        fen
    }

    pub fn castle_rights(&self, side: Side) -> &CastleRights {
        &self.castle_rights[side]
    }

    pub fn castle_rights_mut(&mut self, side: Side) -> &mut CastleRights {
        &mut self.castle_rights[side]
    }

    pub fn enpassant(&self) -> &BitBoard {
        &self.enpassant
    }

    fn assert_piece_has_color(&self, piece: Piece) {
        for p in self.pieces(piece).into_iter() {
            assert!(self.sides[Side::White].get(p) || self.sides[Side::Black].get(p));
        }
    }

    pub fn assert_is_sane(&self) {
        for sq in self.sides[Side::White].into_iter() {
            self.piece(sq).unwrap();
        }

        for sq in self.sides[Side::Black].into_iter() {
            self.piece(sq).unwrap();
        }

        self.assert_piece_has_color(Piece::Bishop);
        self.assert_piece_has_color(Piece::Rook);
        self.assert_piece_has_color(Piece::Queen);
        self.assert_piece_has_color(Piece::King);
        self.assert_piece_has_color(Piece::Pawn);
        self.assert_piece_has_color(Piece::Knight);
    }

    pub fn to_move(&self) -> Side {
        self.to_move
    }

    pub fn set_enpassant(&mut self, enpassant: BitBoard) {
        self.enpassant = enpassant;
    }
}

impl From<BoardState> for Board {
    fn from(value: BoardState) -> Self {
        let mut b = Self {
            castle_rights: CastleRights::from_boardstate(&value),
            to_move: (&value.side_to_play).into(),
            enpassant: value.en_passant_square.map_or(BitBoard::default(), |s| {
                BitBoard::from_square(unsafe { Square::new(s) })
            }),
            halfmove_clock: value.halfmove_clock,
            fullmoves: value.fullmove_number,
            ..Default::default()
        };
        for (num, piece) in value.pieces.iter().enumerate() {
            if let Some(piece) = piece {
                b.set_square(
                    unsafe { Square::new(num as u8) },
                    (&piece.kind).into(),
                    (&piece.color).into(),
                )
            }
        }
        b
    }
}

#[allow(dead_code)]
fn fen_char(piece: Piece, side: Side) -> char {
    let c = match piece {
        Piece::Pawn => "Pp",
        Piece::Bishop => "Bb",
        Piece::Knight => "Nn",
        Piece::Rook => "Rr",
        Piece::Queen => "Qq",
        Piece::King => "Kk",
    };
    c.chars().nth(side.into()).unwrap()
}

fn utf8_char(piece: Piece, side: Side) -> char {
    let c = match piece {
        Piece::Pawn => "♙♟︎",
        Piece::Bishop => "♗♝",
        Piece::Knight => "♘♞",
        Piece::Rook => "♖♜",
        Piece::Queen => "♕♛",
        Piece::King => "♔♚",
    };
    c.chars().nth(side.into()).unwrap()
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "kr: white={}, black={}{}",
            self.castle_rights(Side::White),
            self.castle_rights(Side::Black),
            self.enpassant
                .to_square()
                .map_or("".to_owned(), |x| format!(" ; enpassant: {x}"))
        )?;
        for rank in (Rank::FIRST..=Rank::LAST).rev() {
            for file in File::A..=File::H {
                let sq = Square::from_rank_and_file(rank, file);
                if file == File::A && rank != Rank::LAST {
                    writeln!(f)?;
                }
                if file == File::A {
                    write!(f, "{} ", rank.0)?;
                }
                let p = self.piece(sq);
                if let Some((piece, side)) = p {
                    let s = format!(" {}  ", utf8_char(piece, side)).black();
                    write!(
                        f,
                        "{}",
                        if sq.is_dark() {
                            s.on_bright_blue()
                        } else {
                            s.on_white()
                        }
                    )?;
                } else {
                    let s = "    ".to_string();
                    write!(
                        f,
                        "{}",
                        if sq.is_dark() {
                            s.on_bright_blue()
                        } else {
                            s.on_white()
                        }
                    )?;
                }
            }
        }
        writeln!(f)?;
        write!(f, "   ")?;
        for file in File::A..=File::H {
            write!(f, "{file}   ")?;
        }
        writeln!(f)
    }
}

impl AlphaBeta for Board {
    fn is_terminal(&self) -> bool {
        false
    }

    fn score(&self) -> f32 {
        1.0
    }

    fn children(&self) -> Self::ItemIterator<'_> {
        self.legal_moves()
            .map(|m| (apply(self, m), MoveData { mv: m }))
    }

    type ItemIterator<'a> = impl Iterator<Item = (Board, Self::Data)> + 'a;

    type Data = MoveData;
}

pub struct MoveData {
    pub mv: Move,
}

fn apply(b: &Board, m: Move) -> Board {
    b.clone().apply_move(&m).unwrap()
}

impl Board {
    pub fn alphabeta(&self, settings: &SearchSettings, max: bool) -> AlphaBetaResult<MoveData> {
        crate::ab::alphabeta(
            self,
            settings,
            settings.depth,
            f32::NEG_INFINITY,
            f32::INFINITY,
            max,
        )
    }
}

#[cfg(test)]
mod test {
    // write a test that ensures the to_fen function works correctly
    use super::*;
    #[test]
    fn test_to_fen_start() {
        let b = Board::from_fen("startpos").unwrap();
        assert_eq!(
            b.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
