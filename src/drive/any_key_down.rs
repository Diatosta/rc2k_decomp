//! `drive::any_key_down` (0x444F55): whether any of the 0x80 key bytes (`g_keysPressed`)
//! is set: their or into eax, CF set when not 0.
//!
//! In:  nothing
//! Out: CF; eax the bytes' or, ecx 0x80, esi `g_keysPressed`; the rest kept
//!
//! Caller: `drive::race_frame` (`sub_444D46`, `jb`).
//! Self-test: the original and the Rust version on the key bytes (saved and put back):
//! none, one, several.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::any_key_down", rva(0x444F55), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KEYS: usize = 0x4F6CDC; // g_keysPressed

/// The Rust version: (the bytes' or, esi).
pub unsafe fn any_key_down() -> (u32, u32) {
    let at = a(KEYS);
    let or = (0..0x80).fold(0u8, |o, k| o | unsafe { ((at + k) as *const u8).read() });
    (or as u32, at)
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    let (or, esi) = unsafe { any_key_down() };
    Ok(Left::carry(or != 0)
        .with(EAX, or)
        .with(ECX, 0x80)
        .with(ESI, esi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [(a(KEYS), 0x80)];
    let saved = capture(&all);
    for case in 0..5u32 {
        let set = || unsafe {
            std::ptr::write_bytes(a(KEYS) as *mut u8, 0, 0x80);
            for k in 0..case {
                ((a(KEYS) + 0x1F * k + 3) as *mut u8).write((0x11 * (k + 1)) as u8);
            }
        };
        check_state(
            &mut t,
            &format!("{case} keys"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
