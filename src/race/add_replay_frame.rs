//! `race::add_replay_frame` (0x41E8DE): with the replay open, ecx bytes at ebx (a frame
//! when 0x10: counted) put at the block's cursor; at 100 frames the block written
//! (`race::write_replay_block`), the cursor and count back to the start.
//!
//! In:  ebx = the bytes, ecx = their count
//! Out: every register preserved
//!
//! See `race/replay.rs`. One caller in code (`race::close_replay`, its padding). Refuses
//! bytes past the block's room (before anything; the original writes on). Shadow mode
//! runs the original alone (it may write a file). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version on the same: closed, a frame, other sizes, the
//! hundredth frame.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::add_replay_frame", rva(0x41E8DE), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { add_replay_frame(r, &mut game_part) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { add_replay_frame(r, &mut game_part) }?;
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
        0x41E8DE,
        0x52,
        &[0x60, 0x83, 0x3D, 0x88, 0xEC, 0x5B, 0x00, 0x01],
        &[0x37],
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
    let frame: Vec<u32> = (0..0x40).map(|n| 0x0101_0101u32.wrapping_mul(n)).collect();
    for (open, count, in_block) in [
        (0u32, 0x10u32, 5u32),
        (1, 0x10, 5),
        (1, 0x10, 99),
        (1, 0x20, 99),
        (1, 0x7, 3),
        (1, 0x10, 150),
    ] {
        run_both(
            &mut t,
            &format!("open {open}, {count:#x} bytes, {in_block} frames"),
            &copy,
            Regs {
                ebx: frame.as_ptr() as u32,
                ecx: count,
                ..Regs::SENTINEL
            },
            &|| prepare(open, 0, 3, 300, in_block),
            &|r| unsafe { add_replay_frame(r, &mut test_part) },
            false,
        );
    }
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
