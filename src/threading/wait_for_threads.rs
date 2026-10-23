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

use crate::framework::{game_addr, rva};

/// g_inputThreadSlot, g_audioThreadSlot, stru_518714: three slots of 0x110 bytes in a row.
const SLOTS_RVA: usize = rva(0x5184F4);
const SLOT_LEN: u32 = 0x110;

/// `threading::resume_slot_thread` (0x410D1D): the slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1).
fn resume_slot_thread(slot: u32) {
    use windows_sys::Win32::System::Threading::ResumeThread;
    let flag = (slot + 0xC) as *mut u8;
    unsafe {
        // Converted: `rc2k_core::threading::resumes`.
        if rc2k_core::threading::resumes(flag.read_volatile()) {
            flag.write_volatile(rc2k_core::threading::RESUMED);
            ResumeThread(((slot + 8) as *const u32).read_unaligned() as _);
        }
    }
}

/// The Rust version: the game's three slots' threads (input, audio, network) resumed and
/// waited for, three seconds at most, their handles closed; the wait's result.
pub fn wait_for_threads() -> u32 {
    // Converted: `rc2k_core::threading::{wait_order, WAIT_MS}`.
    use rc2k_core::threading::{WAIT_MS, WAITED, wait_order};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::WaitForMultipleObjects;
    let slots = game_addr(SLOTS_RVA) as u32;
    let mut handles = [0u32; WAITED];
    for (k, handle) in handles.iter_mut().enumerate() {
        let slot = slots + SLOT_LEN * k as u32;
        *handle = unsafe { ((slot + 8) as *const u32).read_unaligned() };
        resume_slot_thread(slot);
    }
    // The stack: the last pushed first.
    let array = wait_order(handles);
    let rc = unsafe { WaitForMultipleObjects(WAITED as u32, array.as_ptr() as _, 1, WAIT_MS) };
    for handle in array {
        unsafe { CloseHandle(handle as _) };
    }
    rc
}
