//! `race::free_point_records` (0x41F992): `stru_5C6E50` freed, its total off
//! `dword_5C6F90`, `g_pointRecords` 0. One caller, `sub_4433A7`.
//!
//! In:  nothing
//! Out: eax = the list's total, ebx = the list; the rest preserved
//!
//! See `race/stage_free.rs`. Shadow mode runs the original alone (it frees). Refuses,
//! before anything, a list with a block while the game's own heap is in use. Self-test:
//! with the list empty in the game, blocks of the test's (none, one, three) freed by each.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::stage_free::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::free_point_records", rva(0x41F992), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { free(r, &POINT_RECORDS) } {
        Ok(g) => {
            r.eax = g.eax;
            r.ebx = g.ebx;
            // `memory::free_list` leaves carry clear, and nothing after it sets flags.
            r.set_carry(false);
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { free(r, &POINT_RECORDS) }?;
        r.eax = g.eax;
        r.ebx = g.ebx;
        // `memory::free_list` leaves carry clear, and nothing after it sets flags.
        r.set_carry(false);
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_free(t, &POINT_RECORDS, call_original)
}
