use serde::{Deserialize, Serialize};

/// Events over the wire
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum NetworkEvent {
    GameRx(GameRxEvent),
}

/// Events coming into our game
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum GameRxEvent {
    Start(u64),
    Input(InputEvent),
    Tick(u64),
    AddRows(usize),
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
    RemovedRow,
    Lose,
}
