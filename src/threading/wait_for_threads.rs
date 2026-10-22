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
//! One caller, `threading::shutdown_thread`. The resumes through `part` (in the game
//! `resume_slot_thread` with `api`), the wait and the closes through `api` (the same
//! functions in the standalone). Shadow mode runs the original alone (handles are
//! closed). Self-test: a copy whose three slots are the test's and whose calls go to
//! noting stand-ins (the wait's array noted as its handles), against the Rust version.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{CLOSE_HANDLE, RESUME_THREAD, WAIT_MULTIPLE};

/// g_inputThreadSlot, g_audioThreadSlot, stru_518714: three slots of 0x110 bytes in a row.
const SLOTS: usize = 0x5184F4;
const SLOT_LEN: u32 = 0x110;

/// `threading::resume_slot_thread` (0x410D1D): the slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1).
pub fn resume_slot_thread(slot: u32, api: Api) {
    if slot == 0 {
        return;
    }
    let flag = (slot + 0xC) as *mut u8;
    unsafe {
        // Converted: `rc2k_core::threading::resumes`.
        if rc2k_core::threading::resumes(flag.read_volatile()) {
            flag.write_volatile(rc2k_core::threading::RESUMED);
            api(
                RESUME_THREAD,
                &[((slot + 8) as *const u32).read_unaligned()],
            );
        }
    }
}

/// The Rust version on the slots from `slots`: each resumed through `resume` (given the
/// slot), then their threads waited for and their handles closed through `api`. Returns
/// the wait's result.
pub fn wait_for_threads_with(
    slots: u32,
    resume: &mut dyn FnMut(u32) -> Result<(), Refused>,
    api: Api,
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::threading::{wait_order, WAIT_MS}`.
    use rc2k_core::threading::{WAIT_MS, WAITED, wait_order};
    let mut handles = [0u32; WAITED];
    for (k, handle) in handles.iter_mut().enumerate() {
        let slot = slots + SLOT_LEN * k as u32;
        *handle = unsafe { ((slot + 8) as *const u32).read_unaligned() };
        resume(slot)?;
    }
    // The stack: the last pushed first.
    let array = wait_order(handles);
    let rc = api(
        WAIT_MULTIPLE,
        &[WAITED as u32, array.as_ptr() as u32, 1, WAIT_MS],
    );
    for handle in array {
        api(CLOSE_HANDLE, &[handle]);
    }
    Ok(rc)
}

/// The Rust version: the game's three slots' threads (input, audio, network) resumed and
/// waited for, three seconds at most; the wait's result.
pub fn wait_for_threads(api: Api) -> u32 {
    let mut resume = |slot: u32| -> Result<(), Refused> {
        resume_slot_thread(slot, &mut crate::system::exit_parts::game_api);
        Ok(())
    };
    // `resume` never fails
    wait_for_threads_with(game_addr(rva(SLOTS)) as u32, &mut resume, api).unwrap_or(0)
}
