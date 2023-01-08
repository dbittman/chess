#![feature(step_trait)]
#![feature(type_alias_impl_trait)]
#![feature(test)]
#![feature(let_chains)]

use std::{io::stdin, sync::Arc};

use chess::{board::Board, engine::Engine};
use vampirc_uci::{parse_one, UciMessage, UciTimeControl};

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
    let _x = tokio::task::spawn(async {
        let res = { tokio::task::spawn_blocking(move || board.alphabeta(&settings, true)) }
            .await
            .unwrap();
        res
    })
    .await
    .unwrap();
    let engine = Arc::new(Engine::default());
    for line in stdin().lines() {
        eprintln!("line: {:?}", line);
        let msg = if line.as_ref().unwrap().starts_with("go ponder") {
            UciMessage::Go {
                time_control: Some(UciTimeControl::Ponder),
                search_control: None,
            }
        } else {
            parse_one(&line.unwrap())
        };
        engine.handle_uci_message(msg).await;
    }
    std::future::pending::<()>().await;
}
