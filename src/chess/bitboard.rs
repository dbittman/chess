use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitXorAssign, Not},
};

use colored::Colorize;

use super::{File, Rank, Square};

#[derive(Debug, Clone, Copy, Default)]
pub struct BitBoard(u64);

#[allow(dead_code)]
pub const EMPTY: BitBoard = BitBoard(0);
#[allow(dead_code)]
pub const FULL: BitBoard = BitBoard(!0);

impl BitBoard {
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.0 = 0
    }

    pub fn set(&mut self, sq: Square, set: bool) {
        if set {
            self.0 |= 1 << sq.0;
        } else {
            self.0 &= !(1 << sq.0);
        }
    }

    pub fn get(&self, sq: Square) -> bool {
        (self.0 & (1 << sq.0)) != 0
    }

    #[inline]
    pub fn to_square(&self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(unsafe { Square::new(self.0.trailing_zeros() as u8) })
        }
    }

    pub fn from_square(sq: Square) -> BitBoard {
        BitBoard(1u64 << sq.0)
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;

    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter { board: self }
    }
}

impl From<Square> for BitBoard {
    fn from(value: Square) -> Self {
        Self(1 << value.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, other: BitBoard) {
        self.0 &= other.0;
    }
}

impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, other: BitBoard) {
        self.0 ^= other.0;
    }
}

pub struct BitBoardIter {
    board: BitBoard,
}

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.board.0 == 0 {
            None
        } else {
            let result = self.board.to_square().unwrap();
            self.board ^= BitBoard::from_square(result);
            Some(result)
        }
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (Rank::FIRST..=Rank::LAST).rev() {
            for file in File::A..=File::H {
                let sq = Square::from_rank_and_file(rank, file);
                if file == File::A && rank != Rank::LAST {
                    writeln!(f)?;
                }
                if file == File::A {
                    write!(f, "{} ", rank.0)?;
                }
                let p = self.get(sq);
                if p {
                    let s = " xx ".to_string().black();
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
