#![feature(step_trait)]

use std::io::stdin;

use chess::board::Board;
use vampirc_uci::{parse_one, UciMessage};

use crate::chess::{bitboard::BitBoard, piecemoves, File, Piece, Rank, Side, Square};

mod chess;
fn main() {
    let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();

    println!("{}", b);
    for m in b.moves(Side::White) {
        println!("{:?}", m);
    }

    let o = BitBoard::from_square(Square::from_rank_and_file(Rank::new(4), File::E));
    let m = piecemoves::get_piece_moves(
        Piece::Pawn,
        Side::White,
        Square::from_rank_and_file(Rank::new(2), File::E),
        BitBoard::default(),
        o,
        BitBoard::default(),
    );
    //println!("{}", m);

    return;
    #[allow(unreachable_code)]
    for line in stdin().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());

        eprintln!("rec: {} :: {:?}", msg, msg);
    }
}
