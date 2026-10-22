//! `threading::start_threads` (0x410B84): the game's three worker threads started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `threading::start_slot_thread` for the slot at `slot` (0x5184F4), the audio thread's
//! (`g_audioThreadSlot`, 0x518604) and `stru_518714`. One caller, `start` (0x4010B1). In
//! the standalone program the threads whose routines are not ported end at once; the audio
//! thread's is (`audio::thread`).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::audio::buffer_play::{Leaves, leaf_entry};
use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};

/// The three slots, in the order they are started.
pub const SLOTS: [usize; 3] = [0x5184F4, 0x518604, 0x518714];

/// Each slot's start, by its address.
pub fn start_threads(start: &mut dyn FnMut(u32) -> Result<(), Refused>) -> Result<(), Refused> {
    for slot in SLOTS {
        start(game_addr(rva(slot)) as u32)?;
    }
    Ok(())
}

fn game_start(slot: u32) -> Result<(), Refused> {
    unsafe { crate::threading::start_slot_thread::start_slot_thread(slot as *mut u8) }?;
    Ok(())
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    start_threads(&mut game_start)?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    leaf_entry(r, rust)
}
