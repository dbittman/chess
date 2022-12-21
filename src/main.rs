#![feature(step_trait)]
#![feature(return_position_impl_trait_in_trait)]

use std::{io::stdin, process::Command};

use chess::board::Board;
use vampirc_uci::{parse_one, UciMessage};

use crate::chess::{
    ab::alphabeta, bitboard::BitBoard, engine::Engine, piecemoves, File, Piece, Rank, Side, Square,
};

mod chess;
fn main() {
    //let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    let b = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();

    println!("{}", b);
    let mut x = 0;
    for m in b.legal_moves() {
        let test = b.clone().apply_move(&m).unwrap();
        let (count, val) = alphabeta(&test, 2, f32::NEG_INFINITY, f32::INFINITY, true, true);
        println!("{} {} {}", m, count, val);
        x += count;
    }
    println!("total count: {}", x);

    let o = BitBoard::from_square(Square::from_rank_and_file(Rank::new(4), File::E));
    let _m = piecemoves::get_piece_moves(
        Piece::Pawn,
        Side::White,
        Square::from_rank_and_file(Rank::new(2), File::E),
        BitBoard::default(),
        o,
        BitBoard::default(),
    );
    return;
    //println!("{}", m);

    //let cmd = Command::new("")

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
