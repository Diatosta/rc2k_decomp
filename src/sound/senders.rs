//! The game's small functions that send the audio thread one command and wait for it:
//! what they share, for their hooks and self-tests.
//!
//! Each is `pusha`, some registers set, a sender of one command (0xB, the music, or 9),
//! `sound::wait_for_commands`, `popa`; the tunes go through `sound::play_music` for it.
//! A file for one of them holds what differs: the registers the command is sent with.

use crate::framework::{Regs, game_read, rva};
use crate::sound::queue::{Queue, send_command, sent};
use crate::sound::wait_for_commands::{Game, wait_for_commands};

/// One such function.
pub struct Sender {
    /// The command it sends.
    pub command: u32,
    /// From the registers at entry: the registers the command is sent (and waited) with,
    /// or None if it sends nothing.
    pub sends: fn(&Regs) -> Option<Regs>,
}

/// `dispatch_sender` without its hook.
pub fn send_entry(r: &mut Regs, sender: &Sender) -> Result<(), crate::errors::Refused> {
    send_in_game(sender, r);
    Ok(())
}

/// For Rust callers: what one of the senders' Rust versions gives, sent and waited for, as
/// its hook does outside shadow mode.
pub fn send_in_game(sender: &Sender, entry: &Regs) {
    if let Some(with) = (sender.sends)(entry) {
        unsafe { send_command(&Queue::game(), sender.command, sent(&with)) };
        wait_for_commands(&mut Game(sent(&with)));
    }
}

/// For the `sends` functions: a game variable's value.
pub fn variable(at: usize) -> u32 {
    unsafe { game_read(rva(at)) }
}

/// What `sound::play_music` sends for a tune, a second tune and a volume, from the
/// registers its caller has.
pub fn music(entry: &Regs, tune: u32, second: u32, volume: u32) -> Regs {
    music_regs(
        entry,
        rc2k_core::sound::senders::Music::play(tune, second, volume),
    )
}

/// The registers command 0xB is sent with (converted: `rc2k_core::sound::senders::Music`).
pub fn music_regs(entry: &Regs, m: rc2k_core::sound::senders::Music) -> Regs {
    Regs {
        eax: m.tune,
        ebx: m.then,
        ecx: m.volume,
        edx: m.action,
        ..*entry
    }
}

/// `sound::play_tune_n`'s registers (converted: `rc2k_core::sound::senders::tune_command`).
pub fn tune_regs(entry: &Regs, n: u32) -> Option<Regs> {
    let m = rc2k_core::sound::senders::tune_command(
        n,
        variable(0x5180FC),
        variable(0x518100),
        variable(0x5180F4) == 1,
    )?;
    Some(music_regs(entry, m))
}
