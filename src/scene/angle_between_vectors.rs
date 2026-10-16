//! `scene::angle_between_vectors` (0x4658CF): the angle between the vectors at esi and edi
//! (both normalised in place), left in st(0).
//!
//! In:  esi, edi the two vectors (three floats)
//! Out: st(0) the angle (a float's value), every register kept
//!
//! See `scene/vector3.rs`. Thunk by `result_hook!`. Callers: `sub_4B9E7D`
//! (`marks::bound_spline_points`), `sub_4C4CEC`, `sub_4C7119`. Self-test: the original and
//! the Rust version on vectors by lot (the same, opposite, a zero one among them); the
//! status variable put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RESULT: Result80 = Result80::new();

pub static HOOK: RawHook = RawHook::new(
    "scene::angle_between_vectors",
    rva(0x4658CF),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

result_hook!(
    thunk,
    call_original,
    original_keeping_result,
    dispatch = dispatch,
    original = ORIGINAL,
    result = RESULT
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    RESULT.set(unsafe { angle_between_vectors(r.esi, r.edi) });
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [RESULT.region(), (r.esi, 12), (r.edi, 12), at(STATUS_RVA)];
    dispatch_state(&HOOK, r, original_keeping_result, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let status = [at(STATUS_RVA)];
    let saved = capture(&status);
    let mut v = [0u32; 6];
    let p = v.as_mut_ptr() as u32;
    for seed in 0..14u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            original_keeping_result,
            Regs {
                esi: p,
                edi: p + 12,
                ..Regs::SENTINEL
            },
            &|| {
                let mut x = [0u32; 6];
                test_floats(seed + 1, &mut x);
                match seed {
                    0 => x.copy_within(0..3, 3),
                    1 => {
                        for n in 0..3 {
                            x[3 + n] = x[n] ^ 0x8000_0000;
                        }
                    }
                    2 => x[3..].fill(0),
                    _ => {}
                }
                unsafe { (p as *mut [u32; 6]).write(x) };
                RESULT.set([0xAA; 10]);
            },
            &[RESULT.region(), (p, 24), status[0]],
            &rust,
        );
    }
    restore(&status, &saved);
    t.finish()
}
