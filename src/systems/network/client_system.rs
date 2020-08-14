use std::net::SocketAddr;

use amethyst::ecs::{Read, System, SystemData, Write};
use amethyst::network::simulation::{NetworkSimulationEvent, TransportResource};
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use crossbeam::channel::{Receiver, Sender};
use log::error;

use crate::events::GameRxEvent;
use crate::systems::network::{forward_events, handle_message};
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

pub struct ClientSystem {
    player_out_tx: Receiver<GameRxEvent>,
    opponent_in_rx: Sender<GameRxEvent>,
    server_address: SocketAddr,
    reader: ReaderId<NetworkSimulationEvent>,
}

impl<'s> System<'s> for ClientSystem {
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
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => {}
            }
        }

        forward_events(self.server_address, &self.player_out_tx, &mut net);
    }
}

pub struct ClientSystemDesc {
    pub player_out_tx: Receiver<GameRxEvent>,
    pub opponent_in_rx: Sender<GameRxEvent>,
    pub server_address: SocketAddr,
}

impl<'a, 'b> SystemDesc<'a, 'b, ClientSystem> for ClientSystemDesc {
    fn build(self, world: &mut World) -> ClientSystem {
        <ClientSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        ClientSystem {
            player_out_tx: self.player_out_tx,
            opponent_in_rx: self.opponent_in_rx,
            server_address: self.server_address,
            reader: reader_id,
        }
    }
}

impl KnownSystem<'_> for ClientSystem {
    fn name() -> KnownSystems {
        KnownSystems::NetworkSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
