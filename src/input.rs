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

    OneDropHard,
    OneHold,

    TwoLeft,
    TwoRight,
    TwoRotateClockwise,
    TwoDropSoft,
    TwoDropHard,
    TwoHold,

    Debug,
}

impl GameActions {
    pub fn single(&self) -> bool {
        match self {
            GameActions::Left
            | GameActions::Right
            | GameActions::RotateClockwise
            | GameActions::DropSoft
            | GameActions::DropHard
            | GameActions::Hold => true,
            _ => false,
        }
    }

    pub fn one(&self) -> bool {
        match self {
            GameActions::Left
            | GameActions::Right
            | GameActions::RotateClockwise
            | GameActions::DropSoft
            | GameActions::OneDropHard
            | GameActions::OneHold => true,
            _ => false,
        }
    }

    pub fn two(&self) -> bool {
        match self {
            GameActions::TwoLeft
            | GameActions::TwoRight
            | GameActions::TwoRotateClockwise
            | GameActions::TwoDropSoft
            | GameActions::TwoDropHard
            | GameActions::TwoHold => true,
            _ => false,
        }
    }
}

impl Into<Option<InputEvent>> for &GameActions {
    fn into(self) -> Option<InputEvent> {
        match self {
            GameActions::Left | GameActions::TwoLeft => Some(InputEvent::Left),
            GameActions::Right | GameActions::TwoRight => Some(InputEvent::Right),
            GameActions::RotateClockwise | GameActions::TwoRotateClockwise => {
                Some(InputEvent::RotateClockwise)
            }
            GameActions::DropSoft | GameActions::TwoDropSoft => Some(InputEvent::DropSoft),
            GameActions::DropHard | GameActions::OneDropHard | GameActions::TwoDropHard => {
                Some(InputEvent::DropHard)
            }
            GameActions::Hold | GameActions::OneHold | GameActions::TwoHold => {
                Some(InputEvent::Hold)
            }
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
