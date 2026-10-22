//! `screens::classify_controller` (0x46B27A): the button and the direction held from the
//! controller's four values, into `dword_7559EC` (see `screens/controller.rs`).
//!
//! In:  ecx x, edx y, eax z, ebx the buttons' bits
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `screens::learn_axis`. Shadow mode: by state. Self-test: buttons none, one and
//! two, each direction held through 32 polls and not; the histories put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::controller::classify_controller;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::classify_controller",
    rva(0x46B27A),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { classify_controller(r.eax, r.ebx, r.ecx, r.edx) };
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(0x7551E4)) as u32, 0x10), at(rva(0x7559EC))]
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
    for (history, values) in [
        (
            [0u32, 0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF],
            [0x10u32, 0x4, 0xFFFF_FF00, 0x30],
        ),
        (
            [0xFFFF_FFFF, 0, 0xFFFF_FFFF, 0xFFFF_FFFF],
            [0x10, 0, 0x200, 0x30],
        ),
        (
            [0xFFFF_FFFF, 0xFFFF_FFFF, 0, 0xFFFF_FFFF],
            [0x100, 6, 0x10, 0x10],
        ),
        (
            [0xFFFF_FFFF, 0xFFFF_FFFF, 0xFFFF_FFFF, 0],
            [0x100, 0, 0x10, 0x10],
        ),
        ([0xFF, 0xFFFF_FFFF, 0, 0xFFFF_FFFF], [0x10, 0, 0x10, 0x10]),
        ([1, 2, 3, 4], [0, 1, 0, 0]),
    ] {
        check_state(
            &mut t,
            &format!("history {history:x?} values {values:x?}"),
            call_original,
            Regs {
                eax: values[0],
                ebx: values[1],
                ecx: values[2],
                edx: values[3],
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(rva(0x7551E4), history) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
