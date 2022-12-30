#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
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
