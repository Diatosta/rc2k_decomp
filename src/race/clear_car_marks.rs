//! `race::clear_car_marks` (0x46202A): the 0x40 dwords at `dword_5CD688` (the car's marks'
//! textures) 0 (see `race/car_draw.rs`).
//!
//! In:  nothing
//! Out: eax 0, ecx 0, edi past them
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: the buffer filled, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::car_draw::clear_car_marks;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_car_marks", rva(0x46202A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { clear_car_marks() })
}

fn regions() -> [Region; 1] {
    [(game_addr(rva(0x5CD688)) as u32, 0x100)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "filled",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &[0xAA; 0x100]),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
