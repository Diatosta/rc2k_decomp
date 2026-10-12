//! `math::random_below` (0x4018A2): a random number under a limit.
//!
//! In:  eax = the limit
//! Out: eax = `next_random` modulo the limit; everything else preserved
//! 73 callers. A limit of 0 makes the original's `div` trap: refused.
//! Manual patch with a private copy (0x12 bytes, its call relocated): the call starts in
//! the first five bytes.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::math::next_random::{STATE_RVA, TEST_STATES, next_random};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("math::random_below", rva(0x4018A2), thunk, &ORIGINAL)
    .private_copy(0x12)
    .relocate_calls(&[0x04])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn random_below(limit: u32) -> Result<u32, BadInput> {
    if limit == 0 {
        return Err(BadInput(
            "a random number under 0: the original divides by it",
        ));
    }
    Ok(unsafe { next_random() } % limit)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { random_below(r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(STATE_RVA)], &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(STATE_RVA)];
    let saved = capture(&regions);
    for state in TEST_STATES {
        for limit in [
            1u32,
            2,
            3,
            0xB4,
            0x100,
            0x7FFF_FFFF,
            0x8000_0000,
            0xFFFF_FFFF,
        ] {
            check_state(
                &mut t,
                &format!("state={state:#x} limit={limit:#x}"),
                call_original,
                Regs {
                    eax: limit,
                    ..Regs::SENTINEL
                },
                &|| unsafe { game_write(STATE_RVA, state) },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
