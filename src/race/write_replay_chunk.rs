//! `race::write_replay_chunk` (0x41EEE4): ecx bytes at ebx written to file eax, packed
//! by runs (`race::pack_runs` into `g_pngInfo`, rounded up to a dword) when that is
//! smaller and `dword_5BEC80` is not 1 (eax then the bytes written negated).
//!
//! In:  eax = the file, ebx = the bytes, ecx = their count
//! Out: eax = the bytes written (negated when packed), carry set for a failed write;
//!      the rest preserved
//!
//! See `race/replay.rs`. Callers: `race::write_replay_block` and `sub_41ED1C` (each
//! `jb` after it: dispatch leaves the carry). With `dword_5BEC80` 1 the original jumps
//! to `files::write` (a tail call). Shadow mode runs the original alone (it writes a
//! file). Self-test: a copy whose calls and jump go to stand-ins (a write that works or
//! fails, packings smaller and not), against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::write_replay_chunk", rva(0x41EEE4), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { write_replay_chunk(r, &mut game_part) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { write_replay_chunk(r, &mut game_part) }?;
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
        0x41EEE4,
        0x43,
        &[0x83, 0x3D, 0x80, 0xEC, 0x5B, 0x00, 0x01],
        &[0x18, 0x29, 0x32],
        Some(0x7),
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
    for (plain, packed, fail) in [
        (0u32, 0x100u32, false),
        (0, 0x0FFE, false),
        (0, 0x2000, false),
        (0, 0x100, true),
        (1, 0, false),
        (1, 0, true),
    ] {
        run_both(
            &mut t,
            &format!("plain {plain}, packed {packed:#x}, failing {fail}"),
            &copy,
            Regs {
                eax: 0x5A5A_0001,
                ebx: 0x0040_0000,
                ecx: 0x2000,
                ..Regs::SENTINEL
            },
            &|| {
                prepare(1, plain, 3, 300, 50);
                answers(fail, false, false, 0x10000, packed, 0);
            },
            &|r| unsafe { write_replay_chunk(r, &mut test_part) },
            true,
        );
    }
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
