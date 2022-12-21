use super::board::Board;

pub trait AlphaBeta {
    fn is_terminal(&self) -> bool;
    fn score(&self) -> f32;
    fn children(&self) -> impl Iterator<Item = Self> + '_;
}

pub fn alphabeta<T: AlphaBeta>(
    node: &T,
    depth: u64,
    mut alpha: f32,
    mut beta: f32,
    max: bool,
    no_pruning: bool,
) -> (u64, f32) {
    if depth == 0 || node.is_terminal() {
        return (1, node.score());
    }

    let mut value = if max {
        f32::NEG_INFINITY
    } else {
        f32::INFINITY
    };
    let mut count = 0;
    for ch in node.children() {
        count += if max {
            let (c, chv) = alphabeta(&ch, depth - 1, alpha, beta, false, no_pruning);
            value = f32::max(value, chv);
            alpha = f32::max(alpha, value);
            if value >= beta && !no_pruning {
                break;
            }
            c
        } else {
            let (c, chv) = alphabeta(&ch, depth - 1, alpha, beta, true, no_pruning);
            value = f32::min(value, chv);
            beta = f32::min(alpha, value);
            if value <= alpha && !no_pruning {
                break;
            }
            c
        };
    }
    if count == 0 {
        return (1, node.score());
    }
    return (count, value);
}

impl AlphaBeta for Board {
    fn is_terminal(&self) -> bool {
        false
    }

    fn score(&self) -> f32 {
        1.0
    }

    fn children(&self) -> impl Iterator<Item = Self> + '_ {
        self.legal_moves()
            .map(|m| self.clone().apply_move(&m).unwrap())
    }
}
