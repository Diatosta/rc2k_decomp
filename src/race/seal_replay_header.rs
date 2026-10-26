//! `race::seal_replay_header` (0x41F0A3): the replay header's sealed part
//! (`dword_719B54`, 0x100 bytes): version 6, the player's name, the settings, a checksum
//! (also `dword_4F66F9`), then its 32 blocks chained through `race::seal_block` with the
//! key "halfmoon" (the game's key put back).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/replay.rs`. One caller, `race::open_replay`. Shadow mode: the original
//! alone (its result is the seal's, the cipher state changes). Self-test: a copy whose two
//! calls (the name, the seal) go to stand-ins, against the Rust version on the same, with
//! the part's dwords filled two ways.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::replay::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static ORIGINAL: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::seal_replay_header", rva(0x41F0A3), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { seal_replay_header(r, &mut game_part) } {
        Ok(g) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { seal_replay_header(r, &mut game_part) }?;
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
        0x41F0A3,
        0x1FA,
        &[0x60, 0xC6, 0x05, 0x58, 0x9B, 0x71, 0x00, 0x06],
        &[0x1A, 0x1CB],
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
    for fill in [0x00u8, 0xA5] {
        run_both(
            &mut t,
            &format!("filled {fill:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| {
                prepare(0, 0, 0, 0, 0);
                unsafe {
                    std::ptr::write_bytes(
                        crate::framework::game_addr(SEALED_RVA) as *mut u8,
                        fill,
                        0x100,
                    )
                };
            },
            &|r| unsafe { seal_replay_header(r, &mut test_part) },
            false,
        );
    }
    answers(false, false, false, 0x10000, 0x100, 0xFFFF_FF00);
    restore(&regions, &saved);
    t.finish()
}
