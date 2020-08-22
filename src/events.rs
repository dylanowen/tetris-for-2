use serde::{Deserialize, Serialize};

/// Events over the wire
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum NetworkEvent {
    GameRx(TetrisIn),
}

/// Events coming into our game
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TetrisIn {
    Start(u64),
    Tick,
    AddRows(usize),
    User(UserInput),
}

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UserInput {
    Left,
    Right,
    RotateClockwise,
    DropSoft,
    DropHard,
    Hold,
}

/// Events coming out of our game
#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TetrisOut {
    // returns all the valid input that was passed to the simulation
    ValidIn(TetrisIn),
    LockedPiece,
    RemovedRows(usize),
    Lose,
}
