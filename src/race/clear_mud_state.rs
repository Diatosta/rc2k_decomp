//! `race::clear_mud_state` (0x449E01): the mud's 20 state dwords 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/mud.rs`. One caller, `race::init_mud`. Self-test: the game's variables
//! filled with 0xAA, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::mud::{STATE_DWORDS, STATE_MORE, STATE_RVA, clear_mud_state};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::clear_mud_state", rva(0x449E01), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = vec![(game_addr(STATE_RVA) as u32, STATE_DWORDS * 4)];
    r.extend(STATE_MORE.map(at));
    r
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { clear_mud_state() };
    Ok(Left::nothing())
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
        "filled with 0xAA",
        call_original,
        Regs::SENTINEL,
        &|| {
            for &(a, n) in &regions {
                unsafe { std::ptr::write_bytes(a as *mut u8, 0xAA, n) };
            }
        },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
