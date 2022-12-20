#![feature(step_trait)]

use std::{io::stdin, process::Command};

use chess::board::Board;
use vampirc_uci::{parse_one, UciMessage};

use crate::chess::{
    bitboard::BitBoard, engine::Engine, piecemoves, File, Piece, Rank, Side, Square,
};

mod chess;
fn main() {
    //let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    let b = Board::from_fen("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2")
        .unwrap();

    println!("{}", b);
    println!("{}", b.is_in_check(Side::Black));
    for m in b.legal_moves() {
        println!("{}", m);
    }

    let o = BitBoard::from_square(Square::from_rank_and_file(Rank::new(4), File::E));
    let _m = piecemoves::get_piece_moves(
        Piece::Pawn,
        Side::White,
        Square::from_rank_and_file(Rank::new(2), File::E),
        BitBoard::default(),
        o,
        BitBoard::default(),
    );
    //println!("{}", m);

    let cmd = Command::new("")

    //   return;
    #[allow(unreachable_code)]
    let mut engine = Engine::default();
    for line in stdin().lines() {
        let msg: UciMessage = parse_one(&line.unwrap());
        eprintln!("rec: {} :: {:?}", msg, msg);
        engine.handle_uci_message(msg);
        eprintln!("{}", engine.board());
    }
}
