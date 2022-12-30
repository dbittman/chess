use super::{
    board::Board,
    direction::{Direction, ALL_DIRS},
    moves::Move,
    piece::Piece,
    side::Side,
    square::{File, Rank, Square},
};

impl Board {
    pub fn legal_moves(&self) -> impl Iterator<Item = Move> + '_ {
        self.moves(self.to_move())
            .filter(|m| self.move_legal(m, self.to_move()))
    }

    pub fn is_pinned_by_us(&self, sq: Square, us: Side) -> bool {
        let their_king_sq = (self.pieces(Piece::King) & self.color_pieces(us.other()))
            .to_square()
            .unwrap_or_else(|| {
                panic!(
                    "no king found on board for {:?}. Board state:\n{}",
                    us.other(),
                    self
                )
            });

        // you can't pin a king.
        if their_king_sq == sq {
            return false;
        }

        let mut without = self.clone();
        without.clear_square(sq);
        without.is_in_check(us.other()) && !self.is_in_check(us.other())
    }

    pub fn is_in_check(&self, side: Side) -> bool {
        let king_sq = (self.pieces(Piece::King) & self.color_pieces(side))
            .to_square()
            .unwrap();
        self.is_attacked(king_sq, side, true)
    }

    fn check_attacking_ray(
        &self,
        start: Square,
        us: Side,
        dir: Direction,
        ignore_pins: bool,
    ) -> bool {
        let mut check = start;
        while let Some(next) = check.next_sq(dir) {
            if let Some((piece, side)) = self.piece(next) {
                if side != us {
                    if dir.is_diag() {
                        if ignore_pins || !self.is_pinned_by_us(next, us) {
                            return piece == Piece::Bishop
                                || piece == Piece::Queen
                                || (next.is_kingmove_away(start) && piece == Piece::King);
                        }
                    } else if ignore_pins || !self.is_pinned_by_us(next, us) {
                        return piece == Piece::Rook
                            || piece == Piece::Queen
                            || (next.is_kingmove_away(start) && piece == Piece::King);
                    }
                }
                return false;
            }
            check = next;
        }
        false
    }

    pub fn is_attacked(&self, sq: Square, us: Side, ignore_pins: bool) -> bool {
        // check attacks from bishops, rooks, queens, and kings.
        for dir in ALL_DIRS {
            if self.check_attacking_ray(sq, us, dir, ignore_pins) {
                return true;
            }
        }

        // check attacks from knights
        for dir in ALL_DIRS {
            if let Some(next) = sq.next_sq_knight(dir) {
                if let Some((piece, side)) = self.piece(next) {
                    if piece == Piece::Knight
                        && side != us
                        && (ignore_pins || !self.is_pinned_by_us(next, us))
                    {
                        return true;
                    }
                }
            }
        }

        //check attacks from pawns
        let pawn_attack_rank = match us {
            Side::White => sq.rank().next(),
            Side::Black => sq.rank().prev(),
        };
        let f1 = sq.file().prev();
        let f2 = sq.file().next();

        if let Some(rank) = pawn_attack_rank {
            if let Some(f1) = f1 {
                let source = Square::from_rank_and_file(rank, f1);
                if let Some((piece, side)) = self.piece(source) {
                    if piece == Piece::Pawn
                        && side != us
                        && (ignore_pins || !self.is_pinned_by_us(source, us))
                    {
                        return true;
                    }
                }
            }
            if let Some(f2) = f2 {
                let source = Square::from_rank_and_file(rank, f2);
                if let Some((piece, side)) = self.piece(source) {
                    if piece == Piece::Pawn
                        && side != us
                        && (ignore_pins || !self.is_pinned_by_us(source, us))
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn move_legal(&self, mv: &Move, side: Side) -> bool {
        match self.piece(mv.start()) {
            Some((_, s)) => {
                if s != side {
                    return false;
                }
            }
            None => return false,
        }
        // check for checks
        let applied = match self.clone().apply_move(mv) {
            Ok(x) => x,
            _ => return false,
        };

        if applied.is_in_check(side) {
            return false;
        }

        // check relevant squares for castling over and from check.
        if mv.is_castling(self) {
            let rank = match side {
                Side::White => Rank::new(1),
                Side::Black => Rank::new(8),
            };
            if mv.is_kingside_castle(self) {
                if !self.castle_rights(side).kingside() {
                    return false;
                }
                if let Some((r, s)) = self.piece(Square::from_rank_and_file(rank, File::H)) {
                    if r != Piece::Rook || s != side {
                        return false;
                    }
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::E), side, true) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::F), side, true) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::G), side, true) {
                    return false;
                }
            } else {
                if !self.castle_rights(side).queenside() {
                    return false;
                }
                if let Some((r, s)) = self.piece(Square::from_rank_and_file(rank, File::A)) {
                    if r != Piece::Rook || s != side {
                        return false;
                    }
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::E), side, true) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::D), side, true) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::C), side, true) {
                    return false;
                }
            }
        }

        true
    }
}
