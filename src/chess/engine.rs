use vampirc_uci::UciMessage;

use super::{board::Board, Side};

#[derive(Default)]
pub struct Engine {
    board: Board,
    engine_control: Side,
}

impl Engine {
    pub fn handle_uci_message(&mut self, uci: UciMessage) {
        match uci {
            UciMessage::Uci => {}
            UciMessage::Debug(_) => todo!(),
            UciMessage::IsReady => todo!(),
            UciMessage::Register { later, name, code } => todo!(),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {
                let fen = fen.unwrap_or(vampirc_uci::UciFen::from(
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                ));
                self.board = Board::from_fen(fen.as_str()).unwrap();
                for mv in moves {
                    self.board = self.board.clone().apply_move(&mv.into()).unwrap();
                }
            }
            UciMessage::SetOption { name, value } => todo!(),
            UciMessage::UciNewGame => {}
            UciMessage::Stop => todo!(),
            UciMessage::PonderHit => todo!(),
            UciMessage::Quit => todo!(),
            UciMessage::Go {
                time_control,
                search_control,
            } => todo!(),
            UciMessage::Id { name, author } => todo!(),
            UciMessage::UciOk => todo!(),
            UciMessage::ReadyOk => todo!(),
            UciMessage::BestMove { best_move, ponder } => todo!(),
            UciMessage::CopyProtection(_) => todo!(),
            UciMessage::Registration(_) => todo!(),
            UciMessage::Option(_) => todo!(),
            UciMessage::Info(_) => todo!(),
            UciMessage::Unknown(_, _) => todo!(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}
