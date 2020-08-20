use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::Transform;
use amethyst::core::{HiddenPropagate, Parent, SystemBundle, Time};
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::{
    Builder, DispatcherBuilder, Entity, Read, ReadExpect, System, World, WorldExt, Write,
    WriteStorage,
};
use amethyst::error::Error as AmethystError;
use amethyst::input::InputEvent;
use amethyst::prelude::SystemDesc;
use amethyst::shred::{SetupHandler, SystemData};
use amethyst::shrev::{EventChannel, ReaderId};
use amethyst::ui::{
    get_default_font, Anchor, FontAsset, FontHandle, LineMode, UiText, UiTransform,
};
use amethyst::utils::fps_counter::{FpsCounter, FpsCounterBundle};

use crate::input::{GameActions, GameInput};
use crate::systems::utils::{AddKnownSystem, KnownSystem};
use crate::systems::KnownSystems;

/// This system displays a debug view overlay when the users presses F9
#[derive(SystemDesc)]
#[system_desc(name(DebugSystemDesc))]
pub struct DebugSystem {
    #[system_desc(event_channel_reader)]
    reader: ReaderId<InputEvent<GameInput>>,
}

impl DebugSystem {
    fn new(reader: ReaderId<InputEvent<GameInput>>) -> Self {
        DebugSystem { reader }
    }
}

impl<'s> System<'s> for DebugSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        // For handling the visibility
        Read<'s, EventChannel<InputEvent<GameInput>>>,
        WriteStorage<'s, HiddenPropagate>,
        // For rendering FPS
        ReadExpect<'s, Time>,
        ReadExpect<'s, FpsCounter>,
        // For updating debug values
        WriteStorage<'s, UiText>,
        Write<'s, DebugEntities, DebugEntities>,
    );

    fn run(
        &mut self,
        (events, mut hidden, time, fps_counter, mut ui_text, mut debug): Self::SystemData,
    ) {
        // grab all of our events
        for event in events.read(&mut self.reader) {
            if InputEvent::ActionReleased(GameActions::Debug) == *event {
                // flip whether we're hidden and set HiddenPropagate for our parent if we are
                debug.hidden = !debug.hidden;
                if debug.hidden {
                    hidden
                        .insert(debug.parent, HiddenPropagate::new())
                        .expect("We should never call this while our parent is still alive");
                } else {
                    hidden.remove(debug.parent);
                }
            }
        }

        debug.fps(
            || {
                // update our FPS every 20 frames to limit this moving too fast
                if time.frame_number() % 20 == 0 {
                    let fps = fps_counter.sampled_fps();
                    Some(format!("FPS: {:.*}", 2, fps))
                } else {
                    None
                }
            },
            &mut ui_text,
        );
    }
}

pub struct DebugEntities {
    parent: Entity,
    pub coordinates: Entity,
    pub fps: Entity,
    hidden: bool,
}

impl DebugEntities {
    // pub fn coordinates<F>(&self, text_fn: F, ui_text: &mut WriteStorage<UiText>)
    // where
    //     F: FnOnce() -> Option<String>,
    // {
    //     self.write(self.coordinates, ui_text, text_fn)
    // }

    pub fn fps<F>(&self, text_fn: F, ui_text: &mut WriteStorage<UiText>)
    where
        F: FnOnce() -> Option<String>,
    {
        self.write(self.fps, ui_text, text_fn)
    }

    #[inline]
    fn write<F>(&self, entity: Entity, ui_text: &mut WriteStorage<UiText>, text_fn: F)
    where
        F: FnOnce() -> Option<String>,
    {
        // TODO do we need to check if we're actually hidden?
        if !self.hidden {
            // grab our actual UIText element
            if let Some(text_element) = ui_text.get_mut(entity) {
                if let Some(text) = text_fn() {
                    // update our text if the fn actually has any value for us
                    text_element.text = text;
                }
            }
        }
    }
}

impl SetupHandler<DebugEntities> for DebugEntities {
    fn setup(world: &mut World) {
        let default_font = get_default_font(
            &world.read_resource::<Loader>(),
            &world.read_resource::<AssetStorage<FontAsset>>(),
        );

        // create our parent that is centered / stretched to be the dimensions of the screen
        let parent_pos = UiTransform::new(
            "debug_parent".to_string(),
            Anchor::Middle,
            Anchor::Middle,
            0.0,
            0.0,
            1.0,
            1.0,
            1.0,
        )
        .into_percent();

        // construct our parent entity that will be used to hide the debug view when disabled
        let parent = world
            .create_entity()
            .with(Transform::default())
            .with(parent_pos)
            .with(HiddenPropagate::new())
            .build();

        let fps = DebugTextConfig {
            anchor: Anchor::TopLeft,
            ..DebugTextConfig::default()
        }
        .build(default_font.clone(), parent, world);
        let coordinates = DebugTextConfig {
            anchor: Anchor::TopMiddle,
            ..DebugTextConfig::default()
        }
        .build(default_font, parent, world);

        world.insert(DebugEntities {
            parent,
            coordinates,
            fps,
            hidden: true,
        });
    }
}

/// A convenience struct for building the location of
struct DebugTextConfig {
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
    anchor: Anchor,
}

impl Default for DebugTextConfig {
    fn default() -> Self {
        DebugTextConfig {
            x: 0.0,
            y: 0.0,
            z: 1.0,
            width: 300.0,
            height: 30.0,
            anchor: Anchor::Middle,
        }
    }
}

impl DebugTextConfig {
    fn build(&self, default_font: FontHandle, parent: Entity, world: &mut World) -> Entity {
        let transform = UiTransform::new(
            "coordinates".to_string(),
            self.anchor,
            self.anchor,
            self.x,
            self.y,
            self.z,
            self.width,
            self.height,
        );

        let mut ui_text = UiText::new(
            default_font,
            "".to_string(),
            [1., 1., 1., 0.5],
            self.height,
            LineMode::Single,
            Anchor::Middle,
        );
        ui_text.align = Anchor::MiddleLeft;

        world
            .create_entity()
            .with(transform)
            .with(Parent { entity: parent })
            .with(ui_text)
            .build()
    }
}

#[derive(Default)]
pub struct DebugBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for DebugBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), AmethystError> {
        // add our FPS Counter Bundle first
        FpsCounterBundle::default().build(world, builder)?;

        // add our debug system
        builder.add_known(DebugSystemDesc::default().build(world));

        Ok(())
    }
}

impl KnownSystem<'_> for DebugSystem {
    fn name() -> KnownSystems {
        KnownSystems::Debug
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[KnownSystems::FPSCounter] //, KnownSystems::HideHierarchy]
    }
}
