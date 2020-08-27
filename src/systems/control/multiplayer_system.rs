use amethyst::core::ecs::shrev::EventChannel;
use amethyst::core::Time;
use amethyst::ecs::{Read, ReaderId, System, SystemData, World};
use amethyst::error::Error as AmethystError;
use amethyst::network::simulation::laminar::{LaminarNetworkBundle, LaminarSocket};
use amethyst::network::simulation::NetworkSimulationEvent;
use amethyst::prelude::*;
use amethyst::GameDataBuilder;
use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use log::info;

use crate::events::{TetrisIn, TetrisOut, UserInput};
use crate::systems::control::{sent_pieces, LocalAttackPlayer, LocalPlayer, MARGIN};
use crate::systems::input_system::InputSystemDesc;
use crate::systems::network::client_system::ClientSystemDesc;
use crate::systems::network::server_system::ServerSystemDesc;
use crate::systems::tetris::tetris_system::{TetrisGameSystemDesc, TetrisRenderingConfig};
use crate::systems::tetris::RENDERED_WIDTH;
use crate::systems::utils::{KnownSystem, WithKnownSystemDesc};
use crate::systems::{GameType, KnownSystems};
use crate::ExpectSender;

struct MultiplayerSystem {
    state: State,
    local_player: LocalAttackPlayer,
    player_net_tx: Sender<TetrisIn>,
    opponent_rx: Receiver<TetrisOut>,
    reader: ReaderId<NetworkSimulationEvent>,
}

#[derive(Copy, Clone, Debug)]
enum State {
    Started,
    ClientWaiting,
    ServerWaiting,
}

// TODO create an event system for the entire game, we can reuse it for different things
// start game: send start events
// won
// lost
// etc
impl<'s> System<'s> for MultiplayerSystem {
    type SystemData = (
        Read<'s, EventChannel<NetworkSimulationEvent>>,
        // track when we should emit Tick events
        Read<'s, Time>,
    );

    fn run(&mut self, (net_events, time): Self::SystemData) {
        match self.state {
            State::Started => {
                // drop all of our network events (we only needed the Connect event)
                net_events.read(&mut self.reader);

                self.local_player.process_input(&time);
            }
            State::ClientWaiting => {
                // we're a client so we can kick off our start event immediately
                self.state = State::Started;

                self.local_player.start_game();
            }
            State::ServerWaiting => {
                // either we've started a client and will never get this event, or we're a server waiting for it
                for net_event in net_events.read(&mut self.reader) {
                    if let NetworkSimulationEvent::Connect(_) = net_event {
                        self.state = State::Started;

                        self.local_player.start_game();
                    }
                }
            }
        }

        let net_tx = self.player_net_tx.clone();
        let (_, local_lost) = self
            .local_player
            .handle_events(|in_event| net_tx.send_expect(in_event));

        // read the opponent output and see if anything interesting happened in our game
        let mut remote_lost = false;
        while let Ok(opponent_event) = self.opponent_rx.try_recv() {
            match opponent_event {
                TetrisOut::RemovedRows(rows) => {
                    self.local_player.handle_opponent_lines(sent_pieces(rows));
                }
                TetrisOut::Lose => remote_lost = true,
                _ => (),
            }
        }

        if local_lost || remote_lost {
            panic!("Somebody won!")
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
            one_input_tx: input_out_tx,
            two_input_tx: None,
        })
        .with_system_desc(
            TetrisGameSystemDesc {
                position: (MARGIN, MARGIN),
                in_rx: player_in_rx,
                out_tx: player_out_tx,
                config: TetrisRenderingConfig::default(),
            },
            "game_system_player",
            &[KnownSystems::SpriteLoader.into()],
        )
        .with_system_desc(
            TetrisGameSystemDesc {
                position: (RENDERED_WIDTH + MARGIN * 2., MARGIN),
                in_rx: opponent_in_net_rx,
                out_tx: opponent_out_tx,
                config: TetrisRenderingConfig {
                    show_ghost: false,
                    show_next: false,
                    show_hold: false,
                },
            },
            "game_system_opponent",
            &[KnownSystems::SpriteLoader.into()],
        );

    let state;
    game_data = match game_type {
        GameType::Server(address) => {
            info!("Server listening on: {}", address);

            state = State::ServerWaiting;

            let socket = LaminarSocket::bind(address)?;
            // let listener = TcpListener::bind(address)?;
            //listener.set_nonblocking(true)?;

            game_data
                .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
                // .with_bundle(TcpNetworkBundle::new(Some(listener), 2048))?
                .with_known_desc(ServerSystemDesc {
                    player_out_tx: player_out_net_rx,
                    opponent_in_rx: opponent_in_net_tx,
                })
        }
        GameType::Client(server_address) => {
            state = State::ClientWaiting;

            // make sure we're binding a socket on our external interface
            let socket = LaminarSocket::bind("0.0.0.0:0")?;

            game_data
                .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
                //.with_bundle(TcpNetworkBundle::new(None, 2048))?
                .with_known_desc(ClientSystemDesc {
                    player_out_tx: player_out_net_rx,
                    opponent_in_rx: opponent_in_net_tx,
                    server_address,
                })
        }
        _ => unreachable!(),
    };

    Ok(game_data.with_known_desc(MultiplayerSystemDesc {
        state,

        input_rx: input_out_rx,
        player_tx: player_in_tx,
        player_rx: player_out_rx,
        player_net_tx: player_out_net_tx,
        opponent_rx: opponent_out_rx,
    }))
}

pub struct MultiplayerSystemDesc {
    state: State,
    input_rx: Receiver<UserInput>,
    player_tx: Sender<TetrisIn>,
    player_rx: Receiver<TetrisOut>,
    player_net_tx: Sender<TetrisIn>,
    opponent_rx: Receiver<TetrisOut>,
}

impl<'a, 'b> SystemDesc<'a, 'b, MultiplayerSystem> for MultiplayerSystemDesc {
    fn build(self, world: &mut World) -> MultiplayerSystem {
        <MultiplayerSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        MultiplayerSystem {
            state: self.state,
            local_player: LocalAttackPlayer::new(self.input_rx, self.player_tx, self.player_rx),
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
