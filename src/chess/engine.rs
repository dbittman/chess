use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{
    select, spawn,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};
use vampirc_uci::{UciFen, UciMessage, UciMove, UciSearchControl, UciTimeControl};

use crate::ab::SearchSettings;

use super::{board::Board, side::Side};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct EngineResult {
    best_move: Option<UciMove>,
    ponder: Option<UciMove>,
    stats: Stats,
    out_of_time: bool,
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

    /// Returns `true` if the engine state is [`Pondering`].
    ///
    /// [`Pondering`]: EngineState::Pondering
    #[must_use]
    fn is_pondering(&self) -> bool {
        matches!(self, Self::Pondering(..))
    }
}

#[derive(Debug, Clone)]
struct ThinkState {
    start_time: Instant,
    time_control: Option<UciTimeControl>,
    // TODO
    _search_control: Option<UciSearchControl>,
    best_result: EngineResultState,
    our_side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Stats {
    confidence: f32,
    depth: u64,
}

impl ThinkState {
    fn new(
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
        our_side: Side,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            time_control,
            _search_control: search_control,
            best_result: EngineResultState::Calculating,
            our_side,
        }
    }

    fn adj_controls_for_ponder(&mut self) {
        let time_since = self.start_time.elapsed();
        if let Some(UciTimeControl::TimeLeft {
            white_time,
            black_time,
            ..
        }) = &mut self.time_control
        {
            let time = match self.our_side {
                Side::White => white_time,
                Side::Black => black_time,
            };
            if let Some(time) = time {
                *time = time
                    .checked_sub(&vampirc_uci::Duration::from_std(time_since).unwrap())
                    .unwrap_or(vampirc_uci::Duration::milliseconds(100));
            }
        }
        self.start_time = Instant::now();
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
            self.board = self.board.clone().apply_move(&mv.into()).unwrap();
        }
    }

    fn get_move_immediately(&self) -> EngineResult {
        for mv in self.board.legal_moves() {
            return EngineResult {
                best_move: Some(mv.into()),
                out_of_time: true,
                ..Default::default()
            };
        }
        EngineResult {
            best_move: None,
            ponder: None,
            stats: Stats {
                confidence: f32::INFINITY,
                depth: 0,
            },
            out_of_time: true,
        }
    }
}

struct EngineTimes {
    min: Duration,
    max: Duration,
}

impl EngineTimes {
    fn inf() -> Self {
        Self {
            min: Duration::from_secs(0),
            max: Duration::from_secs(100000000),
        }
    }
}

fn calc_time_left(
    white_time: Option<Duration>,
    black_time: Option<Duration>,
    white_increment: Option<Duration>,
    black_increment: Option<Duration>,
    _moves_to_go: Option<u8>,
    our_side: Side,
) -> EngineTimes {
    eprintln!(
        "calc_time_left({:?}, {:?}, {:?}, {:?}, {:?}, {:?})",
        white_time, black_time, white_increment, black_increment, _moves_to_go, our_side
    );
    let (time, _inc) = match our_side {
        Side::White => (white_time, white_increment),
        Side::Black => (black_time, black_increment),
    };
    if let Some(time) = time {
        return EngineTimes {
            min: time / 20,
            max: time / 10,
        };
    }
    // TODO: take all the inputs into account
    //todo!()
    EngineTimes::inf()
}

impl Engine {
    async fn get_times(self: &Arc<Self>, state: &ThinkState) -> EngineTimes {
        match &state.time_control {
            Some(tc) => match tc {
                UciTimeControl::Ponder => EngineTimes::inf(),
                UciTimeControl::Infinite => EngineTimes::inf(),
                UciTimeControl::TimeLeft {
                    white_time,
                    black_time,
                    white_increment,
                    black_increment,
                    moves_to_go,
                } => calc_time_left(
                    white_time.map(|x| x.to_std().unwrap_or_default()),
                    black_time.map(|x| x.to_std().unwrap_or_default()),
                    white_increment.map(|x| x.to_std().unwrap_or_default()),
                    black_increment.map(|x| x.to_std().unwrap_or_default()),
                    *moves_to_go,
                    state.our_side,
                ),
                UciTimeControl::MoveTime(x) => EngineTimes {
                    min: x.to_std().unwrap_or_default(),
                    max: x.to_std().unwrap_or_default(),
                },
            },
            None => EngineTimes {
                min: Duration::from_millis(5000),
                max: Duration::from_secs(10),
            },
        }
    }

    async fn find_moves(self: &Arc<Self>, state: &ThinkState, past_min_time: bool) -> EngineResult {
        let depth = match state.best_result {
            EngineResultState::Calculating => 1,
            EngineResultState::Ready(last) => last.stats.depth + 2,
            EngineResultState::Communicated(_) => panic!("Engine is not in a state to calculate"),
        };
        eprintln!("find_moves {} {}", past_min_time, depth);
        let board = self.internals.lock().await.board.clone();
        let settings = SearchSettings {
            depth,
            divide: false,
            ab_prune: true,
        };
        let mut res = { tokio::task::spawn_blocking(move || board.alphabeta(&settings, true)) }
            .await
            .unwrap();
        //eprintln!("got data: {:#?}", res.data);
        let best = res.data.pop();
        let response = res.data.pop();
        EngineResult {
            best_move: best.map(|x| x.mv.into()),
            stats: Stats {
                confidence: -0.1,
                depth,
            },
            ponder: response.map(|x| x.mv.into()),
            ..Default::default()
        }
    }

    async fn get_last_result(self: &Arc<Self>, state: &ThinkState) -> EngineResult {
        match state.best_result {
            EngineResultState::Calculating => self.internals.lock().await.get_move_immediately(),
            EngineResultState::Ready(last) => last,
            EngineResultState::Communicated(last) => last,
        }
    }

    #[async_recursion::async_recursion]
    async fn calculate(self: &Arc<Self>) -> EngineResult {
        let last_state = match &self.internals.lock().await.state {
            EngineState::Going(state) => state.clone(),
            EngineState::Pondering(state) => state.clone(),
            _ => {
                panic!("Engine is not in a state to calculate")
            }
        };

        let times = self.get_times(&last_state).await;

        let elapsed = last_state.start_time.elapsed();
        let past_min_time = elapsed >= times.min;
        let is_pondering = self.internals.lock().await.state.is_pondering()
            || last_state
                .time_control
                .as_ref()
                .map(|tc| matches!(tc, UciTimeControl::Ponder))
                .unwrap_or_default();

        let is_infinite = last_state
            .time_control
            .as_ref()
            .map(|tc| matches!(tc, UciTimeControl::Infinite))
            .unwrap_or_default();

        let past_max_time = elapsed >= times.max && !is_pondering && !is_infinite;

        let remaining = if is_pondering || is_infinite {
            Duration::from_secs(100000000)
        } else {
            match (times.max - elapsed).checked_div(2) {
                Some(x) => x,
                None => return self.calculate().await,
            }
        };

        eprintln!(
            "calculate {} {} {} {}",
            past_min_time,
            past_max_time,
            elapsed.as_millis(),
            remaining.as_millis()
        );
        if past_max_time {
            return self.get_last_result(&last_state).await;
        }
        match tokio::time::timeout(remaining, self.find_moves(&last_state, past_min_time)).await {
            Ok(result) => result,
            Err(_) => {
                let mut result = self.get_last_result(&last_state).await;
                result.out_of_time = true;
                result
            }
        }
    }

    fn send_bestmove(self: &Arc<Self>, mv: EngineResult) {
        self.send_uci_message(UciMessage::BestMove {
            best_move: mv.best_move.unwrap(),
            ponder: mv.ponder,
        });
    }

    async fn handle_message(self: &Arc<Self>, msg: UciMessage) {
        eprintln!("got message: {:?}", msg);
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
                let side = self.internals.lock().await.board.to_move();
                self.internals.lock().await.state =
                    EngineState::Going(ThinkState::new(time_control, search_control, side));
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
                let mut internal = self.internals.lock().await;
                match &internal.state {
                    EngineState::Pondering(state) => {
                        internal.state = EngineState::Going(state.clone());
                    }
                    _ => {}
                }
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
            EngineState::Pondering(state) => {
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
                    if !x.out_of_time && x.stats.confidence < 0.0 {
                        return None;
                    }
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
            //eprintln!("top of loop: {:?}", state);
            if !state.is_stopped() {
                let self2 = self.clone();
                let calc = spawn(async move { self2.calculate().await });
                let mut messages_recv = self.messages_recv.lock().await;
                let msg = messages_recv.recv();
                select! {
                    calc = calc => {
                        self.record_bestmove(calc.unwrap()).await;
                        if let Some(mv) = self.should_send_bestmove().await {
                            self.send_bestmove(mv);
                            let mut internal = self.internals.lock().await;
                            if let Some(ourmv) = mv.best_move && let Some(ponder) = mv.ponder && let EngineState::Going(mut state) = internal.state.clone() {
                                let board = internal.board.clone();
                                let ourmv: crate::chess::moves::Move = (&ourmv).into();
                                let ponder: crate::chess::moves::Move = (&ponder).into();
                                let board = board.apply_move(&ourmv).unwrap();
                                let board = board.apply_move(&ponder).unwrap();
                                internal.board = board;
                                state.adj_controls_for_ponder();
                                state.best_result = EngineResultState::Calculating;
                                internal.state = EngineState::Pondering(state);
                            } else {
                                internal.state = EngineState::Stopped;
                            }
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
        eprintln!("uci message: {}", uci.to_string());
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
