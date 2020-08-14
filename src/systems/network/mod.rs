use std::net::SocketAddr;

use amethyst::network::simulation::{DeliveryRequirement, TransportResource, UrgencyRequirement};
use amethyst::network::Bytes;
use crossbeam::channel::{Receiver, Sender};
use rmp_serde::{decode, encode};

use crate::events::{GameRxEvent, NetworkEvent};

pub mod client_system;
pub mod server_system;

pub fn send(event: &NetworkEvent, address: SocketAddr, net: &mut TransportResource) {
    let payload = encode::to_vec(&event).expect("We should always be able to serialize our events");

    net.send_with_requirements(
        address,
        &payload,
        DeliveryRequirement::ReliableOrdered(None),
        UrgencyRequirement::OnTick,
    );
}

pub fn handle_message(payload: &Bytes, input_tx: &Sender<GameRxEvent>) {
    let network_event =
        decode::from_read_ref::<_, NetworkEvent>(&payload).expect("We should only send valid data");

    log::trace!("Received message {:?}", network_event);

    match network_event {
        NetworkEvent::GameRx(game_event) => {
            input_tx.send(game_event).expect("Always send");
        }
    }
}

pub fn forward_events(
    other_address: SocketAddr,
    output_rx: &Receiver<GameRxEvent>,
    net: &mut TransportResource,
) {
    while let Ok(rx_event) = output_rx.try_recv() {
        log::trace!("Forwarding message {:?} to {}", rx_event, other_address);

        send(&NetworkEvent::GameRx(rx_event), other_address, net);
    }
}
