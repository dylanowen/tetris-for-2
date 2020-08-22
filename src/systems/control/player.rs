use amethyst::core::Time;
use crossbeam::channel::{Receiver, Sender};
use rand::Rng;

use crate::events::{TetrisIn, TetrisOut, UserInput};
use crate::systems::control::{sent_pieces, ATTACK_LEVEL};
use crate::ExpectSender;

pub trait LocalPlayer {
    fn level(&self) -> usize;

    fn tick_timer(&mut self) -> &mut f32;

    fn input_rx(&self) -> &Receiver<UserInput>;
    fn tetris_tx(&self) -> &Sender<TetrisIn>;
    fn tetris_rx(&self) -> &Receiver<TetrisOut>;

    fn start_game(&self) {
        let seed = rand::thread_rng().gen();
        self.tetris_tx().send_expect(TetrisIn::Start(seed));
    }

    fn process_input(&mut self, time: &Time) {
        // forward all of our input events
        while let Ok(input_event) = self.input_rx().try_recv() {
            self.tetris_tx().send_expect(TetrisIn::User(input_event))
        }

        // see if we need to forward a tick event
        *self.tick_timer() -= time.delta_seconds();
        if *self.tick_timer() <= 0. {
            let level_float = self.level() as f32 - 1.;
            *self.tick_timer() = (0.8 - (level_float * 0.007)).powf(level_float);

            // send our tick event
            self.tetris_tx().send_expect(TetrisIn::Tick);
        }
    }
}

pub struct SinglePlayer {
    level: usize,
    tick_timer: f32,
    input_rx: Receiver<UserInput>,
    tetris_tx: Sender<TetrisIn>,
    tetris_rx: Receiver<TetrisOut>,
}

impl SinglePlayer {
    pub fn new(
        input_rx: Receiver<UserInput>,
        tetris_tx: Sender<TetrisIn>,
        tetris_rx: Receiver<TetrisOut>,
    ) -> SinglePlayer {
        SinglePlayer {
            level: 0,
            tick_timer: 0.,
            input_rx,
            tetris_tx,
            tetris_rx,
        }
    }

    pub fn handle_events(&mut self) -> bool {
        while let Ok(game_event) = self.tetris_rx.try_recv() {
            match game_event {
                TetrisOut::RemovedRows(_rows) => {}
                TetrisOut::Lose => return true,
                _ => (),
            }
        }

        false
    }
}

impl LocalPlayer for SinglePlayer {
    fn level(&self) -> usize {
        self.level
    }

    fn tick_timer(&mut self) -> &mut f32 {
        &mut self.tick_timer
    }

    fn input_rx(&self) -> &Receiver<UserInput> {
        &self.input_rx
    }

    fn tetris_tx(&self) -> &Sender<TetrisIn> {
        &self.tetris_tx
    }

    fn tetris_rx(&self) -> &Receiver<TetrisOut> {
        &self.tetris_rx
    }
}

pub struct LocalAttackPlayer {
    pending_lines: usize,
    tick_timer: f32,
    input_rx: Receiver<UserInput>,
    tetris_tx: Sender<TetrisIn>,
    tetris_rx: Receiver<TetrisOut>,
}

impl LocalAttackPlayer {
    pub fn new(
        input_rx: Receiver<UserInput>,
        tetris_tx: Sender<TetrisIn>,
        tetris_rx: Receiver<TetrisOut>,
    ) -> LocalAttackPlayer {
        LocalAttackPlayer {
            pending_lines: 0,
            tick_timer: 0.,
            input_rx,
            tetris_tx,
            tetris_rx,
        }
    }

    /// return (number of lines sent, did we lose)
    pub fn handle_events<F>(&mut self, mut input_handler: F) -> (usize, bool)
    where
        F: FnMut(TetrisIn),
    {
        let mut to_send_pieces = 0;
        while let Ok(game_event) = self.tetris_rx.try_recv() {
            match game_event {
                TetrisOut::ValidIn(in_event) => input_handler(in_event),
                TetrisOut::RemovedRows(rows) => {
                    let mut lines = sent_pieces(rows);

                    if lines > self.pending_lines {
                        lines -= self.pending_lines;
                        self.pending_lines = 0;
                    } else {
                        self.pending_lines -= lines;
                        lines = 0;
                    }

                    if lines > 0 {
                        to_send_pieces += lines;
                    }
                }
                TetrisOut::LockedPiece => {
                    // we locked a piece so we can send all of our pending pieces
                    if self.pending_lines > 0 {
                        self.tetris_tx
                            .send(TetrisIn::AddRows(self.pending_lines))
                            .expect("Always send");
                        self.pending_lines = 0;
                    }
                }
                TetrisOut::Lose => return (0, true),
            }
        }

        (to_send_pieces, false)
    }

    pub fn handle_opponent_lines(&mut self, lines: usize) {
        self.pending_lines += lines;
    }
}

impl LocalPlayer for LocalAttackPlayer {
    fn level(&self) -> usize {
        ATTACK_LEVEL
    }

    fn tick_timer(&mut self) -> &mut f32 {
        &mut self.tick_timer
    }

    fn input_rx(&self) -> &Receiver<UserInput> {
        &self.input_rx
    }

    fn tetris_tx(&self) -> &Sender<TetrisIn> {
        &self.tetris_tx
    }

    fn tetris_rx(&self) -> &Receiver<TetrisOut> {
        &self.tetris_rx
    }
}
