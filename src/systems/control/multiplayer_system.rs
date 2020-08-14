use std::net::TcpListener;

use amethyst::core::Time;
use amethyst::ecs::{Read, ReaderId, System, SystemData, World};
use amethyst::error::Error as AmethystError;
use amethyst::network::simulation::tcp::TcpNetworkBundle;
use amethyst::GameDataBuilder;
use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use log::info;

use crate::events::{GameRxEvent, GameTxEvent};
use crate::systems::control::{start_game, tick, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::network::client_system::ClientSystemDesc;
use crate::systems::network::server_system::ServerSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, BOARD_WIDTH, PIXEL_DIMENSION};
use crate::systems::utils::{KnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};
use amethyst::core::ecs::shrev::EventChannel;
use amethyst::network::simulation::NetworkSimulationEvent;
use amethyst::prelude::*;

struct MultiplayerSystem {
    started: bool,
    level: usize,
    tick_timer: f32,
    input_rx: Receiver<GameRxEvent>,
    player_tx: Sender<GameRxEvent>,
    player_rx: Receiver<GameTxEvent>,
    player_net_tx: Sender<GameRxEvent>,
    opponent_rx: Receiver<GameTxEvent>,
    reader: ReaderId<NetworkSimulationEvent>,
}

// TODO create an event system for the entire game, we can reuse it for different things
// start game: send start events
// won
// lost
// etc
impl<'s> System<'s> for MultiplayerSystem {
    type SystemData = (
        Read<'s, EventChannel<NetworkSimulationEvent>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (net_events, time): Self::SystemData) {
        if !self.started {
            // we're a client so we can kick off our start event immediately
            self.started = true;

            start_game(&self.player_tx);
        } else {
            // either we've started a client and will never get this event, or we're a server waiting for it
            for net_event in net_events.read(&mut self.reader) {
                if let NetworkSimulationEvent::Connect(_) = net_event {
                    start_game(&self.player_tx);
                }
            }
        }

        // forward all of our input events
        while let Ok(input_event) = self.input_rx.try_recv() {
            self.player_tx.send(input_event).expect("Always send")
        }

        self.tick_timer = tick(self.tick_timer, self.level, &time, &self.player_tx);

        // read the output and see if anything interesting happened in our game / forward it
        while let Ok(game_event) = self.player_rx.try_recv() {
            match game_event {
                GameTxEvent::RxEvent(rx_event) => {
                    self.player_net_tx.send(rx_event).expect("Always send");
                }
                _ => (),
            }
        }

        // read the opponent output and see if anything interesting happened in our game
        while let Ok(_game_event) = self.opponent_rx.try_recv() {
            // TODO
        }
    }
}

pub fn setup<'a, 'b>(
    game_type: GameType,
    mut game_data: GameDataBuilder<'a, 'b>,
) -> Result<GameDataBuilder<'a, 'b>, AmethystError> {
    let (input_out_tx, input_out_rx) = channel::unbounded();

    let (player_in_tx, player_in_rx) = channel::unbounded();
    let (player_out_tx, player_out_rx) = channel::unbounded();

    let (player_out_net_tx, player_out_net_rx) = channel::unbounded();

    let (opponent_in_net_tx, opponent_in_net_rx) = channel::unbounded();
    let (opponent_out_tx, opponent_out_rx) = channel::unbounded();

    game_data = game_data
        .with_known_desc(InputSystemDesc {
            input_tx: input_out_tx,
        })
        .with_system_desc(
            TetrisGameSystemDesc {
                position: (MARGIN, MARGIN),
                in_rx: player_in_rx,
                out_tx: player_out_tx,
            },
            "game_system_player",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_system_desc(
            TetrisGameSystemDesc {
                position: ((PIXEL_DIMENSION * BOARD_WIDTH as f32) + MARGIN * 2., MARGIN),
                in_rx: opponent_in_net_rx,
                out_tx: opponent_out_tx,
            },
            "game_system_opponent",
            &[KnownSystems::SpriteLoader.into()],
        );

    let started;
    game_data = match game_type {
        GameType::Server(address) => {
            info!("Server listening on: {}", address);

            // let socket = LaminarSocket::bind(address)?;
            let listener = TcpListener::bind(address)?;
            listener.set_nonblocking(true)?;

            started = true;
            game_data
                //.with_bundle(LaminarNetworkBundle::new(Some(socket)))?
                .with_bundle(TcpNetworkBundle::new(Some(listener), 2048))?
                .with_known_desc(ServerSystemDesc {
                    player_out_tx: player_out_net_rx,
                    opponent_in_rx: opponent_in_net_tx,
                })
        }
        GameType::Client(server_address) => {
            started = false;
            game_data
                .with_bundle(TcpNetworkBundle::new(None, 2048))?
                // .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
                .with_known_desc(ClientSystemDesc {
                    player_out_tx: player_out_net_rx,
                    opponent_in_rx: opponent_in_net_tx,
                    server_address,
                })
        }
        _ => unreachable!(),
    };

    Ok(game_data.with_known_desc(MultiplayerSystemDesc {
        started,

        input_rx: input_out_rx,
        player_tx: player_in_tx,
        player_rx: player_out_rx,
        player_net_tx: player_out_net_tx,
        opponent_rx: opponent_out_rx,
    }))
}

pub struct MultiplayerSystemDesc {
    started: bool,
    input_rx: Receiver<GameRxEvent>,
    player_tx: Sender<GameRxEvent>,
    player_rx: Receiver<GameTxEvent>,
    player_net_tx: Sender<GameRxEvent>,
    opponent_rx: Receiver<GameTxEvent>,
}

impl<'a, 'b> SystemDesc<'a, 'b, MultiplayerSystem> for MultiplayerSystemDesc {
    fn build(self, world: &mut World) -> MultiplayerSystem {
        <MultiplayerSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        MultiplayerSystem {
            started: self.started,
            level: 1,
            tick_timer: 0.,
            input_rx: self.input_rx,
            player_tx: self.player_tx,
            player_rx: self.player_rx,
            player_net_tx: self.player_net_tx,
            opponent_rx: self.opponent_rx,
            reader: reader_id,
        }
    }
}

impl KnownSystem<'_> for MultiplayerSystem {
    fn name() -> KnownSystems {
        KnownSystems::ControlSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
