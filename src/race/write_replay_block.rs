//! `race::write_replay_block` (0x41E9A3): the replay's block (`dword_9613A4`, 0x2810
//! bytes) written by `race::write_replay_chunk`, its offset and size into the index
//! ("RCIB"), the file's length and the block count on; a failed write or the index full
//! (0x708 blocks) closes the replay (`race::close_replay`) and sets carry.
//!
//! In:  nothing
//! Out: every register preserved; carry set for the failure
//!
//! See `race/replay.rs`. Callers: `race::add_replay_frame` and `sub_41E930` (neither
//! reads its flags). Refuses an index full already (before anything; the original writes
//! past it). Shadow mode runs the original alone (it writes a file). Self-test: a copy
//! whose calls go to stand-ins (the chunk packed, plain, failing; the index filling up),
//! against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::write_replay_block", rva(0x41E9A3), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { write_replay_block(r, &mut game_part) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

#[allow(unused_variables)] // not every test sets every answer
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x41E9A3,
        0x5C,
        &[0x60, 0x8B, 0x15, 0xD0, 0xEC, 0x5B, 0x00],
        &[0x23, 0x54],
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
    for (blocks, chunk, fail) in [
        (3u32, 0xFFFF_F000u32, false),
        (3, 0x2810, false),
        (3, 0, true),
        (0x707, 0x100, false),
        (0, 0xFFFF_FFF0, false),
    ] {
        run_both(
            &mut t,
            &format!("{blocks:#x} blocks, chunk {chunk:#x}, failing {fail}"),
            &copy,
            Regs::SENTINEL,
            &|| {
                prepare(1, 0, blocks, 300, 100);
                answers(false, false, fail, 0x10000, 0x100, chunk);
            },
            &|r| unsafe { write_replay_block(r, &mut test_part) },
            true,
        );
    }
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
