//! `marks::place_shape_points` (0x4BA0AE): see `marks/splines.rs`' `place_shape_points`.
//!
//! In:  ebp a spline record (`g_marksSplinesB` / `g_marksSplinesD`)
//! Out: see `place_shape_points`
//!
//! Callers: 0x4B9968, 0x4B9979 (`sub_4B98FD`). Refuses more than 0x80 points, an angle
//! `fsincos` cannot take, and a null point pointer, before anything. Self-test: a record
//! of the test's (`TestSpline`) whose pointers name a buffer of the test's, several point
//! counts and seeds; the work variables, the record and the buffer compared.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::splines::{
    MOST_POINTS, SPLINE_LEN, TestSpline, place_shape_points, work_regions,
};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::place_shape_points", rva(0x4BA0AE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COUNT: u32 = 0x50;
const POINTERS: u32 = 0x454;

/// What it may write: the work variables, the record, and the points' targets.
pub fn regions(spline: u32) -> Vec<Region> {
    let mut r = work_regions();
    r.push((spline, SPLINE_LEN as usize));
    let n =
        unsafe { ((spline + COUNT) as *const i32).read_unaligned() }.clamp(0, MOST_POINTS as i32);
    for k in 0..n as u32 {
        let p = unsafe { ((spline + POINTERS + 4 * k) as *const u32).read_unaligned() };
        if p != 0 {
            r.push((p, 12));
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_shape_points(r) }?))
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
    let targets = vec![0u32; 3 * MOST_POINTS as usize];
    let base = targets.as_ptr() as u32;
    let mut regions = work_regions();
    regions.push((spline.addr(), SPLINE_LEN as usize));
    regions.push((base, 4 * targets.len()));
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
                &|| unsafe {
                    spline.fill(seed * 0x115 + points, points);
                    // Each pointer names a slot of the test's buffer (in a shuffled order).
                    for k in 0..MOST_POINTS {
                        let slot = (k * 37 + seed) % MOST_POINTS;
                        ((spline.addr() + POINTERS + 4 * k) as *mut u32)
                            .write_unaligned(base + 12 * slot);
                    }
                    std::ptr::write_bytes(base as *mut u8, 0xAA, 4 * targets.len());
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
    spline.fill(3, 4);
    unsafe { ((spline.addr() + POINTERS + 4) as *mut u32).write_unaligned(0) };
    t.check(unsafe { place_shape_points(&r) }.is_err(), || {
        "a null point pointer: the Rust version went on".into()
    });
    restore(&regions, &saved);
    drop(targets);
    t.finish()
}
