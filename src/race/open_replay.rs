//! `race::open_replay` (0x41E6EC): with replays on and 30 MB free, the replay's header
//! made ("REPL", 0x50400, the date, "tmpreplay", the settings, the sealed part), the
//! index and block cleared, `var\\tmprpl` created and the header written; a failed open
//! or write closes it and sets carry. `dword_5BEC84` 0 first.
//!
//! In:  nothing
//! Out: every register preserved; carry set for the failure
//!
//! See `race/replay.rs`. Callers: `sub_443F0B`, `sub_443F96` (a `mov` follows; no
//! flags read). Shadow mode runs the original alone (it creates a file). Self-test: a
//! copy whose calls go to stand-ins (the free space, the time, the header's seal, the
//! open and the write that work or fail), against the Rust version on the same; replays
//! off each way.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::open_replay", rva(0x41E6EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { open_replay(r, &mut game_part) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { open_replay(r, &mut game_part) }?;
        *r = Regs { esp: r.esp, ..g }
    }
    Ok(())
}

#[allow(unused_variables)] // not every test sets every answer
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x41E6EC,
        0x16E,
        &[
            0x60, 0xC7, 0x05, 0x84, 0xEC, 0x5B, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        &[0x3F, 0x72, 0xD5, 0x12F, 0x15B, 0x166],
        None,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let answers = |w: bool, o: bool, c: bool, free: u32, packed: u32, chunk: u32| {
        FAIL_WRITE.store(w, Ordering::SeqCst);
        FAIL_OPEN.store(o, Ordering::SeqCst);
        FAIL_CHUNK.store(c, Ordering::SeqCst);
        FREE_KB.store(free, Ordering::SeqCst);
        PACKED.store(packed, Ordering::SeqCst);
        CHUNK.store(chunk, Ordering::SeqCst);
    };
    let _ = (AtomicBool::new(false), AtomicU32::new(0));
    let switches = [rva(0x5BEC78), rva(0x5BEC7C), rva(0x52A910), rva(0x75ACDC)];
    let kept: Vec<_> = switches
        .iter()
        .map(|&a| crate::service::harness::at(a))
        .collect();
    let saved_switches = capture(&kept);
    for (sw, free, fail_open, fail_write) in [
        ([1u32, 0u32, 0u32, 0u32], 0x10000u32, false, false),
        ([1, 0, 0, 0], 0x10000, true, false),
        ([1, 0, 0, 0], 0x10000, false, true),
        ([1, 0, 0, 0], 0x77FF, false, false),
        ([0, 0, 0, 0], 0x10000, false, false),
        ([1, 1, 0, 0], 0x10000, false, false),
        ([1, 0, 1, 0], 0x10000, false, false),
        ([1, 0, 0, 1], 0x10000, false, false),
    ] {
        run_both(
            &mut t,
            &format!(
                "switches {sw:?}, {free:#x} KB, open fails {fail_open}, write fails {fail_write}"
            ),
            &copy,
            Regs::SENTINEL,
            &|| {
                prepare(0, 0, 0, 0, 0);
                for (k, &a) in switches.iter().enumerate() {
                    unsafe { crate::framework::game_write(a, sw[k]) };
                }
                answers(fail_write, fail_open, false, free, 0x100, 0);
            },
            &|r| unsafe { open_replay(r, &mut test_part) },
            false,
        );
    }
    restore(&kept, &saved_switches);
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
