use std::net::SocketAddr;

use amethyst::ecs::{Read, System, SystemData, Write};
use amethyst::network::simulation::{NetworkSimulationEvent, TransportResource};
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use crossbeam::channel::{Receiver, Sender};
use log::{debug, error, info};

use crate::events::{GameRxEvent, NetworkEvent};
use crate::systems::network::{forward_events, handle_message};
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

pub struct ServerSystem {
    player_out_tx: Receiver<GameRxEvent>,
    opponent_in_rx: Sender<GameRxEvent>,
    client_address: Option<SocketAddr>,
    reader: ReaderId<NetworkSimulationEvent>,
}

impl<'s> System<'s> for ServerSystem {
    type SystemData = (
        Write<'s, TransportResource>,
        Read<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, net_events): Self::SystemData) {
        for event in net_events.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => {
                    handle_message(payload, &self.opponent_in_rx)
                }
                NetworkSimulationEvent::Connect(addr) => {
                    info!("connected {}", addr);

                    if self.client_address.is_none() {
                        // get our client's address
                        self.client_address = Some(*addr);
                    }
                }
                NetworkSimulationEvent::Disconnect(_addr) => self.client_address = None,
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => error!("{:?}", event),
            }
        }

        if let Some(client_address) = self.client_address {
            forward_events(client_address, &self.player_out_tx, &mut net);
            // while let Ok(rx_event) = self.player_out_tx.try_recv() {
            //     // if let GameTxEvent::RxEvent(rx_event) = player_output {
            //     debug!("Forwarding message {:?} to {}", rx_event, client_address);
            //
            //     send(, client_address, &mut net);
            // }
        }
    }
}

pub struct ServerSystemDesc {
    pub player_out_tx: Receiver<GameRxEvent>,
    pub opponent_in_rx: Sender<GameRxEvent>,
}

impl<'a, 'b> SystemDesc<'a, 'b, ServerSystem> for ServerSystemDesc {
    fn build(self, world: &mut World) -> ServerSystem {
        <ServerSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        ServerSystem {
            player_out_tx: self.player_out_tx,
            opponent_in_rx: self.opponent_in_rx,
            client_address: None,
            reader: reader_id,
        }
    }
}

impl KnownSystem<'_> for ServerSystem {
    fn name() -> KnownSystems {
        KnownSystems::NetworkSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
