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
use crate::service::floats::*;
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
    unsafe { normalise_vector3_plain(r.esi) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.esi, 12)];
    let memory = [(regions[0], FLOAT)];
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut v = [0u32; 3];
    let p = v.as_mut_ptr() as u32;
    let tolerant = Floats {
        memory: &[((p, 12), FLOAT)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 0..12u32 {
            check_state_floats(
                &mut t,
                "scene::normalise_vector3",
                &format!("control word {word:#x}, seed {seed}"),
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
                floats,
                &rust,
            );
        }
    });
    t.finish()
}
