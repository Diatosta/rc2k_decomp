//! `drive::pedal_inputs` (0x43A004): the work car's pedals from the input words. With the
//! keys' bit 0x4000 of `unk_71A440`: the throttle `unk_71A610` stepped by 0x10 toward 0x100
//! (`dword_613894` set) or 0, held there (by the sign of the steps, as the original); else
//! it is `dword_613894`. The same for the brake `unk_71A614` by bit 0x2000 and
//! `dword_613898`. The handbrake `unk_71A618` = `dword_61389C`, 0 when that is 0x100 or
//! less and `unk_71A400` is set.
//!
//! In:  nothing
//! Out: eax `dword_61389C`, ebx `unk_71A440`; the rest kept; no flags read
//!
//! Callers: `drive::car_controls`, `sub_460835`, `sub_460B2F`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) by lot: the bits, the inputs, the pedals about their ends.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::pedal_inputs", rva(0x43A004), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// One pedal: by the keys' `bit`, stepped from `input`, or set to it.
fn step_pedal(bit: u32, input: usize, at: usize) {
    p(at, pedal(g(0x71A440) & bit != 0, g(input), g(at)));
}

/// The Rust version.
// Converted: `rc2k_core::drive::controls::{pedal, handbrake}`.
pub unsafe fn pedal_inputs() -> Left {
    step_pedal(THROTTLE_KEY, 0x613894, 0x71A610);
    step_pedal(BRAKE_KEY, 0x613898, 0x71A614);
    let hand = g(0x61389C);
    p(0x71A618, handbrake(hand, || g(0x71A400)));
    Left::nothing().with(EAX, hand).with(EBX, g(0x71A440))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { pedal_inputs() })
}

fn regions() -> [crate::service::harness::Region; 1] {
    [(a(0x71A610), 12)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x71A610), 12),
        (a(0x71A440), 4),
        (a(0x613894), 12),
        (a(0x71A400), 4),
    ];
    let saved = capture(&all);
    for case in 0..48u32 {
        let set = || {
            p(
                0x71A440,
                [0u32, 0x4000, 0x2000, 0x6000][(case % 4) as usize],
            );
            p(0x613894, [0u32, 0x80, 0x100][(case / 4 % 3) as usize]);
            p(0x613898, [0x100u32, 0, 0x20][(case / 4 % 3) as usize]);
            p(0x61389C, [0u32, 0x100, 0x101, 0x40][(case % 4) as usize]);
            p(0x71A400, case / 12 % 2);
            p(0x71A610, [0u32, 8, 0xF8, 0x100, 0x10][(case % 5) as usize]);
            p(0x71A614, [0x100u32, 0, 4, 0xF5, 0x80][(case % 5) as usize]);
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
