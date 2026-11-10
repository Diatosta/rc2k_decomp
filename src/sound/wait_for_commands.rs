//! `sound::wait_for_commands` (0x409C1E): waits until the audio thread has taken every
//! command.
//!
//! Two "end of turn" commands (0xA) are queued; then, while commands are waiting
//! (0x409C01): a millisecond's sleep, the window's messages (`system::pump_messages`),
//! and the quit switch at 1 ends the wait.
//! Fourteen callers.

use crate::sound::queue::{END_OF_TURN, pending, send_command};
use crate::system::owner::system;

// ---- game data ----

// Converted: `rc2k_core::sound::set_up::wait_for_commands` (its parts the trait `Waiting`).
pub use rc2k_core::sound::set_up::{Waiting as Parts, wait_for_commands};

/// The parts in the game: the queue's by the Rust functions, the window's messages by
/// the game.
pub struct Game(pub [u32; 7]);

impl Parts for Game {
    fn end_turn(&mut self) {
        send_command(END_OF_TURN, self.0);
    }

    fn pending(&mut self) -> bool {
        pending()
    }

    fn sleep_a_millisecond(&mut self) {
        crate::system::sleep::sleep(1);
    }

    fn pump_messages(&mut self) {
        crate::platform::pump_messages_now();
    }

    fn quitting(&mut self) -> bool {
        system().session.flags.quitting.get() == 1
    }
}

/// With the game's parts: what `start` and the route call.
pub fn wait_in_game() {
    wait_for_commands(&mut Game([0; 7]));
}
