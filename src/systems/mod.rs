pub use self::debug_system::DebugBundle;
pub use self::game_system::GameSystemBundle;
pub use self::input_system::InputSystemDesc;
pub use self::network_system::NetworkSystemDesc;

mod debug_system;
mod game_system;
mod input_system;
mod network_system;

pub mod utils;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum KnownSystems {
    // Internal Systems
    InputSystem,
    Debug,
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
            KnownSystems::InputSystem => "control_system",
            KnownSystems::Debug => "debug_system",
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
