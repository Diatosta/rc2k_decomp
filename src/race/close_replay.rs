//! `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
//! padding frame (`race::add_replay_frame`), the last block's frame count, the time
//! sealed, the index ("RCIB") written, the file closed; `dword_5BEC88` 0,
//! `dword_5BEC84` 1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/replay.rs`. Callers: `race::open_replay`, `race::write_replay_block`
//! (their failures), `sub_443F0B`, `sub_443F96` (none reads its flags). Shadow mode
//! runs the original alone (it writes a file). Self-test: a copy whose calls go to
//! stand-ins, against the Rust version on the same: closed, open with frames and
//! without, the last block or not.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::close_replay", rva(0x41E85A), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { close_replay(r, &mut game_part) } {
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
        0x41E85A,
        0x84,
        &[0x60, 0x83, 0x3D, 0x88, 0xEC, 0x5B, 0x00, 0x01],
        &[0x1E, 0x4A, 0x5F, 0x69],
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
    for (open, blocks, frames, in_block) in [
        (0u32, 3u32, 300u32, 5u32),
        (1, 3, 300, 5),
        (1, 3, 250, 0),
        (1, 5, 1234, 0),
        (1, 1, 0, 7),
    ] {
        run_both(
            &mut t,
            &format!("open {open}, {blocks} blocks, {frames} frames, {in_block} in the block"),
            &copy,
            Regs::SENTINEL,
            &|| {
                prepare(open, 0, blocks, frames, in_block);
                if frames == 250 {
                    unsafe { crate::framework::game_write(LAST_BLOCK_RVA, 0u32) };
                }
            },
            &|r| unsafe { close_replay(r, &mut test_part) },
            false,
        );
    }
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
