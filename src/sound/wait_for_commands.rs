//! `sound::wait_for_commands`: waits until the audio thread has taken every command.
//!
//! An "end of turn" command is queued; then the thread sleeps until the audio thread has taken
//! every command and ended that turn (`sound::queue::wait_a_while`: the audio thread is told
//! and does not pause meanwhile). Every `STRETCH` of a long wait the window's messages are
//! taken, and the quit switch ends the wait.

use crate::sound::queue::{END_OF_TURN, send_command_in, wait_a_while};

/// The longest the wait goes without the window's messages being taken.
const STRETCH: std::time::Duration = std::time::Duration::from_millis(10);

// ---- game data ----

// The logic is `rc2k_core::sound::set_up::wait_for_commands` (its parts the trait `Waiting`).
pub use rc2k_core::sound::set_up::{Waiting as Parts, wait_for_commands};

/// The parts in the game: the queue's by the Rust functions, the window's messages by
/// the game.
pub struct Game {
    pub o: crate::owners::Owners<'static>,
}

impl Parts for Game {
    fn end_turn(&mut self) {
        send_command_in(self.o.audio, END_OF_TURN);
    }

    fn wait_a_while(&mut self) -> bool {
        wait_a_while(self.o.audio, STRETCH)
    }

    fn pump_messages(&mut self) {
        crate::platform::pump_messages_now(self.o);
    }

    fn quitting(&mut self) -> bool {
        self.o.system.session.flags.quitting.is_set()
    }
}

/// With the game's parts: what `start` and the route call.
pub fn wait_in_game(o: crate::owners::Owners<'static>) {
    wait_for_commands(&mut Game { o });
}
