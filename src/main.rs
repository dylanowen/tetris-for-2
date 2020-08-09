use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::renderer::plugins::RenderToWindow;
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::RenderFlat2D;
use amethyst::renderer::RenderingBundle;
use amethyst::utils::application_root_dir;
use amethyst::{Application, GameDataBuilder, LoggerConfig};
use log::LevelFilter;
use rmp_serde::{decode, encode};

use tetris_for_two::events::{GameRxEvent, InputEvent};
use tetris_for_two::input::GameInput;
use tetris_for_two::sprite_loader::SpriteLoaderDesc;
use tetris_for_two::systems::utils::WithKnownSystemDesc;
use tetris_for_two::systems::{ControlSystemDesc, GameSystemBundle};
use tetris_for_two::GameState;

fn main() -> amethyst::Result<()> {
    let mut logger_config = LoggerConfig::default();
    logger_config.level_filter = LevelFilter::Debug;
    amethyst::start_logger(logger_config);

    {
        let event = GameRxEvent::Input(InputEvent::RotateClockwise);

        let buf = encode::to_vec(&event).unwrap();

        println!("{:X?}", buf);

        println!("{:?}", decode::from_read_ref::<_, GameRxEvent>(&buf));
    }

    let app_root = application_root_dir()?;
    let resources_dir = app_root.join("resources");
    let assets_dir = resources_dir.join("assets");
    let config_dir = resources_dir.join("config");

    let display_config_path = config_dir.join("display.ron");
    let key_bindings_path = config_dir.join("input.ron");

    let game_data = GameDataBuilder::default()
        // Manages input events
        .with_bundle(InputBundle::<GameInput>::new().with_bindings_from_file(&key_bindings_path)?)?
        .with_bundle(TransformBundle::new())?
        // .with_known_desc(HideHierarchySystemDesc::default())
        // Manages the various Resources for a UI
        // .with_bundle(UiBundle::<GameInput>::new())?
        // This bundle handles the rendering
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([1., 1., 1., 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        // Our own systems
        .with_known_desc(SpriteLoaderDesc::default())
        .with_known_desc(ControlSystemDesc::default())
        .with_bundle(GameSystemBundle::default())?;

    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 60)
        .build(game_data)?;
    game.run();
    Ok(())
}
