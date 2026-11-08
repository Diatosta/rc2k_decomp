//! `threading::find_thread_slot` (0x410CC5): finds the `ThreadSlot` of the current thread.
//!
//! Returns the slot, or none; the result depends on which thread calls it.

use crate::threading::thread_slot::ThreadSlot;

const _: () = assert!(size_of::<ThreadSlot>() == 0x110);
const _: () = assert!(std::mem::offset_of!(ThreadSlot, thread_id) == 4);

// ---- Rust version ----

/// The current thread's slot, None if it has none.
pub fn current_slot() -> Result<Option<&'static ThreadSlot>, crate::errors::BadInput> {
    // Converted: `rc2k_core::threading::slot_of`.
    let tid = crate::sync::thread_id();
    let failed = std::cell::Cell::new(None);
    let found = rc2k_core::threading::slot_of(
        |n| match crate::threading::thread_slot::slot_ref(n) {
            Ok(slot) => slot.thread_id.get(),
            Err(e) => {
                failed.set(Some(e));
                0
            }
        },
        tid,
    );
    if let Some(e) = failed.take() {
        return Err(e);
    }
    found
        .map(crate::threading::thread_slot::slot_ref)
        .transpose()
}

/// The current thread's slot's game address, 0 if it has none. For the fatal reporter: a
/// slot that cannot be read (the slots are a fixed region, so never) is 0 too, as a report
/// is being made already.
pub fn find_thread_slot() -> u32 {
    match current_slot() {
        Ok(Some(slot)) => crate::pointers::game_address(std::ptr::from_ref(slot)),
        Ok(None) | Err(_) => 0,
    }
}
