use amethyst::core::ecs::{Read, System};
use amethyst::core::Time;
use amethyst::error::Error as AmethystError;
use amethyst::GameDataBuilder;
use crossbeam::channel;

use crate::systems::control::{LocalAttackPlayer, LocalPlayer, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, BOARD_WIDTH, PIXEL_DIMENSION};
use crate::systems::utils::{KnownSystem, WithKnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};

struct DoublePlayerSystem {
    started: bool,
    one: LocalAttackPlayer,
    two: LocalAttackPlayer,
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

            self.one.start_game();
            self.two.start_game();
        }

        self.one.process_input(&time);
        self.two.process_input(&time);

        // read the output and see if anything interesting happened in our game
        let (one_lines, one_lost) = self.one.handle_events(|_| ());
        let (two_lines, two_lost) = self.two.handle_events(|_| ());

        // update our lines from our opponents
        self.one.handle_opponent_lines(two_lines);
        self.two.handle_opponent_lines(one_lines);

        if one_lost || two_lost {
            panic!("Somebody won!")
        }
    }
}

pub fn setup<'a, 'b>(
    _: GameType,
    mut game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
    let (one_input_tx, one_input_rx) = channel::unbounded();
    let (two_input_tx, two_input_rx) = channel::unbounded();

    let (one_in_tx, one_in_rx) = channel::unbounded();
    let (one_out_tx, one_out_rx) = channel::unbounded();

    let (two_in_tx, two_in_rx) = channel::unbounded();
    let (two_out_tx, two_out_rx) = channel::unbounded();

    game_data = game_data
        .with_known_desc(InputSystemDesc {
            one_input_tx,
            two_input_tx: Some(two_input_tx),
        })
        .with_system_desc(
            TetrisGameSystemDesc {
                position: ((PIXEL_DIMENSION * BOARD_WIDTH as f32) + MARGIN * 2., MARGIN),
                // position: (MARGIN, MARGIN),
                in_rx: one_in_rx,
                out_tx: one_out_tx,
            },
            "game_system_player_one",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_system_desc(
            TetrisGameSystemDesc {
                // position: ((PIXEL_DIMENSION * BOARD_WIDTH as f32) + MARGIN * 2., MARGIN),
                position: (MARGIN, MARGIN),
                in_rx: two_in_rx,
                out_tx: two_out_tx,
            },
            "game_system_player_two",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_known(DoublePlayerSystem {
            started: false,
            one: LocalAttackPlayer::new(one_input_rx, one_in_tx, one_out_rx),
            two: LocalAttackPlayer::new(two_input_rx, two_in_tx, two_out_rx),
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
