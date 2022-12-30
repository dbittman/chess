#![feature(step_trait)]
#![feature(type_alias_impl_trait)]
#![feature(test)]

use chess::board::Board;

use crate::chess::ab::SearchSettings;

pub mod chess;
mod testing;
fn main() {
    //let b = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();
    let b =
        Board::from_fen("r3k2r/p1p1qpb1/bn1ppnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/R4K1R b kq - 1 1")
            .unwrap();

    let settings = SearchSettings {
        divide: true,
        ab_prune: false,
        depth: 3,
    };
    for m in b.legal_moves() {
        let test = b.clone().apply_move(&m).unwrap();
        let (_count, _val) = test.alphabeta(&settings, true);
    }
}
