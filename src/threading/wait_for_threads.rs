//! `threading::wait_for_threads` (0x410BA8): the shutdown's wait: the input thread's,
//! the audio thread's and the network's receiving thread's slots (`g_inputThreadSlot`,
//! `g_audioThreadSlot`, `stru_518714`) each resumed if suspended (`sub_410D1D`), their
//! three handles waited for, all of them, three seconds at most
//! (`WaitForMultipleObjects`), and closed (`CloseHandle`, the last pushed first). The
//! handles are the three pushed as `sub_410D1D`'s arguments, which it leaves on the stack:
//! they are the array the wait gets.
//! `sub_410D1D` (`threading::resume_slot_thread`, not hooked: one Win32 call): with a slot
//! whose +0xC is 0xFF (`neg`: made 1), `ResumeThread` on its +8.
//!
//! In:  nothing
//! Out: eax the wait's result (into `popa`'s saved eax); the rest kept
//!
//! One caller, `threading::shutdown_thread`, which calls it directly (2026-10-23). The
//! registers above are the original's, for reference.

use crate::threading::thread_slot::{ThreadSlot, slot_ref};

/// `threading::resume_slot_thread` (0x410D1D): the slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1).
fn resume_slot_thread(slot: &ThreadSlot) {
    use windows_sys::Win32::System::Threading::ResumeThread;
    // Converted: `rc2k_core::threading::resumes`.
    if rc2k_core::threading::resumes(slot.flag.get()) {
        slot.flag.set(rc2k_core::threading::RESUMED);
        unsafe { ResumeThread(crate::pointers::handle(slot.handle.get())) };
    }
}

/// The Rust version: the game's three slots' threads (input, audio, network) resumed and
/// waited for, three seconds at most, their handles closed; the wait's result.
pub fn wait_for_threads() -> Result<u32, crate::errors::BadInput> {
    // Converted: `rc2k_core::threading::{wait_order, WAIT_MS}`.
    use rc2k_core::threading::{WAIT_MS, WAITED, wait_order};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::WaitForMultipleObjects;
    let mut handles = [0u32; WAITED];
    for (k, handle) in handles.iter_mut().enumerate() {
        let slot = slot_ref(k)?;
        *handle = slot.handle.get();
        resume_slot_thread(slot);
    }
    // The stack: the last pushed first.
    let array = wait_order(handles).map(crate::pointers::handle);
    let rc = unsafe { WaitForMultipleObjects(WAITED as u32, array.as_ptr(), 1, WAIT_MS) };
    for handle in array {
        unsafe { CloseHandle(handle) };
    }
    Ok(rc)
}
