use std::ops::{IndexMut, Index};

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

impl From<Side> for usize {
    fn from(val: Side) -> Self {
        val as usize
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

impl From<&fen::Color> for Side {
    fn from(value: &fen::Color) -> Self {
        match value {
            fen::Color::White => Side::White,
            fen::Color::Black => Side::Black,
        }
    }
}
