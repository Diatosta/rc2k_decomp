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

use crate::errors::Refused;
/// The three slots, in the order they are started: input, audio, network.
pub const SLOTS: [usize; 3] = [
    crate::threading::thread_slot::INPUT,
    crate::threading::thread_slot::AUDIO,
    crate::threading::thread_slot::NET,
];

/// Each slot's start, by its address.
pub fn start_threads(start: &mut dyn FnMut(u32) -> Result<(), Refused>) -> Result<(), Refused> {
    for slot in SLOTS {
        start(crate::threading::thread_slot::slot(slot) as u32)?;
    }
    Ok(())
}

/// The three slots' threads started (`threading::start_slot_thread`).
pub fn start_threads_in_game() -> Result<(), Refused> {
    start_threads(&mut |slot| {
        unsafe {
            crate::threading::start_slot_thread::start_slot_thread(crate::pointers::host(slot))
        }?;
        Ok(())
    })
}
