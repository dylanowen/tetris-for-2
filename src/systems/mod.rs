pub use self::control::GameType;
pub use self::debug_system::DebugBundle;

mod control;
mod debug_system;
mod input_system;
mod network;
mod tetris;

pub mod utils;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KnownSystems {
    // Internal Systems
    ControlSystem,
    Debug,
    InputSystem,
    NetworkSystem,
    SpriteLoader,

    // Amethyst Systems
    FlyMovement,
    FPSCounter,
    HideHierarchy,
    Input,
    ParentHierarchy,
    SpriteSheetProcessor,
    TextureProcessor,
    TransformSystem,
}

impl Into<&'static str> for &KnownSystems {
    fn into(self) -> &'static str {
        match self {
            KnownSystems::ControlSystem => "control_system",
            KnownSystems::Debug => "debug_system",
            KnownSystems::InputSystem => "my_input_system",
            KnownSystems::NetworkSystem => "network_system",
            KnownSystems::SpriteLoader => "sprite_loader_system",

            KnownSystems::FlyMovement => "fly_movement",
            KnownSystems::FPSCounter => "fps_counter_system",
            KnownSystems::HideHierarchy => "hide_hierarchy_system",
            KnownSystems::Input => "input_system",
            KnownSystems::ParentHierarchy => "parent_hierarchy_system",
            KnownSystems::SpriteSheetProcessor => "sprite_sheet_processor",
            KnownSystems::TextureProcessor => "texture_processor",
            KnownSystems::TransformSystem => "transform_system",
        }
    }
}

impl Into<&'static str> for KnownSystems {
    fn into(self) -> &'static str {
        (&self).into()
    }
}

impl Into<String> for &KnownSystems {
    fn into(self) -> String {
        Into::<&str>::into(self).to_string()
    }
}

impl Into<String> for KnownSystems {
    fn into(self) -> String {
        (&self).into()
    }
}
