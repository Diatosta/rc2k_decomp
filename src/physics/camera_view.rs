//! The camera's view steps under `sub_437A53` (stream B, 2026-10-18): the mode record
//! (`off_606320[mode]`, the mode `unk_71A65C` held to 0 ..= 0x33), the share of a
//! change done (`unk_719FE0` - `unk_71A668` held to 0 ..= the record's +0x3C, over it,
//! into `flt_605260`), and the tests' frame of `physics::take_view_a`, `take_view_b`,
//! `ease_view_angles`, `ease_view_blend`, `camera_height_bias`, `ease_view_turn`,
//! `place_camera`. The converted ones run `share_done_plain`, `record_plain` and
//! `view_test_floats` (`rc2k_core::physics::camera_view`). What the variables are (the
//! camera's mode, place, turns, a view's figures) is a reading.

use crate::physics::flow::{a, base, g, p, x87_run};
use crate::service::harness::Region;
use rc2k_core::physics::camera_view as view;

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

/// The share done, converted (`rc2k_core::physics::camera_view::share_done`): eax as
/// left (the time held); the time held into `dword_62400C`, the share into
/// `flt_605260`.
pub fn share_done_plain(record: u32) -> u32 {
    let (held, share) = view::share_done(g(0x719FE0), g(0x71A668), rd(record + 0x3C));
    p(0x62400C, held);
    p(0x605260, share.to_bits());
    held
}

/// esi as the original leaves it, through the core's `mode_index`.
pub fn record_plain() -> u32 {
    g(MODES + 4 * view::mode_index(g(0x71A65C)) as usize)
}

/// Everything the seven write, and what they read that the tests set.
pub fn view_regions() -> Vec<Region> {
    vec![
        (a(0x62400C), 4),
        (a(0x605260), 4),
        (a(0x606288), 0x10), // 0x606288 .. 0x606294
        (a(0x60631C), 4),
        (a(0x606678), 0xD0),  // 0x606678 .. 0x606748
        (a(0x608B20), 0x100), // 0x608B20 .. 0x608C20
        (a(0x71A65C), 0x48),
        (a(0x719FE0), 4),
        (a(0x71A998), 12),
        (a(0x6EF7CC), 4),
        (a(0x6238A4), 4),
        (a(0x637638), 12),
        (a(0x6416E8), 12),
        (a(0x71A08C), 0x40),
        crate::math::turn_scratch_xz::scratch(),
    ]
}
