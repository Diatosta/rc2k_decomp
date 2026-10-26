//! `race::render3d_tidy` (0x4433A7): `race::free_point_records` (a `call` and a `retn`;
//! "render3d_tidy"). Eight callers (`sub_444940`, `sub_485570` and menus).
//!
//! In:  nothing
//! Out: eax = the list's total, ebx = the list; the rest preserved
//!
//! See `race/stage_free.rs`. It begins with its call: private copy, the call relocated,
//! manual patch. Shadow mode runs the original alone (it frees). Self-test: as the freeing's
//! (blocks of the test's in the list when it is empty, freed by each).

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::stage_free::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::render3d_tidy", rva(0x4433A7), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

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
