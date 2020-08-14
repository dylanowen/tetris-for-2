use amethyst::ecs::{Read, System, SystemData};
use amethyst::input::InputEvent as AmethystInputEvent;
use amethyst::network::simulation::TransportResource;
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use crossbeam::channel::Sender;

use crate::events::{GameRxEvent, InputEvent};
use crate::input::GameInput;
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

pub struct InputSystem {
    input_tx: Sender<GameRxEvent>,
    reader: ReaderId<AmethystInputEvent<GameInput>>,
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        // TODO do we need this
        Read<'s, TransportResource>,
        // get the input events
        Read<'s, EventChannel<AmethystInputEvent<GameInput>>>,
    );

    fn run(&mut self, (_, input_events): Self::SystemData) {
        for input_event in input_events.read(&mut self.reader) {
            if let AmethystInputEvent::ActionPressed(action) = input_event {
                if let Some(simulation_event) = Into::<Option<InputEvent>>::into(action) {
                    log::debug!("forwarding message");

                    self.input_tx
                        .send(GameRxEvent::Input(simulation_event))
                        .expect("We should always be able to send this message");
                } else {
                    println!("other");
                }
            }
        }
    }
}

pub struct InputSystemDesc {
    pub input_tx: Sender<GameRxEvent>,
}

impl<'a, 'b> SystemDesc<'a, 'b, InputSystem> for InputSystemDesc {
    fn build(self, world: &mut World) -> InputSystem {
        <InputSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<AmethystInputEvent<GameInput>>>()
            .register_reader();

        InputSystem {
            input_tx: self.input_tx,
            reader: reader_id,
        }
    }
}

impl KnownSystem<'_> for InputSystem {
    fn name() -> KnownSystems {
        KnownSystems::InputSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
