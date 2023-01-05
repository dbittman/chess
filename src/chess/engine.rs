use std::{sync::Arc, time::Instant};

use tokio::{
    select, spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
use vampirc_uci::{UciFen, UciMessage, UciMove, UciSearchControl, UciTimeControl};

use super::board::Board;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct EngineResult {
    best_move: Option<UciMove>,
    ponder: Option<UciMove>,
}

#[derive(Debug, Clone, Default)]
enum EngineState {
    Going(ThinkState),
    Pondering(ThinkState),
    #[default]
    Stopped,
}

impl EngineState {
    /// Returns `true` if the engine state is [`Stopped`].
    ///
    /// [`Stopped`]: EngineState::Stopped
    #[must_use]
    fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped)
    }
}

#[derive(Debug, Clone)]
struct ThinkState {
    start_time: Instant,
    time_control: Option<UciTimeControl>,
    search_control: Option<UciSearchControl>,
    best_result: EngineResultState,
}

impl ThinkState {
    fn new(time_control: Option<UciTimeControl>, search_control: Option<UciSearchControl>) -> Self {
        Self {
            start_time: Instant::now(),
            time_control,
            search_control,
            best_result: EngineResultState::Calculating,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum EngineResultState {
    #[default]
    Calculating,
    Ready(EngineResult),
    Communicated(EngineResult),
}

#[derive(Default)]
struct EngineInternals {
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
        *self = Self::default();
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
    async fn calculate(self: &Arc<Self>) -> EngineResult {
        for mv in self.internals.lock().await.board.legal_moves() {
            return EngineResult {
                best_move: Some(mv.into()),
                ponder: None,
            };
            /*let test = self
            .internals
            .lock()
            .await
            .board
            .clone()
            .apply_move(&mv)
            .unwrap();
            */
            //let (_count, _val) = test.alphabeta(&self.settings, true);
        }
        todo!()
    }

    fn send_bestmove(self: &Arc<Self>, mv: EngineResult) {
        self.send_uci_message(UciMessage::BestMove {
            best_move: mv.best_move.unwrap(),
            ponder: mv.ponder,
        });
    }

    async fn handle_message(self: &Arc<Self>, msg: UciMessage) {
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
                self.internals.lock().await.state = EngineState::Stopped;
            }
            UciMessage::Go {
                time_control,
                search_control,
            } => {
                self.internals.lock().await.state =
                    EngineState::Going(ThinkState::new(time_control, search_control));
                // TODO: put something, anything, into the engine result.
            }
            UciMessage::Stop => {
                let mut internal = self.internals.lock().await;
                match &mut internal.state {
                    EngineState::Going(state) => match state.best_result {
                        EngineResultState::Ready(res) => {
                            self.send_bestmove(res);
                        }
                        _ => {}
                    },
                    _ => {}
                }
                internal.state = EngineState::Stopped;
            }
            UciMessage::PonderHit => {
                self.internals.lock().await.state = EngineState::Going(ThinkState::new(None, None));
            }
            UciMessage::UciNewGame => {
                self.internals.lock().await.state = EngineState::Stopped;
            }
            _ => {}
        }
    }

    async fn record_bestmove(self: &Arc<Self>, result: EngineResult) {
        let mut internal = self.internals.lock().await;
        match &mut internal.state {
            EngineState::Going(state) => {
                if let EngineResultState::Communicated(x) = state.best_result {
                    if x == result {
                        return;
                    }
                }
                state.best_result = EngineResultState::Ready(result);
            }
            _ => {}
        }
    }

    async fn should_send_bestmove(&self) -> Option<EngineResult> {
        let mut internal = self.internals.lock().await;
        match &mut internal.state {
            EngineState::Going(state) => match state.best_result {
                EngineResultState::Ready(x) => {
                    state.best_result = EngineResultState::Communicated(x.clone());
                    return Some(x);
                }
                _ => return None,
            },
            _ => None,
        }
    }

    pub async fn main_task_engine(self: &Arc<Self>) {
        loop {
            let state = self.internals.lock().await.state.clone();
            if !state.is_stopped() {
                let calc = self.calculate();
                let mut messages_recv = self.messages_recv.lock().await;
                let msg = messages_recv.recv();
                select! {
                    calc = calc => {
                        self.record_bestmove(calc).await;
                        if let Some(mv) = self.should_send_bestmove().await {
                            self.send_bestmove(mv);
                        }
                    },
                    msg = msg => {
                        self.handle_message(msg.unwrap()).await;
                    }
                }
            } else {
                let mut messages_recv = self.messages_recv.lock().await;
                let msg = messages_recv.recv();
                self.handle_message(msg.await.unwrap()).await;
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
            UciMessage::Position { .. } => {}
            UciMessage::SetOption { .. } => todo!(),
            UciMessage::UciNewGame => {}
            UciMessage::Stop => {}
            UciMessage::PonderHit => {}
            UciMessage::Quit => {
                self.main_task.lock().await.take().unwrap().abort();
            }
            UciMessage::Go { .. } => {}
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
