//! `drive::flush_replay` (0x46022D): the replay's starting state (0xA04 dwords at
//! `unk_7172EC`) copied into the frame `dword_9613A4` and the frames written
//! (`drive::write_replay_frames`).
//!
//! In:  nothing
//! Out: ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::flush_replay_if`.
//! Refuses what `write_replay_frames` refuses. Shadow mode runs the original alone (a
//! file write).
//! Self-test: the original and the Rust version with the replay not being written.

use crate::errors::Refused;
use crate::framework::{ECX, EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::flush_replay", rva(0x46022D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn flush_replay(entry: &Regs) -> Result<Left, Refused> {
    unsafe { std::ptr::copy(a(0x7172EC) as *const u32, a(0x9613A4) as *mut u32, DWORDS) };
    let left = Left::nothing()
        .with(ECX, 0)
        .with(ESI, a(0x7172EC) + 4 * DWORDS as u32)
        .with(EDI, a(0x9613A4) + 4 * DWORDS as u32);
    let mut r = *entry;
    left.apply(&mut r);
    unsafe { crate::drive::write_replay_frames::write_replay_frames(&r) }?;
    Ok(left)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { flush_replay(r) }
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = rust(r)?;
    left.apply(r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(0x9613A4), 4 * DWORDS),
        (a(0x5BEC88), 4),
        (a(0x7172EC), 4 * DWORDS),
    ];
    let saved = capture(&all);
    let set = || {
        p(0x5BEC88, 0);
        for k in 0..DWORDS {
            p(0x7172EC + 4 * k, k as u32 * 3);
        }
    };
    check_state(
        &mut t,
        "not writing",
        call_original,
        Regs::SENTINEL,
        &set,
        &all[..1],
        &rust,
    );
    restore(&all, &saved);
    t.finish()
}
