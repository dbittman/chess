use std::fmt::Display;

use vampirc_uci::UciMove;

use super::{
    bitboard::BitBoard,
    board::Board,
    piece::Piece,
    piecemoves,
    side::Side,
    square::{File, Rank, Square},
};

impl Board {
    pub fn move_structural(&self, mv: &Move) -> bool {
        self.piece(mv.start()).is_some()
    }

    fn check_castle_has_room(&self, side: Side, kingside: bool) -> bool {
        let rank = match side {
            Side::White => Rank::new(1),
            Side::Black => Rank::new(8),
        };
        if kingside {
            !(self
                .piece(Square::from_rank_and_file(rank, File::F))
                .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::G))
                    .is_some())
        } else {
            !(self
                .piece(Square::from_rank_and_file(rank, File::B))
                .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::C))
                    .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::D))
                    .is_some())
        }
    }

    pub fn castle_moves(&self, side: Side) -> Vec<Move> {
        let king_sq = (self.pieces(Piece::King) & self.color_pieces(side))
            .to_square()
            .unwrap();
        let mut v = vec![];
        let rank = king_sq.rank();
        if self.castle_rights(side).kingside() && self.check_castle_has_room(side, true) {
            v.push(Move::new(
                king_sq,
                Square::from_rank_and_file(rank, File::G),
                None,
            ));
        }
        if self.castle_rights(side).queenside() && self.check_castle_has_room(side, false) {
            v.push(Move::new(
                king_sq,
                Square::from_rank_and_file(rank, File::C),
                None,
            ));
        }
        v
    }

    pub fn moves(&self, side: Side) -> impl Iterator<Item = Move> + '_ {
        self.color_pieces(side)
            .into_iter()
            .flat_map(|x| self.moves_from_square(x).unwrap())
            .chain(self.castle_moves(side).into_iter())
    }

    fn moves_from_square(&self, sq: Square) -> Option<impl Iterator<Item = Move>> {
        self.piece(sq).map(move |(piece, side)| {
            let moves = piecemoves::get_piece_moves(
                piece,
                side,
                sq,
                *self.enpassant(),
                self.color_pieces(side.other()),
                self.color_pieces(side),
            );
            // TODO: allocation from vec is slow, maybe
            moves.into_iter().flat_map(move |dest| {
                if piece == Piece::Pawn && dest.rank().is_promo_rank(side) {
                    vec![
                        Move::new(sq, dest, Some(Piece::Queen)),
                        Move::new(sq, dest, Some(Piece::Knight)),
                        Move::new(sq, dest, Some(Piece::Bishop)),
                        Move::new(sq, dest, Some(Piece::Rook)),
                    ]
                    .into_iter()
                } else {
                    vec![Move::new(sq, dest, None)].into_iter()
                }
            })
        })
    }

    unsafe fn apply_move_unchecked(mut self, mv: &Move) -> Self {
        // TODO: remove castle rights if rook is captured?
        let (piece, side) = self.piece(mv.start()).unwrap();
        let is_capture = self.piece(mv.dest()).is_some();

        let qr = match side {
            Side::White => Square::from_rank_and_file(Rank::new(1), File::A),
            Side::Black => Square::from_rank_and_file(Rank::new(8), File::A),
        };
        let kr = match side {
            Side::White => Square::from_rank_and_file(Rank::new(1), File::H),
            Side::Black => Square::from_rank_and_file(Rank::new(8), File::H),
        };
        if piece == Piece::Rook && mv.start() == qr {
            self.castle_rights_mut(side).remove_queenside();
        }
        if piece == Piece::Rook && mv.start() == kr {
            self.castle_rights_mut(side).remove_kingside();
        }
        if piece == Piece::King {
            self.castle_rights_mut(side).remove_kingside();
            self.castle_rights_mut(side).remove_queenside();
        }

        if mv.is_castling(&self) {
            let rank = mv.start().rank();
            if mv.is_kingside_castle(&self) {
                self.set_square(Square::from_rank_and_file(rank, File::F), Piece::Rook, side);
                self.clear_square(Square::from_rank_and_file(rank, File::H));
            } else {
                self.set_square(Square::from_rank_and_file(rank, File::D), Piece::Rook, side);
                self.clear_square(Square::from_rank_and_file(rank, File::A));
            }
        }

        if let Some(enpassant_sq) = self.enpassant().to_square() {
            if enpassant_sq == mv.dest() && piece == Piece::Pawn {
                let kill_rank = match side {
                    Side::White => Rank::new(5),
                    Side::Black => Rank::new(4),
                };

                let enpassant_target_sq = Square::from_rank_and_file(kill_rank, mv.dest().file());
                self.clear_square(enpassant_target_sq);
            }
        }

        self.clear_square(mv.start());
        if let Some(promo) = mv.promo() {
            self.set_square(mv.dest(), promo, side);
        } else {
            self.set_square(mv.dest(), piece, side);
        }
        let half = piece == Piece::Pawn || is_capture;
        self.adv_ply(half);
        self.set_enpassant(BitBoard::default());

        if piece == Piece::Pawn
            && mv.start().rank().allow_double_move(side)
            && (mv.dest().rank() == Rank::new(5) || mv.dest().rank() == Rank::new(4))
        {
            let enpassant_rank = match side {
                Side::White => Rank::new(3),
                Side::Black => Rank::new(6),
            };
            let enpassant_sq = Square::from_rank_and_file(enpassant_rank, mv.start().file());
            self.set_enpassant(BitBoard::from_square(enpassant_sq));
        }

        // TODO: remove for release
        #[cfg(debug_assertions)]
        self.assert_is_sane();
        self
    }

    pub fn apply_move(self, mv: &Move) -> Result<Self, ()> {
        if self.move_structural(mv) {
            Ok(unsafe { self.apply_move_unchecked(mv) })
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    start: Square,
    dest: Square,
    promo: Option<Piece>,
}

impl Move {
    pub fn new(start: Square, dest: Square, promo: Option<Piece>) -> Self {
        Self { start, dest, promo }
    }

    pub fn start(&self) -> Square {
        self.start
    }

    pub fn dest(&self) -> Square {
        self.dest
    }

    pub fn promo(&self) -> Option<Piece> {
        self.promo
    }

    pub fn is_castling(&self, board: &Board) -> bool {
        match board.piece(self.start) {
            Some((piece, _)) => {
                piece == Piece::King
                    && self.start().file() == File::E
                    && (self.dest().file() == File::G || self.dest().file() == File::C)
            }
            None => false,
        }
    }

    pub fn is_kingside_castle(&self, board: &Board) -> bool {
        self.is_castling(board) && self.dest().file() == File::G
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} => {}{} ({:?})",
            self.start().file(),
            self.start().rank(),
            self.dest().file(),
            self.dest().rank(),
            self.promo()
        )
    }
}

impl From<UciMove> for Move {
    fn from(value: UciMove) -> Self {
        Self {
            start: Square::from_rank_and_file(
                value.from.rank.try_into().unwrap(),
                value.from.file.try_into().unwrap(),
            ),
            dest: Square::from_rank_and_file(
                value.to.rank.try_into().unwrap(),
                value.to.file.try_into().unwrap(),
            ),
            promo: value.promotion.map(|p| p.into()),
        }
    }
}
