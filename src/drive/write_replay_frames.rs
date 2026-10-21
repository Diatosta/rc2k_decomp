//! `drive::write_replay_frames` (0x41E930): while a replay is written to its file
//! (`dword_5BEC88` 1): `dword_5BECD4` + 100, the frame's head (`dword_9613A4`, `9613A8`)
//! 0, its byte +5 held to 2 ..= 3 (signed) unless `dword_719B34` is 1,
//! `race::write_replay_block` with `dword_5C252C` 100 round it, and `dword_9613A0` the
//! frame's address. All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::flush_replay`.
//! Refuses what `write_replay_block` refuses. Shadow mode runs the original alone (a file
//! write).
//! Self-test: the original and the Rust version with the replay not being written (its
//! writing is `race::write_replay_block`'s test).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::service::harness::{capture, restore};
use rc2k_core::drive::replay_frames::written_view;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "drive::write_replay_frames",
    rva(0x41E930),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::written_view`.
pub unsafe fn write_replay_frames(entry: &Regs) -> Result<(), Refused> {
    if g(0x5BEC88) != 1 {
        return Ok(());
    }
    p(0x5BECD4, g(0x5BECD4).wrapping_add(0x64));
    p(0x5C252C, 0x64);
    p(0x9613A4, 0);
    p(0x9613A8, 0);
    if g(0x719B34) != 1 {
        let at = a(0x9613A9) as *mut i8;
        let v = written_view(unsafe { at.read() });
        unsafe { at.write(v) };
    }
    unsafe { crate::race::replay::write_replay_block(entry, &mut crate::race::replay::game_part) }?;
    p(0x9613A0, a(0x9613A4));
    p(0x5C252C, 0);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { write_replay_frames(r) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x5BEC88), 4),
        (a(0x5BECD4), 4),
        (a(0x5C252C), 4),
        (a(0x9613A0), 12),
    ];
    let saved = capture(&all);
    p(0x5BEC88, 0);
    let before = capture(&all);
    let mut r = Regs::SENTINEL;
    unsafe { call_original(&mut r) };
    let theirs = capture(&all);
    restore(&all, &before);
    let ok = unsafe { write_replay_frames(&Regs::SENTINEL) }.is_ok() && capture(&all) == theirs;
    t.check(ok, || "not writing: something differs".into());
    restore(&all, &saved);
    t.finish()
}
