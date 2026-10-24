//! The camera's view steps under `sub_437A53` (stream B, 2026-10-18): the mode record
//! (`off_606320[mode]`, the mode `unk_71A65C` held to 0 ..= 0x33), the share of a
//! change done (`unk_719FE0` - `unk_71A668` held to 0 ..= the record's +0x3C, over it,
//! into `flt_605260`), and the tests' frame of `physics::take_view_a`, `take_view_b`,
//! `ease_view_angles`, `ease_view_blend`, `camera_height_bias`, `ease_view_turn`,
//! `place_camera`. The converted ones run `share_done_plain`, `record_plain` and
//! `view_test_floats` (`rc2k_core::physics::camera_view`). What the variables are (the
//! camera's mode, place, turns, a view's figures) is a reading.

use crate::physics::flow::{base, g, p, x87_run};

/// `off_606320`: the camera modes' records (0x34).
pub const MODES: usize = 0x606320;

/// esi as the original leaves it: the mode's record.
pub fn record() -> u32 {
    let mode = (g(0x71A65C) as i32).clamp(0, 0x33);
    g(MODES + 4 * mode as usize)
}

pub fn rd(x: u32) -> u32 {
    unsafe { (x as *const u32).read_unaligned() }
}

/// The share done (0x438498 .. 0x4384BD): eax as left (the time held).
pub fn share_done(record: u32) -> u32 {
    let mut eax = g(0x719FE0).wrapping_sub(g(0x71A668));
    if (eax as i32) < 0 {
        eax = 0;
    }
    if eax >= rd(record + 0x3C) {
        eax = rd(record + 0x3C);
    }
    p(0x62400C, eax);
    x87_run!(
        "fild dword ptr [{b} + 0x22400C]",
        "fidiv dword ptr [{s} + 0x3C]",
        "fstp dword ptr [{b} + 0x205260]";
        b = base(), s = record
    );
    eax
}
