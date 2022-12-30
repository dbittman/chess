use std::ops::{IndexMut, Index};

use vampirc_uci::UciPiece;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

impl From<UciPiece> for Piece {
    fn from(value: UciPiece) -> Self {
        match value {
            UciPiece::Pawn => Self::Pawn,
            UciPiece::Knight => Self::Knight,
            UciPiece::Bishop => Self::Bishop,
            UciPiece::Rook => Self::Rook,
            UciPiece::Queen => Self::Queen,
            UciPiece::King => Self::King,
        }
    }
}

pub const ALL_PIECES: [Piece; NR_PIECE_TYPES] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

pub const NR_PIECE_TYPES: usize = 6;

impl From<Piece> for usize {
    fn from(val: Piece) -> Self {
        val as usize
    }
}

impl<T> Index<Piece> for [T] {
    type Output = T;

    fn index(&self, idx: Piece) -> &Self::Output {
        &self[idx as usize]
    }
}

impl<T> IndexMut<Piece> for [T] {
    fn index_mut(&mut self, idx: Piece) -> &mut Self::Output {
        &mut self[idx as usize]
    }
}

impl From<&fen::PieceKind> for Piece {
    fn from(value: &fen::PieceKind) -> Self {
        match value {
            fen::PieceKind::Pawn => Self::Pawn,
            fen::PieceKind::Knight => Self::Knight,
            fen::PieceKind::Bishop => Self::Bishop,
            fen::PieceKind::Rook => Self::Rook,
            fen::PieceKind::Queen => Self::Queen,
            fen::PieceKind::King => Self::King,
        }
    }
}
