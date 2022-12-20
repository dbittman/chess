use std::borrow::Borrow;

use super::{bitboard::BitBoard, board::Board, piecemoves, File, Piece, Rank, Side, Square};

pub struct MoveIter<'a> {
    board: &'a Board,
    mask: BitBoard,
}

impl<'a> MoveIter<'a> {
    pub fn mask(&mut self, mask: BitBoard) {
        self.mask = mask;
    }
}

impl<'a> Iterator for MoveIter<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Board {
    fn check_castle_has_room(&self, side: Side, kingside: bool) -> bool {
        let rank = match side {
            Side::White => Rank::new(1),
            Side::Black => Rank::new(7),
        };
        if kingside {
            if self
                .piece(Square::from_rank_and_file(rank, File::F))
                .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::G))
                    .is_some()
            {
                false
            } else {
                true
            }
        } else {
            if self
                .piece(Square::from_rank_and_file(rank, File::B))
                .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::C))
                    .is_some()
                || self
                    .piece(Square::from_rank_and_file(rank, File::D))
                    .is_some()
            {
                false
            } else {
                true
            }
        }
    }

    pub fn castle_moves(&self, side: Side) -> Vec<Move> {
        let king_sq = self.pieces(Piece::King).to_square();
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
                BitBoard::default(),
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
}
