#![feature(step_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(test)]

use std::{io::stdin, process::Command};

use chess::board::Board;
use vampirc_uci::{parse_one, UciMessage};

use crate::chess::{
    ab::{alphabeta, SearchSettings},
    bitboard::BitBoard,
    engine::Engine,
    piecemoves, File, Piece, Rank, Side, Square,
};

pub mod chess;
mod testing;
fn main() {
    //let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    let b =
        Board::from_fen("r3k2r/p1p1qpb1/bn1ppnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R4K1R b kq - 1 1")
            .unwrap();

    println!("{}", b);
    let mut x = 0;
    let settings = SearchSettings {
        divide: true,
        ab_prune: false,
        depth: 1,
    };
    for m in b.legal_moves() {
        let test = b.clone().apply_move(&m).unwrap();
        let (count, val) = test.alphabeta(&settings, true);
        println!("{} {} {}", m, count, val);
        //println!("{}", test);
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

    //f2f3, f2f4, g2g4
    // f2f3: e7e6, e7e5
    //  e7e6: g2g4,
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

// e5f7
// c7c6
