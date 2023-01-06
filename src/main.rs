#![feature(step_trait)]
#![feature(type_alias_impl_trait)]
#![feature(test)]

use std::{io::stdin, sync::Arc};

use chess::{board::Board, engine::Engine};
use vampirc_uci::UciMessage;

use crate::ab::SearchSettings;

pub mod ab;
pub mod chess;
mod testing;
#[tokio::main]
async fn main() {
    let settings = SearchSettings {
        depth: 1,
        divide: false,
        ab_prune: true,
    };
    let board = Board::from_fen("startpos").unwrap();
    let x = tokio::task::spawn(async {
        let res = { tokio::task::spawn_blocking(move || board.alphabeta(&settings, true)) }
            .await
            .unwrap();
        res
    })
    .await
    .unwrap();
    let engine = Arc::new(Engine::default());
    for line in stdin().lines() {
        engine
            .handle_uci_message(vampirc_uci::parse_one(&line.unwrap()))
            .await;
    }
    std::future::pending::<()>().await;
}
