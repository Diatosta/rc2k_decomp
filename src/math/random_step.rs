//! `math::random_step` (0x4018C8): one step of the game's random numbers, ebx mixed in
//! (`math/next_random.rs`'s `random_step`).
//!
//! In:  eax the state, ebx what is mixed in
//! Out: eax the next; everything else preserved (edx pushed and popped)
//! Callers: `math::next_random` and 0x401870, 0x401892 (whose Rust versions use the
//! step themselves). Shadow mode: both, compared.
//! Self-test: the original against the Rust version on a set of states.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::random_step", rva(0x4018C8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, crate::math::next_random::random_step(r.eax, r.ebx)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut state = 0x40_0006u32;
    for n in 0..64u32 {
        check_state(
            &mut t,
            &format!("state {state:#x}, mixed {n:#x}"),
            call_original,
            Regs {
                eax: state,
                ebx: n.wrapping_mul(0x1234_5679),
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
        state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
    }
    t.finish()
}
