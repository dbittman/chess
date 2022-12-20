use std::{
    fmt::Display,
    iter::Step,
    mem::transmute,
    ops::{Index, IndexMut},
};

use vampirc_uci::UciPiece;

pub mod bitboard;
pub mod board;
pub mod engine;
pub mod moves;
pub mod piecemoves;

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

impl Into<usize> for Piece {
    fn into(self) -> usize {
        self as usize
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Side {
    #[default]
    White,
    Black,
}

impl Side {
    pub fn other(self) -> Self {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }
}

impl Into<usize> for Side {
    fn into(self) -> usize {
        self as usize
    }
}

impl<T> Index<Side> for [T] {
    type Output = T;

    fn index(&self, idx: Side) -> &Self::Output {
        &self[idx as usize]
    }
}

impl<T> IndexMut<Side> for [T] {
    fn index_mut(&mut self, idx: Side) -> &mut Self::Output {
        &mut self[idx as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

#[allow(dead_code)]
pub const LAST_SQUARE: Square = Square(63);
#[allow(dead_code)]
pub const FIRST_SQUARE: Square = Square(0);

impl Step for Square {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start.0, count).map(|x| Self(x))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start.0, count).map(|x| Self(x))
    }
}

impl Rank {
    pub const FIRST: Self = Self(1);
    pub const LAST: Self = Self(8);

    pub fn prev(self) -> Option<Self> {
        if self.0 == 1 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }

    pub fn next(self) -> Option<Self> {
        if self.0 == 8 {
            None
        } else {
            Some(Self(self.0 + 1))
        }
    }

    pub fn new(x: u8) -> Self {
        if x < 1 || x > 8 {
            panic!("bad rank {}", x);
        }
        Self(x)
    }

    pub fn allow_double_move(&self, side: Side) -> bool {
        match side {
            Side::White => self.0 == 2,
            Side::Black => self.0 == 7,
        }
    }

    pub fn is_promo_rank(&self, side: Side) -> bool {
        match side {
            Side::White => self.0 == 8,
            Side::Black => self.0 == 1,
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for Rank {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 1 || value > 8 {
            Err(())
        } else {
            Ok(Rank(value))
        }
    }
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            File::A => write!(f, "a"),
            File::B => write!(f, "b"),
            File::C => write!(f, "c"),
            File::D => write!(f, "d"),
            File::E => write!(f, "e"),
            File::F => write!(f, "f"),
            File::G => write!(f, "g"),
            File::H => write!(f, "h"),
        }
    }
}

impl TryFrom<char> for File {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            'a' => Self::A,
            'b' => Self::B,
            'c' => Self::C,
            'd' => Self::D,
            'e' => Self::E,
            'f' => Self::F,
            'g' => Self::G,
            'h' => Self::H,
            _ => return Err(()),
        })
    }
}

impl Square {
    pub fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Square((file as u8) + (rank.0 - 1) * 8)
    }

    pub fn rank(&self) -> Rank {
        Rank((self.0 / 8) + 1)
    }

    #[allow(dead_code)]
    pub fn file(&self) -> File {
        unsafe { transmute(self.0 % 8) }
    }

    pub fn is_dark(&self) -> bool {
        let r = self.rank().0 % 2;
        (self.0 + r) % 2 != 0
    }

    pub(super) unsafe fn new(v: u8) -> Self {
        Self(v)
    }

    pub fn is_kingmove_away(&self, other: Self) -> bool {
        for dir in ALL_DIRS {
            if let Some(x) = self.next_sq(dir) {
                if x == other {
                    return true;
                }
            }
        }
        false
    }

    pub fn next_sq_knight(self, dir: Direction) -> Option<Self> {
        let (rank, file) = (self.rank(), self.file());
        let next = match dir {
            Direction::Up => (rank.next().map(|x| x.next()).flatten(), file.next()),
            Direction::UpRight => (rank.next(), file.next().map(|x| x.next()).flatten()),
            Direction::Right => (rank.prev(), file.next().map(|x| x.next()).flatten()),
            Direction::DownRight => (rank.prev().map(|x| x.prev()).flatten(), file.next()),
            Direction::Down => (rank.prev().map(|x| x.prev()).flatten(), file.prev()),
            Direction::DownLeft => (rank.prev(), file.prev().map(|x| x.prev()).flatten()),
            Direction::Left => (rank.next(), file.prev().map(|x| x.prev()).flatten()),
            Direction::UpLeft => (rank.next().map(|x| x.next()).flatten(), file.prev()),
        };
        if let (Some(rank), Some(file)) = next {
            Some(Square::from_rank_and_file(rank, file))
        } else {
            None
        }
    }

    pub fn next_sq(self, dir: Direction) -> Option<Self> {
        let (rank, file) = (self.rank(), self.file());
        let next = match dir {
            Direction::Up => (rank.next(), Some(file)),
            Direction::UpRight => (rank.next(), file.next()),
            Direction::Right => (Some(rank), file.next()),
            Direction::DownRight => (rank.prev(), file.next()),
            Direction::Down => (rank.prev(), Some(file)),
            Direction::DownLeft => (rank.prev(), file.prev()),
            Direction::Left => (Some(rank), file.prev()),
            Direction::UpLeft => (rank.next(), file.prev()),
        };
        if let (Some(rank), Some(file)) = next {
            Some(Square::from_rank_and_file(rank, file))
        } else {
            None
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

impl Direction {
    pub fn is_diag(&self) -> bool {
        match self {
            Direction::Up => false,
            Direction::Right => false,
            Direction::Down => false,
            Direction::Left => false,
            _ => true,
        }
    }
}

pub const ALL_DIRS: [Direction; 8] = [
    Direction::Up,
    Direction::UpRight,
    Direction::Right,
    Direction::DownRight,
    Direction::Down,
    Direction::DownLeft,
    Direction::Left,
    Direction::UpLeft,
];

impl File {
    pub fn prev(self) -> Option<Self> {
        Some(match self {
            File::A => return None,
            File::B => File::A,
            File::C => File::B,
            File::D => File::C,
            File::E => File::D,
            File::F => File::E,
            File::G => File::F,
            File::H => File::G,
        })
    }

    pub fn next(self) -> Option<Self> {
        Some(match self {
            File::A => File::B,
            File::B => File::C,
            File::C => File::D,
            File::D => File::E,
            File::E => File::F,
            File::F => File::G,
            File::G => File::H,
            File::H => return None,
        })
    }
}

impl Step for Rank {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start.0, count).map(|x| Self(x))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start.0, count).map(|x| Self(x))
    }
}

impl Step for File {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&(*start as u8), &(*end as u8))
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start as u8, count).map(|x| unsafe { transmute(x) })
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start as u8, count).map(|x| unsafe { transmute(x) })
    }
}

impl From<&fen::Color> for Side {
    fn from(value: &fen::Color) -> Self {
        match value {
            fen::Color::White => Side::White,
            fen::Color::Black => Side::Black,
        }
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
