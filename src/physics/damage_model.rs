//! The car's damage and wear model under `sub_4AB5D6` (stream B, 2026-10-18):
//! `physics::damage_factors` (0x4AB5E6), `physics::damage_wear` (0x4ABEDF),
//! `physics::damage_gauges` (0x4AC817). Their figures: the work car's (`g_car`,
//! 0x719FD8 ..: the damage gauges `unk_71A284` .. `unk_71A33C`, the handling factors
//! `unk_71A410` ..), the model's own (`flt_75B340` .. 0x75B940: shares, tables of four
//! (x0, y0, x1, y1) for `physics::flow::table_lerp`, rates). What each figure is, is a
//! reading. The three are long straight runs: their Rust was generated from the
//! disassembly by idiom (x87 runs as blocks, the clamps as `hold_low` / `hold_high`, the
//! table lerps, the compares as `if`), the rest written by hand.
//!
//! This file: the regions and the shared test.

use crate::framework::{RawHook, Regs, SelfTest};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, p};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::CallOriginal;
use crate::service::harness::{Region, Rust, capture, check_state, restore};

/// 1.0 (`fld1; fstp`).
pub const ONE: u32 = 0x3F80_0000;

/// What the three read and write: the work car's block and the model's figures.
pub fn regions() -> Vec<Region> {
    vec![(a(0x719FD8), 0xFE0), (a(0x75B340), 0x600)]
}

/// Everything by lot from `seed` (floats small), the whole numbers used as indexes held
/// to their tables: the gear `unk_71A1AC` (into `dword_75B3BC`) 0 .. 7, the car kind
/// `unk_71A018` (into `flt_75B3E4`) 0 .. 1, `unk_71A004` / `dword_75B3E8` 0 .. 3, the
/// column tables `dword_75B8C0` .. 0 .. 15.
pub fn set_inputs(seed: u32) {
    let mut f = vec![0u32; 0x3F8 + 0x180];
    test_floats(seed.wrapping_mul(53).wrapping_add(11), &mut f);
    let small = |v: u32| (f32::from_bits(v) % 120.0).to_bits();
    for (k, v) in f.iter().enumerate().take(0x3F8) {
        p(0x719FD8 + 4 * k, small(*v));
    }
    for k in 0..0x180usize {
        p(0x75B340 + 4 * k, small(f[0x3F8 + k]));
    }
    p(0x71A1AC, seed % 8);
    p(0x75B3BC, seed % 8);
    p(0x71A018, seed % 2);
    p(0x75B3E4, seed % 2);
    p(0x71A004, seed % 4);
    p(0x75B3E8, seed % 4);
    for k in 0..0x10usize {
        p(0x75B8C0 + 8 * k, (k as u32 * 7 + seed) % 16);
        p(0x75B8C4 + 8 * k, (k as u32 * 3 + seed) % 16);
    }
    for (k, v) in f.iter().enumerate().take(0x40) {
        // `dword_75B73C`: whole numbers (`fild`).
        p(0x75B73C + 4 * k, v % 1000);
    }
    p(0x71A22C, seed % 3 / 2);
    p(0x71A614, [0u32, 0x10, 0x40, 8][(seed % 4) as usize]);
    p(0x71A208, [0u32, small(f[0x40])][(seed % 2) as usize]);
    p(0x71A438, seed % 2);
}

/// The test of one of the three: the original and the Rust version on the figures by
/// lot, under three precisions.
pub fn damage_test(hook: &RawHook, call_original: CallOriginal, rust: Rust) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let all = regions();
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..12u32 {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &|| set_inputs(seed),
                    &all,
                    rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
