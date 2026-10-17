//! `marks::turn_shape_points` (0x4BD5ED): the shape set's two lists of points turned (a
//! tilt about X, then ebx and ecx about Z) and, offset, written to their targets.
//!
//! In:  ebx, ecx the two turns (floats)
//! Out: eax the set, the rest as the last list leaves them
//!
//! See `marks/shape_points.rs`. Callers: `sub_4BD122`, `sub_4BEB25` (twice). Refuses a list
//! of more than 0x40 or an angle `fsincos` cannot take, before writing. Self-test: the
//! original and the Rust version on sets, turns and targets by lot, the refusal; put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::turn_shape_points", rva(0x4BD5ED), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { turn_shape_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = turn_regions();
    regions.extend(unsafe { target_regions() });
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = set_inputs();
    let mut regions = turn_regions();
    let saved = (capture(&inputs), capture(&regions));
    let targets = vec![0xAAu8; 12 * 2 * MOST_POINTS as usize];
    let tp = targets.as_ptr() as u32;
    regions.push((tp, targets.len()));
    for (a, b) in [(0u32, 0u32), (1, 3), (0x10, 7), (MOST_POINTS, MOST_POINTS)] {
        for seed in [1u32, 2] {
            let mut v = [0u32; 2];
            crate::scene::matrix3::test_floats(seed + a, &mut v);
            check_state(
                &mut t,
                &format!("{a:#x} and {b:#x} points, seed {seed}"),
                call_original,
                Regs {
                    ebx: v[0],
                    ecx: v[1],
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(tp as *mut u8, 0xAA, targets.len());
                    fill_set(seed * 7 + a, [a, b], tp);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { fill_set(3, [MOST_POINTS + 1, 0], tp) };
    t.check(
        unsafe { turn_shape_points(&Regs::SENTINEL) }.is_err(),
        || "0x41 points: the Rust version went on".into(),
    );
    regions.pop();
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
