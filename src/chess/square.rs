use std::{fmt::Display, iter::Step, mem::transmute};

use memoize::lazy_static::lazy_static;

use super::{
    direction::{Direction, ALL_DIRS},
    side::Side,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Square(pub u8);

#[allow(dead_code)]
pub const LAST_SQUARE: Square = Square(63);
#[allow(dead_code)]
pub const FIRST_SQUARE: Square = Square(0);

impl Step for Square {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&start.0, &end.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start.0, count).map(Self)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start.0, count).map(Self)
    }
}

impl Rank {
    pub const FIRST: Self = Self(1);
    pub const LAST: Self = Self(8);

    pub const fn prev(self) -> Option<Self> {
        if self.0 == 1 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }

    pub const fn next(self) -> Option<Self> {
        if self.0 == 8 {
            None
        } else {
            Some(Self(self.0 + 1))
        }
    }

    pub const fn new(x: u8) -> Self {
        if x < 1 || x > 8 {
            panic!("bad rank");
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
        if !(1..=8).contains(&value) {
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

impl From<File> for char {
    fn from(f: File) -> Self {
        match f {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
        }
    }
}

impl Square {
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Square((file as u8) + (rank.0 - 1) * 8)
    }

    pub const fn rank(&self) -> Rank {
        Rank((self.0 / 8) + 1)
    }

    #[allow(dead_code)]
    pub const fn file(&self) -> File {
        unsafe { transmute(self.0 % 8) }
    }

    pub fn is_dark(&self) -> bool {
        let r = self.rank().0 % 2;
        (self.0 + r) % 2 != 0
    }

    pub(super) const unsafe fn new(v: u8) -> Self {
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
        (&*NEXT_SQ_TABLE_KNIGHT)[self.0 as usize][<Direction as Into<usize>>::into(dir)]
    }

    pub fn next_sq(self, dir: Direction) -> Option<Self> {
        (&*NEXT_SQ_TABLE)[self.0 as usize][<Direction as Into<usize>>::into(dir)]
    }
}

fn build_dir_table(sq: usize) -> [Option<Square>; 8] {
    let build_dir_table_entry = |d: usize| -> Option<Square> {
        do_next_sq(unsafe { Square::new(sq as u8) }, Direction::from_usize(d))
    };
    array_const_fn_init::array_const_fn_init!(build_dir_table_entry; 8)
}

lazy_static! {
    static ref NEXT_SQ_TABLE: [[Option<Square>; 8]; 64] =
        array_const_fn_init::array_const_fn_init!(build_dir_table; 64);
}

fn build_dir_table_knight(sq: usize) -> [Option<Square>; 8] {
    let build_dir_table_entry_k = |d: usize| -> Option<Square> {
        do_next_sq_knight(unsafe { Square::new(sq as u8) }, Direction::from_usize(d))
    };
    array_const_fn_init::array_const_fn_init!(build_dir_table_entry_k; 8)
}

lazy_static! {
    static ref NEXT_SQ_TABLE_KNIGHT: [[Option<Square>; 8]; 64] =
        array_const_fn_init::array_const_fn_init!(build_dir_table_knight; 64);
}

pub const fn do_next_sq(sq: Square, dir: Direction) -> Option<Square> {
    let (rank, file) = (sq.rank(), sq.file());
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

pub fn do_next_sq_knight(sq: Square, dir: Direction) -> Option<Square> {
    let (rank, file) = (sq.rank(), sq.file());
    let next = match dir {
        Direction::Up => (rank.next().and_then(|x| x.next()), file.next()),
        Direction::UpRight => (rank.next(), file.next().and_then(|x| x.next())),
        Direction::Right => (rank.prev(), file.next().and_then(|x| x.next())),
        Direction::DownRight => (rank.prev().and_then(|x| x.prev()), file.next()),
        Direction::Down => (rank.prev().and_then(|x| x.prev()), file.prev()),
        Direction::DownLeft => (rank.prev(), file.prev().and_then(|x| x.prev())),
        Direction::Left => (rank.next(), file.prev().and_then(|x| x.prev())),
        Direction::UpLeft => (rank.next().and_then(|x| x.next()), file.prev()),
    };
    if let (Some(rank), Some(file)) = next {
        Some(Square::from_rank_and_file(rank, file))
    } else {
        None
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl File {
    pub const fn prev(self) -> Option<Self> {
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

    pub const fn next(self) -> Option<Self> {
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
        u8::forward_checked(start.0, count).map(Self)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start.0, count).map(Self)
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

pub const ALL_RANKS: [Rank; 8] = [
    Rank::new(1),
    Rank::new(2),
    Rank::new(3),
    Rank::new(4),
    Rank::new(5),
    Rank::new(6),
    Rank::new(7),
    Rank::new(8),
];
pub const ALL_FILES: [File; 8] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];
