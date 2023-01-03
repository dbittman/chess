use std::sync::Arc;

use tokio::{
    spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
use vampirc_uci::{UciFen, UciMessage, UciMove};

use super::board::Board;

struct EngineResult {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EngineState {
    Going,
    Pondering,
    #[default]
    Stopped,
}

#[derive(Default)]
struct EngineInternals {
    result: Option<EngineResult>,
    state: EngineState,
    board: Board,
    is_init: bool,
}

pub struct Engine {
    internals: Mutex<EngineInternals>,
    main_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    messages_recv: Mutex<UnboundedReceiver<UciMessage>>,
    messages_send: UnboundedSender<UciMessage>,
}

impl Default for Engine {
    fn default() -> Self {
        let (send, recv) = unbounded_channel();
        Self {
            internals: Default::default(),
            messages_recv: Mutex::new(recv),
            messages_send: send,
            main_task: Default::default(),
        }
    }
}

impl EngineInternals {
    pub fn reset(&mut self) {
        self.is_init = true;
    }

    pub fn set_position(&mut self, startpos: bool, fen: Option<UciFen>, moves: &[UciMove]) {
        let start_fen =
            vampirc_uci::UciFen::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let fen = if startpos {
            start_fen
        } else {
            fen.unwrap_or(start_fen)
        };
        self.board = Board::from_fen(fen.as_str()).unwrap();
        for mv in moves {
            self.board = self.board.clone().apply_move(&(*mv).into()).unwrap();
        }
    }
}

impl Engine {
    pub async fn main_task_engine(self: &Arc<Self>) {
        loop {
            let msg = self.messages_recv.lock().await.recv().await.unwrap();
            match msg {
                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    self.internals
                        .lock()
                        .await
                        .set_position(startpos, fen, &moves);
                }
                UciMessage::Go { .. } => {}
                UciMessage::Stop => {}
                UciMessage::PonderHit => {}
                //UciMessage::UciNewGame => {}
                _ => {}
            }
        }
    }

    pub async fn init_uci(self: &Arc<Self>) {
        self.send_uci_message(UciMessage::Id {
            name: Some("Rust Chess".into()),
            author: None,
        });
        self.send_uci_message(UciMessage::Id {
            name: None,
            author: Some("Daniel Bittman".into()),
        });
        *self.main_task.lock().await = None;
        {
            self.internals.lock().await.reset();
        }
        let self2 = self.clone();
        self.main_task
            .lock()
            .await
            .replace(spawn(async move { self2.main_task_engine().await }));

        self.send_uci_message(UciMessage::UciOk);
    }

    pub fn send_uci_message(&self, uci: UciMessage) {
        println!("{}", uci.to_string());
    }

    pub async fn is_init(&self) -> bool {
        self.internals.lock().await.is_init
    }

    pub async fn handle_uci_message(self: &Arc<Self>, uci: UciMessage) {
        //eprintln!("uci message: {}", uci.to_string());
        if !self.is_init().await {
            if uci != UciMessage::Uci {
                eprintln!("UCI message while not in UCI mode {}", uci.to_string());
                return;
            }
        }
        match &uci {
            UciMessage::Uci => {
                self.init_uci().await;
            }
            //UciMessage::Debug(_) => todo!(),
            UciMessage::IsReady => {
                self.send_uci_message(UciMessage::ReadyOk);
            }
            //UciMessage::Register { later, name, code } => todo!(),
            UciMessage::Position {
                startpos,
                fen,
                moves,
            } => {}
            UciMessage::SetOption { name, value } => todo!(),
            UciMessage::UciNewGame => {}
            UciMessage::Stop => {}
            UciMessage::PonderHit => {}
            UciMessage::Quit => {
                self.main_task.lock().await.take().unwrap().abort();
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {}
            //UciMessage::Id { name, author } => todo!(),
            //UciMessage::UciOk => todo!(),
            //UciMessage::ReadyOk => todo!(),
            //UciMessage::BestMove { best_move, ponder } => todo!(),
            //UciMessage::CopyProtection(_) => todo!(),
            //UciMessage::Registration(_) => todo!(),
            //UciMessage::Option(_) => todo!(),
            //UciMessage::Info(_) => todo!(),
            //UciMessage::Unknown(_, _) => todo!(),
            _ => {
                eprintln!("unknown UCI message {}", uci.to_string());
                return;
            }
        }
        self.messages_send.send(uci).unwrap();
    }
}
