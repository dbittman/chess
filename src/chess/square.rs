use std::{fmt::Display, iter::Step, mem::transmute};

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

    #[inline]
    pub fn prev(self) -> Option<Self> {
        if self.0 == 1 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }

    #[inline]
    pub fn next(self) -> Option<Self> {
        if self.0 == 8 {
            None
        } else {
            Some(Self(self.0 + 1))
        }
    }

    #[inline]
    pub fn new(x: u8) -> Self {
        if !(1..=8).contains(&x) {
            panic!("bad rank {x}");
        }
        Self(x)
    }

    #[inline]
    pub fn allow_double_move(&self, side: Side) -> bool {
        match side {
            Side::White => self.0 == 2,
            Side::Black => self.0 == 7,
        }
    }

    #[inline]
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

impl Square {
    #[inline]
    pub fn from_rank_and_file(rank: Rank, file: File) -> Self {
        Square((file as u8) + (rank.0 - 1) * 8)
    }

    #[inline]
    pub fn rank(&self) -> Rank {
        Rank((self.0 / 8) + 1)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn file(&self) -> File {
        unsafe { transmute(self.0 % 8) }
    }

    #[inline]
    pub fn is_dark(&self) -> bool {
        let r = self.rank().0 % 2;
        (self.0 + r) % 2 != 0
    }

    #[inline]
    pub(super) unsafe fn new(v: u8) -> Self {
        Self(v)
    }

    #[inline]
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

    #[inline]
    pub fn next_sq_knight(self, dir: Direction) -> Option<Self> {
        let (rank, file) = (self.rank(), self.file());
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

    #[inline]
    pub fn next_sq(self, dir: Direction) -> Option<Self> {
        do_next_sq(self, dir)
    }
}

pub fn do_next_sq(sq: Square, dir: Direction) -> Option<Square> {
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

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl File {
    #[inline]
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

    #[inline]
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
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&start.0, &end.0)
    }

    #[inline]
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start.0, count).map(Self)
    }

    #[inline]
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start.0, count).map(Self)
    }
}

impl Step for File {
    #[inline]
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        u8::steps_between(&(*start as u8), &(*end as u8))
    }

    #[inline]
    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        u8::forward_checked(start as u8, count).map(|x| unsafe { transmute(x) })
    }

    #[inline]
    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        u8::backward_checked(start as u8, count).map(|x| unsafe { transmute(x) })
    }
}
