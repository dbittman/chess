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
        let (piece, side) = self.piece(mv.start()).unwrap();
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
        // TODO: remove castling rights
        self.clear_square(mv.start());
        self.set_square(mv.dest(), piece, side);
        self.adv_ply();
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
        let mut without = self.clone();
        without.clear_square(sq);
        without.is_in_check(us.other()) && !self.is_in_check(us.other())
    }

    pub fn is_in_check(&self, side: Side) -> bool {
        let king_sq = (self.pieces(Piece::King) & self.color_pieces(side)).to_square();
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
        for dir in ALL_DIRS {
            // println!("checking {:?}", dir);
            if self.check_attacking_ray(sq, us, dir, ignore_pins) {
                return true;
            }
        }

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

        // check relevant squares for castling over and from check.
        if mv.is_castling(self) {
            let rank = match side {
                Side::White => Rank::new(1),
                Side::Black => Rank::new(8),
            };
            if mv.is_kingside_castle(self) {
                if self.is_attacked(Square::from_rank_and_file(rank, File::E), side, false) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::F), side, false) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::G), side, false) {
                    return false;
                }
            } else {
                if self.is_attacked(Square::from_rank_and_file(rank, File::E), side, false) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::D), side, false) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::C), side, false) {
                    return false;
                }
                if self.is_attacked(Square::from_rank_and_file(rank, File::B), side, false) {
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

    pub fn legal_moves(&self) -> impl Iterator<Item = Move> + '_ {
        self.moves(self.to_move)
            .filter(|m| self.move_legal(m, self.to_move))
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
