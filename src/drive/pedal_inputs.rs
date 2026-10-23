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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, EBX};
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};

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
