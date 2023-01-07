pub trait AlphaBeta {
    type ItemIterator<'a>: Iterator<Item = (Self, Self::Data)> + 'a
    where
        Self: Sized + 'a;

    type Data: Clone;

    fn is_terminal(&self) -> bool;
    fn score(&self) -> f32;
    fn children(&self) -> Self::ItemIterator<'_>
    where
        Self: Sized;
}

pub struct AlphaBetaResult<D> {
    pub count: u64,
    pub value: f32,
    pub data: Vec<D>,
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
) -> AlphaBetaResult<T::Data> {
    if depth == 0 || node.is_terminal() {
        return AlphaBetaResult {
            count: 1,
            value: node.score(),
            data: vec![],
        };
    }

    if depth == 1 && settings.divide {
        return AlphaBetaResult {
            count: node.children().count().try_into().unwrap(),
            value: 0.0,
            data: vec![],
        };
    }

    let mut value = if max {
        f32::NEG_INFINITY
    } else {
        f32::INFINITY
    };
    let mut count = 0;
    let mut best = vec![];

    for (ch, data) in node.children() {
        count += if max {
            let res = alphabeta(&ch, settings, depth - 1, alpha, beta, false);
            if res.value > value {
                value = res.value;
                best = res.data.clone();
                best.push(data);
            }
            alpha = f32::max(alpha, value);
            if value >= beta && settings.ab_prune {
                break;
            }
            res.count
        } else {
            let res = alphabeta(&ch, settings, depth - 1, alpha, beta, true);
            if res.value < value {
                value = res.value;
                best = res.data.clone();
                best.push(data);
            }
            beta = f32::min(alpha, value);
            if value <= alpha && settings.ab_prune {
                break;
            }
            res.count
        };
    }
    if count == 0 {
        return AlphaBetaResult {
            count: 0,
            value: node.score(),
            data: vec![],
        };
    }

    AlphaBetaResult {
        count: count,
        value: value,
        data: best,
    }
}
