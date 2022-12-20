use super::{bitboard::BitBoard, Direction, Piece, Side, Square, ALL_DIRS};

fn build_diagonal_moves(sq: Square, attackable: BitBoard, ourside: BitBoard, bb: &mut BitBoard) {
    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::UpRight) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::DownRight) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::DownLeft) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::UpLeft) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }
}

fn build_lateral_moves(sq: Square, attackable: BitBoard, ourside: BitBoard, bb: &mut BitBoard) {
    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::Up) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::Down) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::Left) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }

    let mut cur = sq;
    while let Some(next) = cur.next_sq(Direction::Right) {
        if ourside.get(next) {
            break;
        }
        bb.set(next, true);
        cur = next;
        if attackable.get(next) {
            break;
        }
    }
}

fn build_king_moves(sq: Square, bb: &mut BitBoard) {
    let cur = sq;
    if let Some(next) = cur.next_sq(Direction::Up) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::Down) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::Left) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::Right) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::UpRight) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::DownRight) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::UpLeft) {
        bb.set(next, true);
    }

    if let Some(next) = cur.next_sq(Direction::DownLeft) {
        bb.set(next, true);
    }
}

fn build_knight_moves(sq: Square, bb: &mut BitBoard) {
    for dir in ALL_DIRS {
        if let Some(next) = sq.next_sq_knight(dir) {
            bb.set(next, true);
        }
    }
}

fn build_pawn_moves(
    sq: Square,
    side: Side,
    attackable: BitBoard,
    enpassant: BitBoard,
    bb: &mut BitBoard,
) {
    let rank = sq.rank();
    let dir = match side {
        Side::White => Direction::Up,
        Side::Black => Direction::Down,
    };
    if rank.allow_double_move(side) {
        let next = sq.next_sq(dir).unwrap().next_sq(dir).unwrap();
        bb.set(next, true);
    }

    if let Some(next) = sq.next_sq(dir) {
        bb.set(next, true);
    }

    *bb &= !attackable;

    let ad1 = match side {
        Side::White => Direction::UpRight,
        Side::Black => Direction::DownRight,
    };
    let ad2 = match side {
        Side::White => Direction::UpLeft,
        Side::Black => Direction::DownLeft,
    };
    let as1 = sq.next_sq(ad1);
    let as2 = sq.next_sq(ad2);

    if let Some(as1) = as1 {
        if attackable.get(as1) || enpassant.get(as1) {
            bb.set(as1, true);
        }
    }
    if let Some(as2) = as2 {
        if attackable.get(as2) || enpassant.get(as2) {
            bb.set(as2, true);
        }
    }
}

pub fn get_piece_moves(
    piece: Piece,
    side: Side,
    sq: Square,
    enpassant: BitBoard,
    attackable: BitBoard,
    ourside: BitBoard,
) -> BitBoard {
    let mut bb = BitBoard::default();
    match piece {
        Piece::Pawn => build_pawn_moves(sq, side, attackable, enpassant, &mut bb),
        Piece::Bishop => build_diagonal_moves(sq, attackable, ourside, &mut bb),
        Piece::Knight => build_knight_moves(sq, &mut bb),
        Piece::Rook => build_lateral_moves(sq, attackable, ourside, &mut bb),
        Piece::Queen => {
            build_diagonal_moves(sq, attackable, ourside, &mut bb);
            build_lateral_moves(sq, attackable, ourside, &mut bb);
        }
        Piece::King => build_king_moves(sq, &mut bb),
    };
    bb & !ourside
}
