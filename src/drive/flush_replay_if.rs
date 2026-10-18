//! `drive::flush_replay_if` (0x460031): `drive::flush_replay` while recording
//! (`dword_5C5CE0` 1) at the last tenth (`dword_6EF7DC` 0x63).
//!
//! In:  nothing
//! Out: as `flush_replay` leaves the registers when called; no flags read
//!
//! Caller: `drive::replay_step`.
//! Refuses what `flush_replay` refuses. Shadow mode runs the original alone (a file
//! write).
//! Self-test: the original and the Rust version off the tenth and with the replay not
//! being written.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::flush_replay_if", rva(0x460031), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn flush_replay_if(entry: &Regs) -> Result<Left, Refused> {
    if g(0x5C5CE0) == 1 && g(0x6EF7DC) == 0x63 {
        return unsafe { crate::drive::flush_replay::flush_replay(entry) };
    }
    Ok(Left::nothing())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { flush_replay_if(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match rust(r) {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x9613A4), 0x2810),
        (a(0x5BEC88), 4),
        (a(0x5C5CE0), 4),
        (a(0x6EF7DC), 4),
    ];
    let saved = capture(&all);
    for case in 0..4u32 {
        let set = || {
            p(0x5BEC88, 0);
            p(0x5C5CE0, case % 2);
            p(0x6EF7DC, if case < 2 { 0x63 } else { 5 });
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all[..1],
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
