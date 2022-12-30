use super::{board::Board, moves::Move};

pub trait AlphaBeta {
    type ItemIterator<'a>: Iterator<Item = Self> + 'a
    where
        Self: 'a;

    fn is_terminal(&self) -> bool;
    fn score(&self) -> f32;
    fn children(&self) -> Self::ItemIterator<'_>;
}

pub struct SearchSettings {
    pub divide: bool,
    pub ab_prune: bool,
    pub depth: u64,
}

impl SearchSettings {
    pub fn divide(depth: u64) -> Self {
        Self {
            divide: true,
            ab_prune: false,
            depth,
        }
    }
}

pub fn alphabeta<T: AlphaBeta>(
    node: &T,
    settings: &SearchSettings,
    depth: u64,
    mut alpha: f32,
    mut beta: f32,
    max: bool,
) -> (u64, f32) {
    if depth == 0 || node.is_terminal() {
        return (1, node.score());
    }

    if depth == 1 && settings.divide {
        return (node.children().count().try_into().unwrap(), 0.0);
    }

    let mut value = if max {
        f32::NEG_INFINITY
    } else {
        f32::INFINITY
    };
    let mut count = 0;
    for ch in node.children() {
        count += if max {
            let (c, chv) = alphabeta(&ch, settings, depth - 1, alpha, beta, false);
            value = f32::max(value, chv);
            alpha = f32::max(alpha, value);
            if value >= beta && settings.ab_prune {
                break;
            }
            c
        } else {
            let (c, chv) = alphabeta(&ch, settings, depth - 1, alpha, beta, true);
            value = f32::min(value, chv);
            beta = f32::min(alpha, value);
            if value <= alpha && settings.ab_prune {
                break;
            }
            c
        };
    }
    if count == 0 {
        return (0, node.score());
    }
    (count, value)
}

impl AlphaBeta for Board {
    fn is_terminal(&self) -> bool {
        false
    }

    fn score(&self) -> f32 {
        1.0
    }

    fn children(&self) -> Self::ItemIterator<'_> {
        self.legal_moves().map(|m| apply(self, m))
    }

    type ItemIterator<'a> = impl Iterator<Item = Board> + 'a;
}

fn apply(b: &Board, m: Move) -> Board {
    b.clone().apply_move(&m).unwrap()
}

impl Board {
    pub fn alphabeta(&self, settings: &SearchSettings, max: bool) -> (u64, f32) {
        alphabeta(
            self,
            settings,
            settings.depth,
            f32::NEG_INFINITY,
            f32::INFINITY,
            max,
        )
    }
}
