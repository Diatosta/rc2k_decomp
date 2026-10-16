//! `scene::multiply_matrices3` (0x4657EC): ebp = esi x edi, 3 x 3 matrices of floats.
//!
//! In:  esi, edi the two, ebp the result
//! Out: ecx and edx 0, edi and ebp 12 on, esi kept
//!
//! See `scene/matrix3.rs`. Callers: `sub_4B9FE6` (`marks::turn_spline_points`),
//! `sub_4BA0AE`, `sub_4BA16C`, `sub_4BD5ED` and code outside a function. Self-test: the
//! original and the Rust version on matrices by lot (the result apart, and on the first).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::multiply_matrices3", rva(0x4657EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { multiply_matrices3(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.ebp, 0x24)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = [0u32; 27];
    let b = buffer.as_mut_ptr() as u32;
    for seed in 1..=4u32 {
        for (what, out) in [("apart", b + 0x48), ("on the first", b)] {
            check_state(
                &mut t,
                &format!("seed {seed}, the result {what}"),
                call_original,
                Regs {
                    esi: b,
                    edi: b + 0x24,
                    ebp: out,
                    ..Regs::SENTINEL
                },
                &|| {
                    let mut v = [0u32; 27];
                    test_floats(seed, &mut v);
                    unsafe { std::ptr::copy(v.as_ptr(), b as *mut u32, 27) };
                },
                &[(b, 0x6C)],
                &rust,
            );
        }
    }
    t.finish()
}
