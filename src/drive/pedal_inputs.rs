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

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::session::Dword;
use rc2k_core::drive::controls::{BRAKE_KEY, THROTTLE_KEY, handbrake, pedal};

/// One pedal: by the keys' `bit`, stepped from `input`, or set to it.
fn step_pedal(bit: u32, input: &Dword, at: &Dword) {
    at.set(pedal(
        CAR.input_keys.get() & bit != 0,
        input.get(),
        at.get(),
    ));
}

/// The Rust version.
// Converted: `rc2k_core::drive::controls::{pedal, handbrake}`.
pub unsafe fn pedal_inputs() {
    step_pedal(THROTTLE_KEY, &PHYSICS.input_throttle, &CAR.throttle);
    step_pedal(BRAKE_KEY, &PHYSICS.input_brake, &CAR.brake);
    let hand = PHYSICS.input_handbrake.get();
    CAR.handbrake.set(handbrake(hand, || {
        crate::physics::car_store::CAR.handbrake_failed.get()
    }));
}
