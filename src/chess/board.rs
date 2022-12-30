use std::fmt::Display;

use colored::Colorize;
use fen::{BoardState, FenError};

use super::{
    bitboard::BitBoard, moves::Move, Direction, File, Piece, Rank, Side, Square, ALL_DIRS,
    ALL_PIECES, NR_PIECE_TYPES,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct CastleRights {
    val: u8,
}

impl Display for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if self.kingside() { "k" } else { "" },
            if self.queenside() { "q" } else { "" }
        )
    }
}

impl CastleRights {
    pub fn queenside(&self) -> bool {
        self.val & (1 << 1) == 0
    }

    pub fn kingside(&self) -> bool {
        self.val & (1 << 0) == 0
    }

    pub fn remove_kingside(&mut self) {
        self.val |= 1 << 0;
    }

    pub fn remove_queenside(&mut self) {
        self.val |= 1 << 1;
    }

    fn from_boardstate(value: &BoardState) -> [Self; 2] {
        let mut wcr = CastleRights::default();
        let mut bcr = CastleRights::default();
        if !value.black_can_oo {
            bcr.remove_kingside();
        }
        if !value.black_can_ooo {
            bcr.remove_queenside();
        }
        if !value.white_can_oo {
            wcr.remove_kingside();
        }
        if !value.white_can_ooo {
            wcr.remove_queenside();
        }
        [wcr, bcr]
    }
}

#[allow(dead_code)]
#[derive(Default, Clone)]
pub struct Board {
    pieces: [BitBoard; NR_PIECE_TYPES],
    sides: [BitBoard; 2],
    castle_rights: [CastleRights; 2],
    to_move: Side,
    enpassant: BitBoard,
    halfmove_clock: u64,
    fullmoves: u64,
}

impl Board {
    #[allow(dead_code)]
    pub fn color_pieces(&self, side: Side) -> BitBoard {
        self.sides[side]
    }

    pub fn move_structural(&self, mv: &Move) -> bool {
        self.piece(mv.start()).is_some()
    }

    fn adv_ply(&mut self) {
        //TODO
        self.to_move = self.to_move.other();
    }

    unsafe fn apply_move_unchecked(mut self, mv: &Move) -> Self {
        // TODO: remove castle rights if rook is captured?
        let (piece, side) = self.piece(mv.start()).unwrap();

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
        self.adv_ply();
        self.enpassant = BitBoard::default();

        if piece == Piece::Pawn
            && mv.start().rank().allow_double_move(side)
            && (mv.dest().rank() == Rank::new(5) || mv.dest().rank() == Rank::new(4))
        {
            let enpassant_rank = match side {
                Side::White => Rank::new(3),
                Side::Black => Rank::new(6),
            };
            let enpassant_sq = Square::from_rank_and_file(enpassant_rank, mv.start().file());
            self.enpassant = BitBoard::from_square(enpassant_sq);
        }

        // TODO: remove for release
        #[cfg(debug_assertions)]
        self.assert_is_sane();
        self
    }

    pub fn apply_move(self, mv: &Move) -> Result<Self, ()> {
        if self.move_structural(&mv) {
            Ok(unsafe { self.apply_move_unchecked(mv) })
        } else {
            Err(())
        }
    }

    pub fn is_pinned_by_us(&self, sq: Square, us: Side) -> bool {
        let their_king_sq = (self.pieces(Piece::King) & self.color_pieces(us.other()))
            .to_square()
            .expect(&format!(
                "no king found on board for {:?}. Board state:\n{}",
                us.other(),
                self
            ));

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
        let a = self.is_attacked(king_sq, side, true);
        /* *
        println!(
            "ks: {}{} {:?} => {}",
            king_sq.file(),
            king_sq.rank(),
            side,
            a
        );
        */
        a
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
            //println!("check sq {}", next);
            if let Some((piece, side)) = self.piece(next) {
                if side != us {
                    if dir.is_diag() {
                        if ignore_pins || !self.is_pinned_by_us(next, us) {
                            return piece == Piece::Bishop
                                || piece == Piece::Queen
                                || (next.is_kingmove_away(start) && piece == Piece::King);
                        }
                    } else {
                        if ignore_pins || !self.is_pinned_by_us(next, us) {
                            return piece == Piece::Rook
                                || piece == Piece::Queen
                                || (next.is_kingmove_away(start) && piece == Piece::King);
                        }
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
            // println!("checking {:?}", dir);
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
        /*
        println!(
            "Applied {}: \n{} {} {:?}",
            mv,
            applied,
            applied.is_in_check(side),
            side
        );
        */
        if applied.is_in_check(side) {
            return false;
        }

        //eprintln!("castle check ==> {} {}", mv, mv.is_castling(self));
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

    #[allow(dead_code)]
    pub fn pieces(&self, piece: Piece) -> BitBoard {
        self.pieces[piece]
    }

    pub fn check_piece(&self, sq: Square) -> Option<Piece> {
        for p in ALL_PIECES {
            if self.pieces[p].get(sq) {
                return Some(p);
            }
        }
        None
    }

    pub fn piece(&self, sq: Square) -> Option<(Piece, Side)> {
        if self.sides[Side::White].get(sq) {
            Some((self.check_piece(sq).unwrap(), Side::White))
        } else if self.sides[Side::Black].get(sq) {
            Some((self.check_piece(sq).unwrap(), Side::Black))
        } else {
            None
        }
    }

    pub fn clear_square(&mut self, sq: Square) {
        for i in 0..NR_PIECE_TYPES {
            self.pieces[i].set(sq, false);
        }

        for i in 0..2 {
            self.sides[i].set(sq, false);
        }
    }

    pub fn set_square(&mut self, sq: Square, piece: Piece, side: Side) {
        self.clear_square(sq);
        self.pieces[piece].set(sq, true);
        self.sides[side].set(sq, true);
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let bs = fen::BoardState::from_fen(fen)?;
        let b = Board::from(bs);
        return Ok(b);
    }

    pub fn castle_rights(&self, side: Side) -> &CastleRights {
        &self.castle_rights[side]
    }

    pub fn castle_rights_mut(&mut self, side: Side) -> &mut CastleRights {
        &mut self.castle_rights[side]
    }

    pub fn legal_moves(&self) -> impl Iterator<Item = Move> + '_ {
        self.moves(self.to_move)
            .filter(|m| self.move_legal(m, self.to_move))
    }

    pub fn enpassant(&self) -> &BitBoard {
        &self.enpassant
    }

    fn assert_piece_has_color(&self, piece: Piece) {
        for p in self.pieces(piece).into_iter() {
            assert!(self.sides[Side::White].get(p) || self.sides[Side::Black].get(p));
        }
    }

    pub fn assert_is_sane(&self) {
        for sq in self.sides[Side::White].into_iter() {
            self.piece(sq).unwrap();
        }

        for sq in self.sides[Side::Black].into_iter() {
            self.piece(sq).unwrap();
        }

        self.assert_piece_has_color(Piece::Bishop);
        self.assert_piece_has_color(Piece::Rook);
        self.assert_piece_has_color(Piece::Queen);
        self.assert_piece_has_color(Piece::King);
        self.assert_piece_has_color(Piece::Pawn);
        self.assert_piece_has_color(Piece::Knight);
    }
}

impl From<BoardState> for Board {
    fn from(value: BoardState) -> Self {
        let mut b = Self {
            castle_rights: CastleRights::from_boardstate(&value),
            to_move: (&value.side_to_play).into(),
            enpassant: value.en_passant_square.map_or(BitBoard::default(), |s| {
                BitBoard::from_square(unsafe { Square::new(s) })
            }),
            halfmove_clock: value.halfmove_clock,
            fullmoves: value.fullmove_number,
            ..Default::default()
        };
        for (num, piece) in value.pieces.iter().enumerate() {
            if let Some(piece) = piece {
                b.set_square(
                    unsafe { Square::new(num as u8) },
                    (&piece.kind).into(),
                    (&piece.color).into(),
                )
            }
        }
        b
    }
}

#[allow(dead_code)]
fn fen_char(piece: Piece, side: Side) -> char {
    let c = match piece {
        Piece::Pawn => "Pp",
        Piece::Bishop => "Bb",
        Piece::Knight => "Nn",
        Piece::Rook => "Rr",
        Piece::Queen => "Qq",
        Piece::King => "Kk",
    };
    c.chars().nth(side.into()).unwrap()
}

fn utf8_char(piece: Piece, side: Side) -> char {
    let c = match piece {
        Piece::Pawn => "♙♟︎",
        Piece::Bishop => "♗♝",
        Piece::Knight => "♘♞",
        Piece::Rook => "♖♜",
        Piece::Queen => "♕♛",
        Piece::King => "♔♚",
    };
    c.chars().nth(side.into()).unwrap()
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "kr: white={}, black={}{}",
            self.castle_rights(Side::White),
            self.castle_rights(Side::Black),
            self.enpassant
                .to_square()
                .map_or("".to_owned(), |x| format!(" ; enpassant: {}", x))
        )?;
        for rank in (Rank::FIRST..=Rank::LAST).rev() {
            for file in File::A..=File::H {
                let sq = Square::from_rank_and_file(rank, file);
                if file == File::A && rank != Rank::LAST {
                    writeln!(f)?;
                }
                if file == File::A {
                    write!(f, "{} ", rank.0)?;
                }
                let p = self.piece(sq);
                if let Some((piece, side)) = p {
                    let s = format!(" {}  ", utf8_char(piece, side)).black();
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
                    let s = format!("    ");
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
            write!(f, "{}   ", file)?;
        }
        writeln!(f)
    }
}
