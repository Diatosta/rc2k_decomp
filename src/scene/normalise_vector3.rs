//! `scene::normalise_vector3` (0x4654EA): the three floats at esi made one long.
//!
//! In:  esi the vector
//! Out: every register kept
//!
//! See `scene/vector3.rs`. 37 callers. Each part times 1 over the length, the original's
//! instructions (a zero vector gives NaNs, as there). Self-test: the original and the
//! Rust version on vectors by lot, zero among them.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::normalise_vector3", rva(0x4654EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { normalise_vector3(r.esi) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 12)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut v = [0u32; 3];
    let p = v.as_mut_ptr() as u32;
    for seed in 0..12u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                esi: p,
                ..Regs::SENTINEL
            },
            &|| {
                let mut x = [0u32; 3];
                if seed != 0 {
                    test_floats(seed, &mut x);
                }
                unsafe { (p as *mut [u32; 3]).write(x) };
            },
            &[(p, 12)],
            &rust,
        );
    }
    t.finish()
}
