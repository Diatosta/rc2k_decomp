//! `physics::vector_length` (0x421282): the length of the three floats at esi, left in
//! st(0): `fsqrt` of x x + (y y + z z), not rounded to a float.
//!
//! In:  esi the vector
//! Out: st(0) the length (80 bits), every register kept
//!
//! Thunk by `scene::vector3::result_hook!`. Callers (17): the cars' physics under
//! `sub_4454D2` (`sub_43B635`, `sub_43BD2D`, `sub_43DD54`, `sub_43F7AC`, `sub_440B9D`,
//! `sub_437A53`, `sub_45D8FB`, `sub_45E3BF`, `sub_45E800`, `sub_45F17E`, `sub_45F7B1`,
//! `sub_462485`, and three in unowned code at 0x43AD63 ..).
//! Self-test: the original and the Rust version on vectors by lot, under three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, rva};
use crate::math::x87_run::length_80;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::test_floats;
use crate::scene::vector3::*;
use crate::screens::harness::Left;
use crate::service::floats::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static RESULT: Result80 = Result80::new();

pub static HOOK: RawHook =
    RawHook::new("physics::vector_length", rva(0x421282), thunk, &ORIGINAL).self_test(self_test);

result_hook!(
    thunk,
    call_original,
    original_keeping_result,
    dispatch = dispatch,
    original = ORIGINAL,
    result = RESULT
);

/// The Rust version: the length as it is left in st(0).
pub unsafe fn vector_length(v: u32) -> [u8; 10] {
    unsafe { length_80(v as *const u32) }
}

/// The converted version (`rc2k_core::math::vector`, plain floats): the hook's.
pub unsafe fn vector_length_plain(v: u32) -> [u8; 10] {
    let length = rc2k_core::math::vector::length(unsafe { read_floats(v) });
    rc2k_core::math::extended::from_f64(length)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    RESULT.set(unsafe { vector_length_plain(r.esi) });
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
    let mut v = [0u32; 3];
    let p = v.as_mut_ptr() as u32;
    let tolerant = Floats {
        memory: &[(RESULT.region(), EXTENDED)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for seed in 1..=12u32 {
            check_state_floats(
                &mut t,
                "physics::vector_length",
                &format!("control word {word:#x}, seed {seed}"),
                original_keeping_result,
                Regs {
                    esi: p,
                    ..Regs::SENTINEL
                },
                &|| {
                    let mut x = [0u32; 3];
                    test_floats(seed, &mut x);
                    unsafe { (p as *mut [u32; 3]).write(x) };
                    RESULT.set([0xAA; 10]);
                },
                &[RESULT.region(), (p, 12)],
                floats,
                &rust,
            );
        }
    });
    t.finish()
}
