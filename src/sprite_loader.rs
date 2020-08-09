use amethyst::assets::{AssetStorage, Loader};
use amethyst::ecs::System;
use amethyst::ecs::{Read, SystemData};
use amethyst::prelude::*;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};

use crate::systems::utils::KnownSystem;
use crate::systems::KnownSystems;

pub const PIXEL_DIMENSION: f32 = 100.;

pub struct Sprites {
    pub pixel_sprite: SpriteRender,
}

#[derive(Default)]
pub struct SpriteLoaderDesc;

// TODO there has to be a better way to initialize / share sprites
impl<'a, 'b> SystemDesc<'a, 'b, SpriteLoader> for SpriteLoaderDesc {
    fn build(self, world: &mut World) -> SpriteLoader {
        // setup data we need to initialize, but not actually run
        <Read<'a, AssetStorage<Texture>> as SystemData>::setup(&mut *world);
        <Read<'a, AssetStorage<SpriteSheet>> as SystemData>::setup(&mut *world);

        <SpriteLoader as System<'_>>::SystemData::setup(world);

        let texture_handle = {
            let loader = world.read_resource::<Loader>();
            let texture_storage = world.read_resource::<AssetStorage<Texture>>();
            loader.load(
                "sprites/pixel.png",
                ImageFormat::default(),
                (),
                &texture_storage,
            )
        };

        let sheet_handle = {
            let loader = world.read_resource::<Loader>();
            let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
            loader.load(
                "sprites/pixel.ron",
                SpriteSheetFormat(texture_handle),
                (),
                &sheet_storage,
            )
        };

        let pixel_sprite = SpriteRender {
            sprite_sheet: sheet_handle,
            sprite_number: 0,
        };

        world.insert(Sprites { pixel_sprite });

        SpriteLoader
    }
}

pub struct SpriteLoader;

impl<'s> System<'s> for SpriteLoader {
    type SystemData = ();
    fn run(&mut self, _: Self::SystemData) {}
}

impl KnownSystem<'_> for SpriteLoader {
    fn name() -> KnownSystems {
        KnownSystems::SpriteLoader
    }

    fn dependencies() -> &'static [KnownSystems] {
        &[
            KnownSystems::TextureProcessor,
            KnownSystems::SpriteSheetProcessor,
        ]
    }
}
