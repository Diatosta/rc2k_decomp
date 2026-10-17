//! `physics::dot_product` (0x4212AE): esi . edi, x x' + (y y' + z z'), left in st(0)
//! unrounded. The same instructions as `scene::dot_product3` (0x46553C), whose Rust
//! version (`scene::vector3::dot_product3`) it uses.
//!
//! In:  esi, edi the two vectors (three floats)
//! Out: st(0) the product (80 bits), every register kept
//!
//! Thunk by `scene::vector3::result_hook!`. Callers (5): the cars' physics under
//! `sub_4454D2` (`sub_440B9D`, `sub_45EDB5`, `sub_45F17E` three times).
//! Self-test: the original and the Rust version on vectors by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RESULT: Result80 = Result80::new();

pub static HOOK: RawHook =
    RawHook::new("physics::dot_product", rva(0x4212AE), thunk, &ORIGINAL).self_test(self_test);

result_hook!(
    thunk,
    call_original,
    original_keeping_result,
    dispatch = dispatch,
    original = ORIGINAL,
    result = RESULT
);

/// The Rust version: the product as it is left in st(0).
pub unsafe fn dot_product(a: u32, b: u32) -> [u8; 10] {
    unsafe { dot_product3(a, b) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    RESULT.set(unsafe { dot_product(r.esi, r.edi) });
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, original_keeping_result, &[RESULT.region()], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut v = [0u32; 6];
    let p = v.as_mut_ptr() as u32;
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 1..=12u32 {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    original_keeping_result,
                    Regs {
                        esi: p,
                        edi: p + 12,
                        ..Regs::SENTINEL
                    },
                    &|| {
                        let mut x = [0u32; 6];
                        test_floats(seed * 5 + 1, &mut x);
                        // Signs mixed, so that the sums cancel now and then.
                        for (n, f) in x.iter_mut().enumerate() {
                            if (seed >> (n % 3)) & 1 != 0 {
                                *f ^= 0x8000_0000;
                            }
                        }
                        unsafe { (p as *mut [u32; 6]).write(x) };
                        RESULT.set([0xAA; 10]);
                    },
                    &[RESULT.region(), (p, 24)],
                    &rust,
                );
            }
        });
    }
    t.finish()
}
