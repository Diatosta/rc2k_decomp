//! `race::clear_race_slots` (0x4621CD): the eight `g_raceSlots` (0x71BAC8, 0x78 each)
//! marked unused (first dword 0).
//!
//! In:  nothing
//! Out: edi = past the last record, ecx = 0; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init), twice. Shadow mode: the
//! Rust version, the slots put back, the original's, compared. Self-test: the game's slots
//! filled with values of the test's, both versions, put back afterwards.

use crate::errors::Refused;
use crate::framework::{ECX, EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_race_slots", rva(0x4621CD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { clear_race_slots(r) };
    Ok(Left::nothing().with(EDI, g.edi).with(ECX, g.ecx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &slot_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = slot_regions();
    let saved = capture(&regions);
    for fill in [0u32, 1, 0xAAAA_AAAA] {
        check_state(
            &mut t,
            &format!("slots {fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                for (n, &(a, _)) in regions.iter().enumerate() {
                    unsafe { (a as *mut u32).write_unaligned(fill ^ n as u32) };
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
