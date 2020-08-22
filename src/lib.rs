use crossbeam::channel::Sender;

pub use game_state::GameState;

mod game_state;
pub mod sprite_loader;

pub mod events;
pub mod input;

pub mod entities;
pub mod systems;
pub mod utils;

trait ExpectSender<T> {
    fn send_expect(&self, msg: T);
}

impl<T> ExpectSender<T> for Sender<T> {
    fn send_expect(&self, msg: T) {
        self.send(msg).expect("We always expect this to send");
    }
}
