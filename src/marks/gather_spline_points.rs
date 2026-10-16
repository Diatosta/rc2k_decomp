//! `marks::gather_spline_points` (0x4B9D5F): record ebp filled with the points of object
//! eax's faces of kind ebx.
//!
//! In:  eax the object, ebx the face kind, ebp the spline record
//! Out: eax the count, esi +0x654; ebx, ecx, edi as the last loop leaves them (see
//!      `marks/splines.rs`)
//!
//! Callers: `sub_4B9A32` (four). Refuses an object past 0xFF, a negative face count or
//! faces past `g_faces`. Self-test: the original and the Rust version on the scene of
//! `marks::measure_lines` (two objects of 0x20 faces of kinds 0 .. 3 by lot), the refusal;
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::lines::{TEST_OBJECTS, TestScene};
use crate::marks::splines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::gather_spline_points",
    rva(0x4B9D5F),
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
    vec![(spline, SPLINE_LEN as usize), at(FOUND_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { gather_spline_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r.ebp);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let spline = TestSpline::new();
    let regions = regions(spline.addr());
    let saved = capture(&regions[1..]);
    for object in TEST_OBJECTS {
        for kind in 0..4u32 {
            for seed in [1u32, 2] {
                check_state(
                    &mut t,
                    &format!("object {object:#x}, kind {kind}, seed {seed}"),
                    call_original,
                    Regs {
                        eax: object,
                        ebx: kind,
                        ebp: spline.addr(),
                        ..Regs::SENTINEL
                    },
                    &|| {
                        scene.fill(seed * 5 + kind, 1);
                        spline.fill(seed + kind, 0x10);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    t.check(
        unsafe {
            gather_spline_points(&Regs {
                eax: 0x100,
                ebp: spline.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "object 0x100: the Rust version went on".into(),
    );
    restore(&regions[1..], &saved);
    drop(scene);
    t.finish()
}
