use amethyst::core::ecs::{Read, System};
use amethyst::core::Time;
use amethyst::error::Error as AmethystError;
use amethyst::GameDataBuilder;
use crossbeam::channel;

use crate::systems::control::{LocalPlayer, SinglePlayer, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, BOARD_WIDTH, PIXEL_DIMENSION};
use crate::systems::utils::{KnownSystem, WithKnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};

struct SinglePlayerSystem {
    started: bool,
    player: SinglePlayer,
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

            self.player.start_game();
        }

        self.player.process_input(&time);

        self.player.handle_events();
    }
}

pub fn setup<'a, 'b>(
    _: GameType,
    mut game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
    let (input_tx, input_rx) = channel::unbounded();

    let (player_in_tx, player_in_rx) = channel::unbounded();
    let (player_out_tx, player_out_rx) = channel::unbounded();

    game_data = game_data
        .with_known_desc(InputSystemDesc {
            one_input_tx: input_tx,
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
            player: SinglePlayer::new(input_rx, player_in_tx, player_out_rx),
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
