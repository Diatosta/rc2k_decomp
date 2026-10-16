//! `marks::find_spline_ends` (0x4BA4F1): record ebp's highest point of grade 0 and lowest
//! of grade 1.
//!
//! In:  ebp the spline record
//! Out: eax the second count, ecx the second found, edx its value, edi the last point read
//!      times 12
//!
//! See `marks/splines.rs`. Callers: `sub_4B9A32` (two). Reads its values 0x30 a point apart
//! (section 16), as the original. Refuses a negative start or count, or points past 0x80.
//! Self-test: the original and the Rust version on a record of the test's (with room for
//! those reads), the refusal.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::find_spline_ends", rva(0x4BA4F1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { find_spline_ends(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.ebp, SPLINE_LEN as usize)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let spline = TestSpline::new();
    for points in [0u32, 1, 5, 0x40, MOST_POINTS] {
        for seed in [1u32, 2, 3] {
            check_state(
                &mut t,
                &format!("{points:#x} points, seed {seed}"),
                call_original,
                Regs {
                    ebp: spline.addr(),
                    ..Regs::SENTINEL
                },
                &|| spline.fill(seed * 0x107 + points, points),
                &[spline.region()],
                &rust,
            );
        }
    }
    spline.fill(3, MOST_POINTS);
    unsafe { ((spline.addr() + 0x10) as *mut u32).write_unaligned(MOST_POINTS + 1) };
    t.check(
        unsafe {
            find_spline_ends(&Regs {
                ebp: spline.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "points past 0x80: the Rust version went on".into(),
    );
    t.finish()
}
