//! `input::axis_moved` (0x40939F): with eax 0 the axes' rest values taken (after 250 ms);
//! else which of the device's axes moved past 0x80 from rest, and which way (see
//! `input/axes.rs`).
//!
//! In:  eax = 0 to take the rest values, else anything
//! Out: carry when none (or after taking them); else eax the axis (+0x10 up); the rest kept
//!      (ZF too is the original's `test eax, eax`; no caller reads it)
//!
//! Callers: `screens::learn_axis` (two), four of the controller pages' loops. Shadow mode:
//! by state (the rest values) and the carry; with eax 0 the original alone (it sleeps).
//! Self-test: test axes and devices, each axis each way, none moved.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::input::axes::axis_moved;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::axis_moved", rva(0x40939F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let m = unsafe { axis_moved(r.eax) };
    // No caller reads ZF (they test the carry or go on with a `mov`).
    Ok(Left::carry(m.carry).with(EAX, m.eax))
}

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(0x4F8270)) as u32, 0x18)]
}

extern "C" fn dispatch(r: &mut Regs) {
    if r.eax == 0 && shadow() {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::service::harness::state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        (game_addr(rva(0x4F6A28)) as u32, 0x18),
        at(rva(0x4F6EF0)),
        (game_addr(rva(0x4F78E4)) as u32 + 0x14, 4),
    ]);
    let saved = capture(&kept);
    for (axes, moved, by) in [
        (0x3Fu32, 0usize, 0x100i32),
        (0x3F, 2, -0x200),
        (0x3B, 2, -0x200),
        (0x3F, 5, 0x81),
        (0x3F, 1, 0x80),
        (0, 0, 0x1000),
    ] {
        check_state(
            &mut t,
            &format!("axes {axes:#x} axis {moved} by {by:#x}"),
            call_original,
            Regs {
                eax: 1,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x4F6EF0), 0u32);
                game_write(rva(0x4F78E4) + 0x14, axes);
                let rest: [u32; 6] = [100, 200, 300, 400, 500, 600];
                game_write(rva(0x4F8270), rest);
                let mut now = rest;
                now[moved] = now[moved].wrapping_add(by as u32);
                game_write(rva(0x4F6A28), now);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
