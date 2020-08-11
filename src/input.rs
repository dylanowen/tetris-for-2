use core::fmt;

use amethyst::input::BindingTypes;
use serde::{Deserialize, Serialize};

use crate::events::InputEvent;

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameAxis {}

impl fmt::Display for GameAxis {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameActions {
    Left,
    Right,
    RotateClockwise,
    DropSoft,
    DropHard,
    Hold,
    Debug,
}

impl Into<Option<InputEvent>> for &GameActions {
    fn into(self) -> Option<InputEvent> {
        match self {
            GameActions::Left => Some(InputEvent::Left),
            GameActions::Right => Some(InputEvent::Right),
            GameActions::RotateClockwise => Some(InputEvent::RotateClockwise),
            GameActions::DropSoft => Some(InputEvent::DropSoft),
            GameActions::DropHard => Some(InputEvent::DropHard),
            GameActions::Hold => Some(InputEvent::Hold),
            GameActions::Debug => None,
        }
    }
}

impl fmt::Display for GameActions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GameInput;

impl BindingTypes for GameInput {
    type Axis = GameAxis;
    type Action = GameActions;
}
