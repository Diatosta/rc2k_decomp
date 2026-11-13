//! Finds the `ThreadSlot` of the current thread.
//!
//! Returns the slot, or none; the result depends on which thread calls it.

use crate::threading::thread_slot::ThreadSlot;

/// The number of the current thread's slot, None if it has none.
fn current_slot_number() -> Result<Option<usize>, crate::errors::BadInput> {
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
    Ok(found)
}

/// The current thread's slot, None if it has none.
pub fn current_slot() -> Result<Option<&'static ThreadSlot>, crate::errors::BadInput> {
    current_slot_number()?
        .map(crate::threading::thread_slot::slot_ref)
        .transpose()
}

/// The current thread's slot as one more than its number, 0 if it has none. For the fatal
/// reporter: a slot that cannot be read (the slots are a fixed set, so never) is 0 too, as a
/// report is being made already.
pub fn find_thread_slot() -> u32 {
    match current_slot_number() {
        Ok(Some(n)) => n as u32 + 1,
        Ok(None) | Err(_) => 0,
    }
}
