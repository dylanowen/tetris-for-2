use amethyst::derive::SystemDesc;
use amethyst::ecs::ReadExpect;
use amethyst::ecs::{Read, System, SystemData};
use amethyst::input::InputEvent as AmethystInputEvent;
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};

use crate::events::{GameRxEvent, InputEvent};
use crate::input::GameInput;
use crate::systems::game_system::GameChannels;
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

#[derive(SystemDesc)]
#[system_desc(name(ControlSystemDesc))]
pub struct ControlSystem {
    #[system_desc(event_channel_reader)]
    reader: ReaderId<AmethystInputEvent<GameInput>>,
}

impl ControlSystem {
    fn new(reader: ReaderId<AmethystInputEvent<GameInput>>) -> Self {
        ControlSystem { reader }
    }
}

impl<'s> System<'s> for ControlSystem {
    type SystemData = (
        // get the input events
        ReadExpect<'s, GameChannels>,
        Read<'s, EventChannel<AmethystInputEvent<GameInput>>>,
    );

    fn run(&mut self, (channels, input_events): Self::SystemData) {
        for input_event in input_events.read(&mut self.reader) {
            if let AmethystInputEvent::ActionPressed(action) = input_event {
                if let Some(simulation_event) = Into::<Option<InputEvent>>::into(action) {
                    println!("forwarding message");

                    channels
                        .player_in
                        .send(GameRxEvent::Input(simulation_event))
                        .expect("We should always be able to send this message");
                } else {
                    println!("other");
                }
            }
        }
    }
}

impl KnownSystem<'_> for ControlSystem {
    fn name() -> KnownSystems {
        KnownSystems::ControlSystem
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[]
    }
}
