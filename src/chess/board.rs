use std::fmt::Display;

use colored::Colorize;

use super::{bitboard::BitBoard, File, Piece, Rank, Side, Square, ALL_PIECES, NR_PIECE_TYPES};

#[derive(Debug, Default)]
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
        self.val &= !(1 << 0);
    }

    pub fn remove_queenside(&mut self) {
        self.val &= !(1 << 1);
    }
}

#[derive(Default)]
pub struct Board {
    pieces: [BitBoard; NR_PIECE_TYPES],
    sides: [BitBoard; 2],
    castle_rights: [CastleRights; 2],
}

impl Board {
    #[allow(dead_code)]
    pub fn color_pieces(&self, side: Side) -> BitBoard {
        self.sides[side]
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

    pub fn from_fen(fen: &str) -> Result<Self, ()> {
        let mut rank = Rank::LAST;
        let mut file = File::A;
        let mut board = Self::default();
        for ch in fen.chars() {
            let (piece, skip) = match ch {
                _ if ch.is_ascii_digit() => (None, Some(ch.to_digit(10).unwrap() as usize)),
                'r' => (Some((Side::Black, Piece::Rook)), None),
                'n' => (Some((Side::Black, Piece::Knight)), None),
                'b' => (Some((Side::Black, Piece::Bishop)), None),
                'q' => (Some((Side::Black, Piece::Queen)), None),
                'k' => (Some((Side::Black, Piece::King)), None),
                'p' => (Some((Side::Black, Piece::Pawn)), None),
                'R' => (Some((Side::White, Piece::Rook)), None),
                'N' => (Some((Side::White, Piece::Knight)), None),
                'B' => (Some((Side::White, Piece::Bishop)), None),
                'Q' => (Some((Side::White, Piece::Queen)), None),
                'K' => (Some((Side::White, Piece::King)), None),
                'P' => (Some((Side::White, Piece::Pawn)), None),
                '/' => (None, Some(0)),
                _ => {
                    return Err(());
                }
            };
            let sq = Square::from_rank_and_file(rank, file);
            if let Some((side, piece)) = piece {
                board.set_square(sq, piece, side);
            }
            if let Some(skip) = skip {
                if skip == 0 {
                    file = File::A;
                    rank = rank.prev().ok_or(())?;
                } else {
                    file = file.inc(skip + 1);
                }
            } else {
                file = file.inc(1);
            }
        }
        Ok(board)
    }

    pub fn castle_rights(&self, side: Side) -> &CastleRights {
        &self.castle_rights[side]
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
