//! `threading::wait_for_threads` (0x410BA8): the shutdown's wait: the input thread's,
//! the audio thread's and the network's receiving thread's slots (`g_inputThreadSlot`,
//! `g_audioThreadSlot`, the net slot) each resumed if suspended (`threading::resume_slot_thread`),
//! their three handles waited for, all of them, three seconds at most
//! (`WaitForMultipleObjects`), and closed (`CloseHandle`, the last pushed first). The
//! handles are the three pushed as `threading::resume_slot_thread`'s arguments, which it leaves on
//! the stack: they are the array the wait gets.
//! 0x410D1D (`threading::resume_slot_thread`: one Win32 call): with a slot
//! whose +0xC is 0xFF (`neg`: made 1), `ResumeThread` on its +8.
//!
//! Returns the wait's result.
//!
//! One caller, `threading::shutdown_thread`, which calls it directly.

use crate::threading::thread_slot::{ThreadSlot, slot_ref};

/// `threading::resume_slot_thread` (0x410D1D): the slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1). No thread is ever suspended (nothing sets 0xFF; the
/// `ResumeThread` it called did nothing), so only the flag is kept.
fn resume_slot_thread(slot: &ThreadSlot) {
    // Converted: `rc2k_core::threading::resumes`.
    if rc2k_core::threading::resumes(slot.flag.get()) {
        slot.flag.set(rc2k_core::threading::RESUMED);
    }
}

/// The Rust version: the game's three slots' threads (input, audio, network) resumed and
/// waited for, three seconds at most, their handles closed; the wait's result.
pub fn wait_for_threads() -> Result<u32, crate::errors::BadInput> {
    // Converted: `rc2k_core::threading::{wait_order, WAIT_MS}`.
    use rc2k_core::threading::{WAIT_MS, WAITED, wait_order};
    let mut handles = [0u32; WAITED];
    for (k, handle) in handles.iter_mut().enumerate() {
        let slot = slot_ref(k)?;
        *handle = slot.handle.get();
        resume_slot_thread(slot);
    }
    // The stack: the last pushed first.
    let array = wait_order(handles);
    let rc = crate::sync::wait_all_threads(&array, WAIT_MS);
    for handle in array {
        crate::sync::close(handle);
    }
    Ok(rc)
}
