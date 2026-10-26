//! `marks::bound_spline_points` (0x4B9E7D): record ebp's points' box: its middle made the
//! centre, its angle from `g_marksSplineUp` into +0x20.
//!
//! In:  ebp the spline record
//! Out: eax, ebx, ecx the middle, esi `g_marksSplineUp`, edi `g_marksSplineSize`
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (four). Refuses more than 0x80 points.
//! Self-test: the original and the Rust version on a record of the test's (0 .. 0x80
//! points) with the two vectors by lot, the refusal; the game's vectors put back
//! afterwards.
//!
//! Converted 2026-10-21: the box through `rc2k_core::marks::splines::bounds`; the sizes
//! made floats and the angle stay the x87's. Compared exactly as before.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::bound_spline_points",
    rva(0x4B9E7D),
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

fn regions(spline: u32) -> Vec<Region> {
    let mut r = work_regions();
    r.push((spline, SPLINE_LEN as usize));
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { bound_spline_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r.ebp);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    let spline = TestSpline::new();
    let regions = regions(spline.addr());
    let saved = capture(&regions);
    for points in [0u32, 1, 5, MOST_POINTS] {
        for seed in [1u32, 2, 3] {
            check_state(
                &mut t,
                &format!("{points:#x} points, seed {seed}"),
                call_original,
                Regs {
                    ebp: spline.addr(),
                    ..Regs::SENTINEL
                },
                &|| {
                    spline.fill(seed * 0x115 + points, points);
                    let mut v = [0u32; 6];
                    crate::scene::matrix3::test_floats(seed + points, &mut v);
                    unsafe { crate::framework::game_write(UP_RVA, v) };
                },
                &regions,
                &rust,
            );
        }
    }
    let r = Regs {
        ebp: spline.addr(),
        ..Regs::SENTINEL
    };
    spline.fill(3, MOST_POINTS + 1);
    t.check(unsafe { bound_spline_points(&r) }.is_err(), || {
        "0x81 points: the Rust version went on".into()
    });
    restore(&regions, &saved);
    t.finish()
}
