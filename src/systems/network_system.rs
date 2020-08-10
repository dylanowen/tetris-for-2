use std::net::SocketAddr;

use amethyst::derive::SystemDesc;
use amethyst::ecs::ReadExpect;
use amethyst::ecs::{Read, System, SystemData, Write};
use amethyst::input::InputEvent as AmethystInputEvent;
use amethyst::network::simulation::{NetworkSimulationEvent, TransportResource};
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use log::{debug, error, info};
use rmp_serde::{decode, encode};

use crate::events::{GameRxEvent, GameTxEvent, InputEvent};
use crate::input::GameInput;
use crate::systems::game_system::GameChannels;
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

pub struct NetworkSystem {
    other_address: Option<SocketAddr>,
    reader: ReaderId<NetworkSimulationEvent>,
}

impl<'s> System<'s> for NetworkSystem {
    type SystemData = (
        ReadExpect<'s, GameChannels>,
        Write<'s, TransportResource>,
        Read<'s, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (channels, mut net, net_events): Self::SystemData) {
        for event in net_events.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => {
                    let other_event = decode::from_read_ref::<_, GameRxEvent>(&payload)
                        .expect("We should only send valid data");

                    channels
                        .opponent_in
                        .send(other_event)
                        .expect("we should always be able to send this");
                }
                NetworkSimulationEvent::Connect(addr) => {
                    info!("connected {}", addr);

                    if self.other_address.is_none() {
                        self.other_address = Some(addr.clone());
                    }
                }
                NetworkSimulationEvent::Disconnect(addr) => self.other_address = None,
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => {}
            }
        }

        if let Some(other_address) = self.other_address {
            while let Ok(player_output) = channels.player_out.try_recv() {
                if let GameTxEvent::RxEvent(rx_event) = player_output {
                    debug!("Forwarding message {:?} to {}", rx_event, other_address);

                    let payload = encode::to_vec(&rx_event).unwrap();

                    net.send(other_address, &payload);
                }
            }
        }
    }
}

pub struct NetworkSystemDesc {
    pub other_address: Option<SocketAddr>,
}

impl<'a, 'b> SystemDesc<'a, 'b, NetworkSystem> for NetworkSystemDesc {
    fn build(self, world: &mut World) -> NetworkSystem {
        <NetworkSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        NetworkSystem {
            other_address: self.other_address,
            reader: reader_id,
        }
    }
}

impl KnownSystem<'_> for NetworkSystem {
    fn name() -> KnownSystems {
        KnownSystems::NetworkSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
