use amethyst::core::Time;
use amethyst::ecs::{Read, System, SystemData};
use amethyst::input::InputEvent as AmethystInputEvent;
use amethyst::prelude::*;
use amethyst::shrev::{EventChannel, ReaderId};
use crossbeam::channel::Sender;
use log::warn;

use crate::events::UserInput;
use crate::input::GameInput;
use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;
use crate::ExpectSender;

// the official guide recommends 0.3 but that seems slow
const REPEAT_DELAY: f32 = 0.3;
// the official guide recommends moving across the board in 0.5 seconds but that seems slow
const REPEAT_INTERVAL: f32 = 0.5 / 10.;

pub struct InputSystem {
    one: PlayerInput,
    two: Option<PlayerInput>,
    reader: ReaderId<AmethystInputEvent<GameInput>>,
}

pub struct PlayerInput {
    down_side_keys: Vec<(UserInput, f32)>,
    input_tx: Sender<UserInput>,
}

impl PlayerInput {
    fn action_pressed(&mut self, event: UserInput) {
        if event == UserInput::Left || event == UserInput::Right {
            self.down_side_keys.push((event, REPEAT_DELAY))
        }

        self.send_event(event);
    }

    fn action_released(&mut self, event: UserInput) {
        // retain all the keys that aren't our release event
        self.down_side_keys.retain(|(e, _)| *e != event);

        // once we release a key redo the repeat delay
        self.down_side_keys
            .iter_mut()
            .for_each(|(_, t)| *t = REPEAT_DELAY);
    }

    fn submit_down_keys(&mut self, time: &Read<'_, Time>) {
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

    fn send_event(&self, input_event: UserInput) {
        log::trace!("forwarding message {:?}", input_event);

        self.input_tx.send_expect(input_event);
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
                    let event = Into::<Option<UserInput>>::into(action);

                    match (action, &mut self.two) {
                        (_, None) if action.single() => {
                            self.one.action_pressed(event.unwrap());
                        }
                        (_, Some(_)) if action.one() => {
                            self.one.action_pressed(event.unwrap());
                        }
                        (_, Some(two)) if action.two() => {
                            two.action_pressed(event.unwrap());
                        }
                        (_, _) => {
                            warn!("Other action: {}", action);
                        }
                    }
                }
                AmethystInputEvent::ActionReleased(action) => {
                    let event = Into::<Option<UserInput>>::into(action);

                    match (action, &mut self.two) {
                        (_, None) if action.single() => {
                            self.one.action_released(event.unwrap());
                        }
                        (_, Some(_)) if action.one() => {
                            self.one.action_released(event.unwrap());
                        }
                        (_, Some(two)) if action.two() => {
                            two.action_released(event.unwrap());
                        }
                        (_, _) => {
                            warn!("Other action: {}", action);
                        }
                    }
                }
                _ => (),
            }
        }

        self.one.submit_down_keys(&time);
        self.two.as_mut().map(|p| p.submit_down_keys(&time));
    }
}

pub struct InputSystemDesc {
    pub one_input_tx: Sender<UserInput>,
    pub two_input_tx: Option<Sender<UserInput>>,
}

impl<'a, 'b> SystemDesc<'a, 'b, InputSystem> for InputSystemDesc {
    fn build(self, world: &mut World) -> InputSystem {
        <InputSystem as System<'_>>::SystemData::setup(world);

        let reader_id = world
            .fetch_mut::<EventChannel<AmethystInputEvent<GameInput>>>()
            .register_reader();

        InputSystem {
            one: PlayerInput {
                down_side_keys: Vec::with_capacity(2),
                input_tx: self.one_input_tx,
            },
            two: self.two_input_tx.map(|sender| PlayerInput {
                down_side_keys: Vec::with_capacity(2),
                input_tx: sender,
            }),
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
