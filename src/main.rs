use amethyst::core::frame_limiter::FrameRateLimitStrategy;
use amethyst::core::TransformBundle;
use amethyst::input::InputBundle;
use amethyst::renderer::plugins::RenderToWindow;
use amethyst::renderer::types::DefaultBackend;
use amethyst::renderer::RenderFlat2D;
use amethyst::renderer::RenderingBundle;
use amethyst::utils::application_root_dir;
use amethyst::{Application, GameDataBuilder, Logger, LoggerConfig};
use chrono::Local;
use clap::{App, AppSettings, Arg, SubCommand};
use log::LevelFilter;

use tetris_for_two::input::GameInput;
use tetris_for_two::sprite_loader::SpriteLoaderDesc;
use tetris_for_two::systems::utils::WithKnownSystemDesc;
use tetris_for_two::systems::GameType;
use tetris_for_two::GameState;

fn main() -> amethyst::Result<()> {
    let mut logger_config = LoggerConfig::default();
    // hide the metal error logs until the bug is fixed
    logger_config.log_gfx_backend_level = Some(LevelFilter::Error);
    logger_config.level_filter = LevelFilter::Debug;
    Logger::from_config_formatter(logger_config, |out, message, record| {
        out.finish(format_args!(
            "{} [{level}][{target}] {message}",
            Local::now().format("%Y-%m-%dT%H:%M:%S"),
            level = record.level(),
            target = record.target(),
            message = message,
        ))
    })
    .start();

    let app_root = application_root_dir()?;
    let resources_dir = app_root.join("resources");
    let assets_dir = resources_dir.join("assets");
    let config_dir = resources_dir.join("config");

    let display_config_path = config_dir.join("display.ron");
    let key_bindings_path = config_dir.join("input.ron");

    let matches = App::new("tetris-for-two")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("server").arg(Arg::with_name("address").required(true)))
        .subcommand(SubCommand::with_name("client").arg(Arg::with_name("address").required(true)))
        .subcommand(SubCommand::with_name("local"))
        .get_matches();

    let game_type = match matches.subcommand() {
        ("server", Some(sub_matches)) => {
            let address = sub_matches.value_of("address").unwrap();
            GameType::Server(address.to_string())
        }
        ("client", Some(sub_matches)) => {
            let address = sub_matches.value_of("address").unwrap();
            GameType::Client(address.parse().expect("should parse"))
        }
        ("local", _) => GameType::Local,
        _ => unreachable!(),
    };

    let mut game_data = GameDataBuilder::default()
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
        .with_known_desc(SpriteLoaderDesc::default());

    game_data = game_type.setup(game_data)?;

    let mut game = Application::build(assets_dir, GameState)?
        .with_frame_limit(FrameRateLimitStrategy::Unlimited, 60)
        .build(game_data)?;
    game.run();
    Ok(())
}
