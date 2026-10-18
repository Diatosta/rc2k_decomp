//! `math::count_bits` (0x401854): the number of bits set in eax.
//!
//! In:  eax = the value
//! Out: eax = the count, ZF set (the loop's last `test`); the rest kept
//!
//! Seven callers (`screens::classify_controller` four times). Shadow mode: by state.
//! Self-test: values of none, one and many bits.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::axes::count_bits;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::count_bits", rva(0x401854), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    // No caller reads the flags (each goes on with `mov`, `cmp` or `test`).
    Ok(Left::nothing().with(EAX, count_bits(r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for v in [0u32, 1, 0x8000_0000, 0xFFFF_FFFF, 0x1234_5678] {
        check_state(
            &mut t,
            &format!("{v:#x}"),
            call_original,
            Regs {
                eax: v,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
