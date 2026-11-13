//! The shutdown's wait: the input, audio and network threads' slots each resumed if suspended
//! (`threading::resume_slot_thread`), their three handles waited for, all of them, three
//! seconds at most, and closed. Gives the wait's result. Used by `threading::shutdown_thread`.

use crate::threading::thread_slot::{ThreadSlot, slot_ref};

/// The slot's thread resumed if it was
/// suspended (+0xC 0xFF, made 1). No thread is ever suspended (nothing sets 0xFF; the
/// `ResumeThread` it called did nothing), so only the flag is kept.
fn resume_slot_thread(slot: &ThreadSlot) {
    if rc2k_core::threading::resumes(slot.flag.get()) {
        slot.flag.set(rc2k_core::threading::RESUMED);
    }
}

/// The three threads resumed and waited for, three seconds at most, their handles closed;
/// the wait's result.
pub fn wait_for_threads() -> Result<u32, crate::errors::BadInput> {
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
