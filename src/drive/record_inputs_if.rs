//! `drive::record_inputs_if` (0x46008D): `drive::record_inputs` while recording a replay
//! (`dword_5C5CE0` 1) for the cars below `dword_63DB94`.
//!
//! In:  nothing
//! Out: while recording eax `dword_63DB94`, then as `record_inputs` leaves the registers
//!      when called; no flags read
//!
//! Caller: `drive::replay_step` (`sub_460B2F`).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the mode, cars and counts.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::record_inputs_if", rva(0x46008D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::records_inputs`.
pub unsafe fn record_inputs_if() -> Left {
    let Some(records) =
        rc2k_core::drive::replay_frames::records_inputs(g(0x5C5CE0), g(0x71B3F8), || g(0x63DB94))
    else {
        return Left::nothing();
    };
    // The compare's `mov eax, dword_63DB94` stays when nothing else writes eax.
    let mut left = Left::nothing().with(EAX, g(0x63DB94));
    if records {
        let called = unsafe { crate::drive::record_inputs::record_inputs() };
        for &(reg, v) in &called.regs {
            left = left.with(reg, v);
        }
    }
    left
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { record_inputs_if() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &crate::drive::record_inputs::regions(),
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all: Vec<Region> = vec![
        (a(0x6138C4), 16),
        (a(0x71B3F8), 4),
        (a(0x6EF7DC), 4),
        (a(0x7194BC), 0x80),
        (a(0x5C5CE0), 4),
        (a(0x63DB94), 4),
    ];
    let saved = capture(&all);
    for case in 0..12u32 {
        let set = || {
            p(0x5C5CE0, case % 3);
            p(0x71B3F8, case / 3 % 3);
            p(0x63DB94, case % 2 + 1);
            p(0x6EF7DC, 1);
            for k in 0..4usize {
                p(0x6138C4 + 4 * k, (k as u32 + case) * 0x37);
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
