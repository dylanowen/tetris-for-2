use amethyst::core::ecs::{Read, System};
use amethyst::core::Time;
use amethyst::error::Error as AmethystError;
use amethyst::GameDataBuilder;
use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use rand::Rng;

use crate::events::{GameRxEvent, GameTxEvent};
use crate::systems::control::{calculate_tick, start_game, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, BOARD_WIDTH, PIXEL_DIMENSION};
use crate::systems::utils::{KnownSystem, WithKnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};

struct DoublePlayerSystem {
    started: bool,
    level: usize,
    tick_timer: f32,
    one_input_rx: Receiver<GameRxEvent>,
    one_tx: Sender<GameRxEvent>,
    one_rx: Receiver<GameTxEvent>,
    two_input_rx: Receiver<GameRxEvent>,
    two_tx: Sender<GameRxEvent>,
    two_rx: Receiver<GameTxEvent>,
}

// TODO create an event system for the entire game, we can reuse it for different things
// start game: send start events
// won
// lost
// etc
impl<'s> System<'s> for DoublePlayerSystem {
    type SystemData = Read<'s, Time>;

    fn run(&mut self, time: Self::SystemData) {
        if !self.started {
            self.started = true;

            start_game(&self.one_tx);
            start_game(&self.two_tx);
        }

        let (tick_timer, send_tick) = calculate_tick(self.tick_timer, self.level, &time);
        self.tick_timer = tick_timer;

        progress_player(send_tick, &self.one_input_rx, &self.one_tx);
        progress_player(send_tick, &self.two_input_rx, &self.two_tx);

        // read the output and see if anything interesting happened in our game
        while let Ok(_game_event) = self.one_rx.try_recv() {
            // TODO
        }
        while let Ok(_game_event) = self.two_rx.try_recv() {
            // TODO
        }
    }
}

fn progress_player(
    send_tick: bool,
    player_input_rx: &Receiver<GameRxEvent>,
    player_tx: &Sender<GameRxEvent>,
) {
    // forward all of our input events
    while let Ok(input_event) = player_input_rx.try_recv() {
        player_tx.send(input_event).expect("Always send")
    }

    if send_tick {
        // send our tick event
        player_tx
            .send(GameRxEvent::Tick(rand::thread_rng().gen()))
            .expect("Always send");
    }
}

pub fn setup<'a, 'b>(
    _: GameType,
    mut game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
    let (one_input_out_tx, one_input_out_rx) = channel::unbounded();
    let (two_input_out_tx, two_input_out_rx) = channel::unbounded();

    let (one_in_tx, one_in_rx) = channel::unbounded();
    let (one_out_tx, one_out_rx) = channel::unbounded();

    let (two_in_tx, two_in_rx) = channel::unbounded();
    let (two_out_tx, two_out_rx) = channel::unbounded();

    game_data = game_data
        .with_known_desc(InputSystemDesc {
            one_input_tx: one_input_out_tx,
            two_input_tx: Some(two_input_out_tx),
        })
        .with_system_desc(
            TetrisGameSystemDesc {
                position: (MARGIN, MARGIN),
                in_rx: one_in_rx,
                out_tx: one_out_tx,
            },
            "game_system_player_one",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_system_desc(
            TetrisGameSystemDesc {
                position: ((PIXEL_DIMENSION * BOARD_WIDTH as f32) + MARGIN * 2., MARGIN),
                in_rx: two_in_rx,
                out_tx: two_out_tx,
            },
            "game_system_player_two",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_known(DoublePlayerSystem {
            started: false,
            level: 3,
            tick_timer: 0.,
            one_input_rx: one_input_out_rx,
            one_tx: one_in_tx,
            one_rx: one_out_rx,
            two_input_rx: two_input_out_rx,
            two_tx: two_in_tx,
            two_rx: two_out_rx,
        });

    Ok(game_data)
}

impl KnownSystem<'_> for DoublePlayerSystem {
    fn name() -> KnownSystems {
        KnownSystems::ControlSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
