//! `physics::multiply_parts` (0x420FD5): the three floats at ebx times those at ecx, part
//! by part, into edx (`fld; fmul; fstp`, each stored before the next is read).
//!
//! In:  ebx, ecx the two vectors, edx where the result goes (may be either)
//! Out: every register kept; no flags read
//!
//! Caller: `sub_45F17E` (0x45F256).
//! Self-test: the original on vectors by lot, apart and in the place of each, under
//! three precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::multiply_parts", rva(0x420FD5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn multiply_parts(a: u32, b: u32, out: u32) {
    unsafe {
        for k in 0..3 {
            let x = ((a + 4 * k) as *const u32).read_unaligned();
            let y = ((b + 4 * k) as *const u32).read_unaligned();
            ((out + 4 * k) as *mut u32).write_unaligned(x87::mul(x, y));
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { multiply_parts(r.ebx, r.ecx, r.edx) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.edx, 12)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let work: Cell<[u32; 9]> = Cell::new([0; 9]);
    let base = work.as_ptr() as u32;
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for case in 0..12u32 {
                let out = base + 12 * [2u32, 0, 1][(case % 3) as usize];
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, case {case}"),
                    call_original,
                    Regs {
                        ebx: base,
                        ecx: base + 12,
                        edx: out,
                        ..Regs::SENTINEL
                    },
                    &|| {
                        let mut f = [0u32; 9];
                        test_floats(case * 5 + 2, &mut f);
                        work.set(f);
                    },
                    &[(base, 36)],
                    &rust,
                );
            }
        });
    }
    t.finish()
}
