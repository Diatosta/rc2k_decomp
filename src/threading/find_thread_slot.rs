//! `threading::find_thread_slot` (0x410CC5): finds the `ThreadSlot` of the current thread.
//!
//! In:  nothing
//! Out: esi = slot pointer or NULL, ZF = (esi == 0); callers branch on ZF right after the call
//! No self-test: the result depends on which thread calls it.

use crate::framework::{RawHook, Regs, game_ptr, hook_asm, log, rva, shadow, trace};
use crate::threading::thread_slot::ThreadSlot;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "threading::find_thread_slot",
    rva(0x410CC5),
    thunk,
    &ORIGINAL,
);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    epilogue = "test esi, esi"
);

// ---- game data ----

const SLOTS_RVA: usize = rva(0x5184F4);

const _: () = assert!(size_of::<ThreadSlot>() == 0x110);
const _: () = assert!(std::mem::offset_of!(ThreadSlot, thread_id) == 4);

// ---- Rust version ----

/// Null if the current thread has no slot.
pub unsafe fn find_thread_slot() -> *const ThreadSlot {
    // Converted: `rc2k_core::threading::slot_of`.
    let slots = game_ptr::<ThreadSlot>(SLOTS_RVA).cast_const();
    let tid = unsafe { GetCurrentThreadId() };
    let id_of = |n: usize| unsafe { (&raw const (*slots.add(n)).thread_id).read_volatile() };
    match rc2k_core::threading::slot_of(id_of, tid) {
        Some(n) => {
            let slot = unsafe { slots.add(n) };
            if trace() {
                log(&format!("find_thread_slot: tid={tid:#x} slot={slot:p}"));
            }
            slot
        }
        None => std::ptr::null(),
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mine = unsafe { find_thread_slot() };

    if !shadow() {
        r.esi = mine as u32;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    let original = o.esi as *const ThreadSlot;
    if original != mine {
        HOOK.mismatch(|| format!("original={original:p} rust={mine:p}"));
    }
    r.esi = o.esi;
}
