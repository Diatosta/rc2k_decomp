//! `marks::find_farthest_points` (0x4BA578): the two farthest apart of eax points at esi,
//! looked for inside a `pusha` .. `popa` and so kept nowhere (section 16).
//!
//! In:  eax the count, esi the points (12 bytes)
//! Out: every register kept, nothing written
//!
//! See `marks/splines.rs`. One caller, `sub_4B9D5F`. Self-test: the original and the Rust
//! version (which does nothing) on a record of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::find_farthest_points",
    rva(0x4BA578),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&find_farthest_points(r)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    for points in [0u32, 1, 2, 0x20] {
        check_state(
            &mut t,
            &format!("{points:#x} points"),
            call_original,
            Regs {
                eax: points,
                esi: spline.addr() + 0x654,
                ..Regs::SENTINEL
            },
            &|| spline.fill(points + 9, points),
            &[spline.region()],
            &rust,
        );
    }
    t.finish()
}
