//! `marks::centre_spline_points` (0x4B9F95): record ebp's points made relative to its centre.
//!
//! In:  ebp the spline record
//! Out: eax, ebx, ecx the first point, edi 0
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (four). Refuses more than 0x80 points.
//! Self-test: the original and the Rust version on a record of the test's (0 .. 0x80
//! points), the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::centre_spline_points",
    rva(0x4B9F95),
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
    Ok(left_of(&unsafe { centre_spline_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.ebp, SPLINE_LEN as usize)];
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
    for points in [0u32, 1, 5, 0x40, MOST_POINTS] {
        for seed in [1u32, 2] {
            check_state(
                &mut t,
                &format!("{points:#x} points, seed {seed}"),
                call_original,
                Regs {
                    ebp: spline.addr(),
                    ..Regs::SENTINEL
                },
                &|| spline.fill(seed * 0x101 + points, points),
                &[spline.region()],
                &rust,
            );
        }
    }
    spline.fill(3, MOST_POINTS + 1);
    t.check(
        unsafe {
            centre_spline_points(&Regs {
                ebp: spline.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "0x81 points: the Rust version went on".into(),
    );
    t.finish()
}
