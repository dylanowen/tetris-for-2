use std::net::SocketAddr;

use amethyst::error::Error as AmethystError;
use amethyst::prelude::*;
use crossbeam::channel::Sender;
use rand::Rng;

use crate::events::GameRxEvent;
use crate::systems::tetris::tetris_system::PIXEL_DIMENSION;
use amethyst::core::Time;

mod multiplayer_system;
mod single_player_system;

pub const MARGIN: f32 = PIXEL_DIMENSION / 2. + 20.;

#[derive(Clone, Debug)]
pub enum GameType {
    Local,
    Server(String),
    Client(SocketAddr),
}

impl GameType {
    pub fn setup<'a, 'b>(
        self,
        game_data: GameDataBuilder<'a, 'b>,
    ) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
        match self {
            GameType::Local => single_player_system::setup(self, game_data),
            GameType::Server(_) | GameType::Client(_) => multiplayer_system::setup(self, game_data),
        }
    }
}

fn start_game(board_tx: &Sender<GameRxEvent>) {
    let seed = rand::thread_rng().gen();
    board_tx
        .send(GameRxEvent::Start(seed))
        .expect("Always send");
}

fn tick(mut tick_timer: f32, level: usize, time: &Time, player_tx: &Sender<GameRxEvent>) -> f32 {
    // see if we need to forward a tick event
    tick_timer -= time.delta_seconds();
    if tick_timer <= 0. {
        let level_float = level as f32 - 1.;
        tick_timer = (0.8 - (level_float * 0.007)).powf(level_float);

        // send our tick event
        player_tx.send(GameRxEvent::Tick).expect("Always send");
    }

    tick_timer
}
