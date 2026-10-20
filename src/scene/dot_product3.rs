//! `scene::dot_product3` (0x46553C): esi . edi, left in st(0).
//!
//! In:  esi, edi the two vectors (three floats)
//! Out: st(0) the product (unrounded), every register kept
//!
//! See `scene/vector3.rs`. Thunk by `result_hook!`. Callers: `scene::angle_between_vectors`
//! and two more. Self-test: the original and the Rust version on vectors by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RESULT: Result80 = Result80::new();

pub static HOOK: RawHook =
    RawHook::new("scene::dot_product3", rva(0x46553C), thunk, &ORIGINAL).self_test(self_test);

result_hook!(
    thunk,
    call_original,
    original_keeping_result,
    dispatch = dispatch,
    original = ORIGINAL,
    result = RESULT
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    RESULT.set(unsafe { dot_product3_plain(r.esi, r.edi) });
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(RESULT.region(), EXTENDED)],
        regs: &[],
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_keeping_result,
        &[RESULT.region()],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut v = [0u32; 6];
    let p = v.as_mut_ptr() as u32;
    let tolerant = Floats {
        memory: &[(RESULT.region(), EXTENDED)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 1..=12u32 {
            check_state_floats(
                &mut t,
                "scene::dot_product3",
                &format!("control word {word:#x}, seed {seed}"),
                original_keeping_result,
                Regs {
                    esi: p,
                    edi: p + 12,
                    ..Regs::SENTINEL
                },
                &|| {
                    let mut x = [0u32; 6];
                    test_floats(seed, &mut x);
                    unsafe { (p as *mut [u32; 6]).write(x) };
                    RESULT.set([0xAA; 10]);
                },
                &[RESULT.region(), (p, 24)],
                floats,
                &rust,
            );
        }
    });
    t.finish()
}
