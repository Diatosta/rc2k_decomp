//! The game's three worker threads started: `threading::start_slot_thread` for the audio and
//! net slots and the input thread's. The threads whose routines are not ported end at once; the
//! audio thread's is `audio::thread`.

use crate::errors::Refused;
use crate::threading::thread_slot::SlotOwners;
/// The three slots, in the order they are started: input, audio, network.
pub const SLOTS: [usize; 3] = [
    crate::threading::thread_slot::INPUT,
    crate::threading::thread_slot::AUDIO,
    crate::threading::thread_slot::NET,
];

/// Each slot's start, by its number.
pub fn start_threads(start: &mut dyn FnMut(usize) -> Result<(), Refused>) -> Result<(), Refused> {
    for slot in SLOTS {
        start(slot)?;
    }
    Ok(())
}

/// The three slots' threads started (`threading::start_slot_thread`).
pub fn start_threads_in_game(owners: SlotOwners<'static>) -> Result<(), Refused> {
    start_threads(&mut |slot| {
        crate::threading::start_slot_thread::start_slot_thread(slot, owners)?;
        Ok(())
    })
}
