//! `marks::order_spline_points` (0x4BA388): record ebp's ids and points put in grade order,
//! each grade's start and count noted.
//!
//! In:  ebp the spline record
//! Out: eax +0x654, ebx `g_marksOrderedCoords`, ecx the count times 12, esi the last point
//!      taken times 12, edi 0
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (two). Only the ids and points move (the
//! two values and the grades stay: section 16), and as many as the count come back,
//! whatever their grade. Refuses more than 0x80 points or a negative count. Self-test: the
//! original and the Rust version on a record of the test's (grades 0 .. 3 by lot), the
//! refusal; the game's work tables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::order_spline_points",
    rva(0x4BA388),
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
    Ok(left_of(&unsafe { order_spline_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r.ebp);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    let regions = regions(spline.addr());
    let saved = capture(&regions);
    for points in [0u32, 1, 7, 0x40, MOST_POINTS] {
        for seed in [1u32, 2, 3] {
            check_state(
                &mut t,
                &format!("{points:#x} points, seed {seed}"),
                call_original,
                Regs {
                    ebp: spline.addr(),
                    ..Regs::SENTINEL
                },
                &|| spline.fill(seed * 0x11B + points, points),
                &regions,
                &rust,
            );
        }
    }
    spline.fill(3, MOST_POINTS + 1);
    t.check(
        unsafe {
            order_spline_points(&Regs {
                ebp: spline.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "0x81 points: the Rust version went on".into(),
    );
    restore(&regions, &saved);
    t.finish()
}
