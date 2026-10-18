//! The camera's view steps under `sub_437A53` (stream B, 2026-10-18): the mode record
//! (`off_606320[mode]`, the mode `unk_71A65C` held to 0 ..= 0x33), the share of a
//! change done (`unk_719FE0` - `unk_71A668` held to 0 ..= the record's +0x3C, over it,
//! into `flt_605260`), and the tests' frame of `physics::take_view_a`, `take_view_b`,
//! `ease_view_angles`, `ease_view_blend`, `camera_height_bias`, `ease_view_turn`,
//! `place_camera`. What the variables are (the camera's mode, place, turns, a view's
//! figures) is a reading.

use crate::framework::{RawHook, Regs, SelfTest};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, Rust, capture, check_state, restore};

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

/// The figures by lot from `seed`: floats small, the mode, times, places.
pub fn set_view_inputs(seed: u32) {
    let mut f = vec![0u32; 0x80];
    test_floats(seed.wrapping_mul(41).wrapping_add(7), &mut f);
    let small = |v: u32| (f32::from_bits(v) % 4.0).to_bits();
    for (k, v) in f.iter().enumerate().take(0x40) {
        p(0x608B20 + 4 * k, small(*v));
    }
    // Whole numbers among them: angles and places.
    for (k, x) in [
        0x608B30usize,
        0x608B34,
        0x608B38,
        0x608B50,
        0x608B54,
        0x608B58,
        0x608B74,
        0x608B78,
        0x608B7C,
    ]
    .iter()
    .enumerate()
    {
        p(*x, (f[0x40 + k] as i32 % 0x40000) as u32);
    }
    for k in 0..3 {
        p(0x637638 + 4 * k, small(f[0x50 + k]));
        p(0x6416E8 + 4 * k, (f[0x53 + k] as i32 % 0x10000) as u32);
        p(0x71A998 + 4 * k, f[0x56 + k]);
    }
    for k in 0..0x10 {
        p(0x71A08C + 4 * k, small(f[0x60 + k]));
    }
    for (k, x) in [
        0x71A688usize,
        0x71A68C,
        0x71A690,
        0x71A694,
        0x71A698,
        0x71A69C,
        0x71A67C,
        0x71A680,
        0x71A684,
    ]
    .iter()
    .enumerate()
    {
        p(*x, f[0x70 + k]);
    }
    p(
        0x71A65C,
        [0u32, 1, 5, 0x33, 0x40, u32::MAX, 3, 7][(seed % 8) as usize],
    );
    p(0x71A660, seed % 6);
    p(0x719FE0, 1000 + seed * 37);
    p(0x71A668, 1000 + seed * 29 % 300);
    p(0x60631C, small(f[0x5A]));
    p(0x606290, small(f[0x5B]));
    p(0x60628C, small(f[0x5C]));
    p(0x6EF7CC, seed % 2);
    p(0x6238A4, [2u32, 1, 0][(seed % 3) as usize]);
}

/// The test of one of the seven: the original and the Rust version on the game's
/// variables (saved and put back), `record` the esi given (a test record for the two
/// `take_view`s), under three precisions.
pub fn view_test(hook: &RawHook, call_original: CallOriginal, rust: Rust) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let all = view_regions();
    let saved = capture(&all);
    let mut view = [0u32; 0x10];
    test_floats(3, &mut view);
    let v = view.as_mut_ptr() as u32;
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..16u32 {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs {
                        esi: v,
                        ..Regs::SENTINEL
                    },
                    &|| set_view_inputs(seed),
                    &all,
                    rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
