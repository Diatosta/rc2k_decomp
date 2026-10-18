//! `scene::noise_step` (0x440F4E): one step of the scene's noise (`scene/next_noise.rs`).
//!
//! In:  eax the state
//! Out: eax the next; everything else preserved (edx pushed and popped)
//! Callers: 0x440F2E and `scene::next_noise` (0x440F3E), whose Rust versions use the
//! step themselves. Shadow mode: both, compared.
//! Self-test: the original against the Rust version on a set of states.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::noise_step", rva(0x440F4E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, crate::scene::next_noise::noise_step(r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut state = 1u32;
    for n in 0..64u32 {
        check_state(
            &mut t,
            &format!("state {state:#x}"),
            call_original,
            Regs {
                eax: state,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
        state = state.wrapping_mul(0x9E37_79B9).wrapping_add(n) ^ (n << 22);
    }
    t.finish()
}
