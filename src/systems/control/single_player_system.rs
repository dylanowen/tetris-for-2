use amethyst::core::ecs::{Read, System};
use amethyst::core::Time;
use amethyst::error::Error as AmethystError;
use amethyst::GameDataBuilder;
use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};

use crate::events::{GameRxEvent, GameTxEvent};
use crate::systems::control::{start_game, tick, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, BOARD_WIDTH, PIXEL_DIMENSION};
use crate::systems::utils::{KnownSystem, WithKnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};

struct SinglePlayerSystem {
    started: bool,
    level: usize,
    tick_timer: f32,
    input_rx: Receiver<GameRxEvent>,
    player_tx: Sender<GameRxEvent>,
    player_rx: Receiver<GameTxEvent>,
}

// TODO create an event system for the entire game, we can reuse it for different things
// start game: send start events
// won
// lost
// etc
impl<'s> System<'s> for SinglePlayerSystem {
    type SystemData = Read<'s, Time>;

    fn run(&mut self, time: Self::SystemData) {
        if !self.started {
            self.started = true;

            start_game(&self.player_tx);
        }

        // forward all of our input events
        while let Ok(input_event) = self.input_rx.try_recv() {
            self.player_tx.send(input_event).expect("Always send")
        }

        self.tick_timer = tick(self.tick_timer, self.level, &time, &self.player_tx);

        // read the output and see if anything interesting happened in our game
        while let Ok(_game_event) = self.player_rx.try_recv() {
            // TODO
        }
    }
}

pub fn setup<'a, 'b>(
    _: GameType,
    mut game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
    let (input_out_tx, input_out_rx) = channel::unbounded();

    let (player_in_tx, player_in_rx) = channel::unbounded();
    let (player_out_tx, player_out_rx) = channel::unbounded();

    game_data = game_data
        .with_known_desc(InputSystemDesc {
            one_input_tx: input_out_tx,
            two_input_tx: None,
        })
        .with_system_desc(
            TetrisGameSystemDesc {
                position: (MARGIN + (PIXEL_DIMENSION * BOARD_WIDTH as f32) / 2., MARGIN),
                in_rx: player_in_rx,
                out_tx: player_out_tx,
            },
            "game_system_player",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_known(SinglePlayerSystem {
            started: false,
            level: 3,
            tick_timer: 0.,
            input_rx: input_out_rx,
            player_tx: player_in_tx,
            player_rx: player_out_rx,
        });

    Ok(game_data)
}

impl KnownSystem<'_> for SinglePlayerSystem {
    fn name() -> KnownSystems {
        KnownSystems::ControlSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
