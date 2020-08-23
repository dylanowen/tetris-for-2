use std::net::SocketAddr;

use amethyst::error::Error as AmethystError;
use amethyst::GameDataBuilder;

pub use player::*;

use crate::systems::tetris::PIXEL_DIMENSION;

mod double_player_system;
mod multiplayer_system;
mod player;
mod single_player_system;

pub const MARGIN: f32 = PIXEL_DIMENSION / 2. + 20.;
const ATTACK_LEVEL: usize = 3;

#[derive(Clone, Debug)]
pub enum GameType {
    Single,
    Double,
    CoOp,
    Server(String),
    Client(SocketAddr),
}

impl GameType {
    pub fn setup<'a, 'b>(
        self,
        game_data: GameDataBuilder<'a, 'b>,
    ) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
        match self {
            GameType::Single => single_player_system::setup(self, game_data),
            GameType::Double => double_player_system::setup(self, game_data),
            GameType::CoOp => todo!("add a coop mode"),
            GameType::Server(_) | GameType::Client(_) => multiplayer_system::setup(self, game_data),
        }
    }
}

fn sent_pieces(removed_lines: usize) -> usize {
    if removed_lines < 4 {
        removed_lines - 1
    } else {
        removed_lines
    }
}
