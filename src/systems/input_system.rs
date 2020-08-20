use amethyst::core::Time;
use amethyst::ecs::{Read, System, SystemData};
use amethyst::input::InputEvent as AmethystInputEvent;
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use crossbeam::channel::Sender;
use log::warn;

use crate::events::{GameRxEvent, InputEvent};
use crate::input::GameInput;
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

// the official guide recommends 0.3 but that seems slow
const REPEAT_DELAY: f32 = 0.1;
// the official guide recommends moving across the board in 0.5 seconds but that seems slow
const REPEAT_INTERVAL: f32 = 0.01;

pub struct InputSystem {
    down_side_keys: Vec<(InputEvent, f32)>,
    input_tx: Sender<GameRxEvent>,
    reader: ReaderId<AmethystInputEvent<GameInput>>,
}

impl InputSystem {
    fn send_event(&self, input_event: InputEvent) {
        log::trace!("forwarding message {:?}", input_event);

        self.input_tx
            .send(GameRxEvent::Input(input_event))
            .expect("Always send");
    }
}

impl<'s> System<'s> for InputSystem {
    type SystemData = (
        // use the time resource to repeat key presses
        Read<'s, Time>,
        // get the input events
        Read<'s, EventChannel<AmethystInputEvent<GameInput>>>,
    );

    fn run(&mut self, (time, input_events): Self::SystemData) {
        for input_event in input_events.read(&mut self.reader) {
            match input_event {
                AmethystInputEvent::ActionPressed(action) => {
                    if let Some(event) = Into::<Option<InputEvent>>::into(action) {
                        if event == InputEvent::Left || event == InputEvent::Right {
                            self.down_side_keys.push((event, REPEAT_DELAY))
                        }

                        self.send_event(event);
                    } else {
                        warn!("Other action: {}", action);
                    }
                }
                AmethystInputEvent::ActionReleased(action) => {
                    if let Some(event) = Into::<Option<InputEvent>>::into(action) {
                        // retain all the keys that aren't our release event
                        self.down_side_keys.retain(|(e, _)| *e != event);

                        // once we release a key redo the repeat delay
                        self.down_side_keys
                            .iter_mut()
                            .for_each(|(_, t)| *t = REPEAT_DELAY);
                    }
                }
                _ => (),
            }

            // check for our first down key and submit it's event
            if !self.down_side_keys.is_empty() {
                let i = self.down_side_keys.len() - 1;
                self.down_side_keys[i].1 -= time.delta_seconds();
                if self.down_side_keys[i].1 <= 0. {
                    self.send_event(self.down_side_keys[i].0);
                    self.down_side_keys[i].1 = REPEAT_INTERVAL - self.down_side_keys[i].1;
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
            down_side_keys: Vec::with_capacity(2),
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
