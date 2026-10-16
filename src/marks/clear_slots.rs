//! `marks::clear_slots` (0x4C967B): `init_marks`' step 11, the first dword of the 0x20
//! slots of `g_markSlots` 0.
//!
//! In:  nothing
//! Out: esi 0
//!
//! See `marks/setup.rs`. One caller, `marks::init_marks`. Manual patch with a private copy
//! (its loop jumps back into the first five bytes). Self-test: the slots filled with 0xAA,
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::setup::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::clear_slots", rva(0x4C967B), thunk, &ORIGINAL)
    .private_copy(0x1E)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(SLOTS_RVA) as u32, (SLOTS * SLOT_LEN) as usize)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_slots(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "filled with 0xAA",
        call_original,
        Regs::SENTINEL,
        &|| unsafe {
            for &(a, n) in &regions {
                std::ptr::write_bytes(a as *mut u8, 0xAA, n);
            }
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
