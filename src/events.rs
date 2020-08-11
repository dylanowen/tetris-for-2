use serde::{Deserialize, Serialize};

/// Events coming into our game
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameRxEvent {
    Input(InputEvent),
    Tick,
    AddRows(usize),
}

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum InputEvent {
    Left,
    Right,
    RotateClockwise,
    DropSoft,
    DropHard,
    Hold,
}

/// Events coming out of our game
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameTxEvent {
    RxEvent(GameRxEvent),
    Lose,
}
