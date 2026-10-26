//! `threading::find_thread_slot` (0x410CC5): finds the `ThreadSlot` of the current thread.
//!
//! In:  nothing
//! Out: esi = slot pointer or NULL, ZF = (esi == 0); callers branch on ZF right after the call
//! No self-test: the result depends on which thread calls it.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::threading::thread_slot::{SLOTS, ThreadSlot};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

const _: () = assert!(size_of::<ThreadSlot>() == 0x110);
const _: () = assert!(std::mem::offset_of!(ThreadSlot, thread_id) == 4);

// ---- Rust version ----

/// The current thread's slot, None if it has none.
pub fn current_slot() -> Result<Option<&'static ThreadSlot>, crate::errors::BadInput> {
    // Converted: `rc2k_core::threading::slot_of`.
    let tid = unsafe { GetCurrentThreadId() };
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

/// Null if the current thread has no slot.
pub unsafe fn find_thread_slot() -> *const ThreadSlot {
    // Converted: `rc2k_core::threading::slot_of`.
    let slots = SLOTS.ptr().cast::<ThreadSlot>().cast_const();
    let tid = unsafe { GetCurrentThreadId() };
    let id_of = |n: usize| unsafe { (*slots.add(n)).thread_id.get() };
    match rc2k_core::threading::slot_of(id_of, tid) {
        Some(n) => unsafe { slots.add(n) },
        None => std::ptr::null(),
    }
}
