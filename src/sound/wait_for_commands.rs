//! `sound::wait_for_commands` (0x409C1E): waits until the audio thread has taken every
//! command.
//!
//! In:  nothing (the registers go into the two commands it sends)
//! Out: every register preserved
//! Two "end of turn" commands (0xA) are queued; then, while commands are waiting
//! (`sub_409C01`): a millisecond's sleep, the window's messages (`system::pump_messages`),
//! and the quit switch at 1 ends the wait.
//! Fourteen callers.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_read, rva};
use crate::sound::queue::{END_OF_TURN, Queue, pending, send_command};

// ---- game data ----

const QUIT_RVA: usize = rva(0x51837C); // dword_51837C

// ---- Rust version ----

// Converted: `rc2k_core::sound::set_up::wait_for_commands` (its parts the trait `Waiting`).
pub use rc2k_core::sound::set_up::{Waiting as Parts, wait_for_commands};

/// The parts in the game: the queue's by the Rust functions, the window's messages by
/// the game.
pub struct Game(pub [u32; 7]);

impl Parts for Game {
    fn end_turn(&mut self) {
        unsafe { send_command(&Queue::game(), END_OF_TURN, self.0) };
    }

    fn pending(&mut self) -> bool {
        unsafe { pending(&Queue::game()) }
    }

    fn sleep_a_millisecond(&mut self) {
        crate::system::sleep::sleep(1);
    }

    fn pump_messages(&mut self) {
        crate::calls::call(rva(0x410535), &mut Regs::default());
    }

    fn quitting(&mut self) -> bool {
        unsafe { game_read::<u32>(QUIT_RVA) == 1 }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    wait_for_commands(&mut Game([0; 7]));
    Ok(())
}
