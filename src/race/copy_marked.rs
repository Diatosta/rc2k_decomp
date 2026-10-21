//! `race::copy_marked` (0x41E2F6): the marked blocks (from `dword_96E478` up to
//! `dword_96E47C`) onto the clipboard: their count into `dword_96E484`, their places
//! (`lDistanceToMove`) into `unk_96E488` and their cameras (`unk_963BB4`) into
//! `unk_971CC8`. Nothing when the marks are not in order (`jle`, signed).
//!
//! In:  nothing
//! Out: eax the first mark, ebx the second; copied: edx the count, ecx 0, esi and edi past
//!      the cameras' copy; no flags read
//!
//! Callers: `race::viewer_action` (action 0x1B, and in 0x11 before the cut). Refused, before
//! anything is written: a first mark below 0 or a second past 0x708 (the original copies
//! from outside the tables). Shadow mode compares. Self-test: marks in order, equal,
//! reversed, at the ends; the tables filled by the test, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    CAMERAS, CLIP_BLOCKS, CLIP_CAMERAS, CLIP_PLACES, MARK_IN, MARK_OUT, PLACES, Saved, TABLE_LEN,
    check_original, dispatch_regs, dword, fill, get, move_dwords, put, region,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_marked", rva(0x41E2F6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub fn copy_marked(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    // Converted: `rc2k_core::race::viewer::clip`.
    r.eax = get(MARK_IN);
    r.ebx = get(MARK_OUT);
    let Some((_, blocks)) = rc2k_core::race::viewer::clip(r.eax, r.ebx)? else {
        return Ok(r);
    };
    r.edx = blocks;
    put(CLIP_BLOCKS, r.edx);
    let dwords = 2 * r.edx as usize;
    move_dwords(CLIP_PLACES, PLACES + 8 * r.eax as usize, dwords);
    move_dwords(CLIP_CAMERAS, CAMERAS + 8 * r.eax as usize, dwords);
    r.ecx = 0;
    r.esi = a(CAMERAS + 8 * r.eax as usize) + 4 * dwords as u32;
    r.edi = a(CLIP_CAMERAS) + 4 * dwords as u32;
    Ok(r)
}

pub fn regions() -> Vec<Region> {
    vec![
        dword(CLIP_BLOCKS),
        region(CLIP_PLACES, TABLE_LEN),
        region(CLIP_CAMERAS, TABLE_LEN),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &regions(), 0, &copy_marked);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend([
        dword(MARK_IN),
        dword(MARK_OUT),
        region(PLACES, TABLE_LEN),
        region(CAMERAS, TABLE_LEN),
    ]);
    let saved = Saved::new(&all);
    fill(PLACES, TABLE_LEN, 3);
    fill(CAMERAS, TABLE_LEN, 4);
    for (first, second) in [
        (2u32, 9u32),
        (5, 5),
        (9, 2),
        (0, 0x708),
        (0x707, 0x708),
        (0xFFFF_FFFE, 1),
    ] {
        if (first as i32) < 0 {
            continue; // refused
        }
        check_original(
            &mut t,
            &format!("marks {first:#x} {second:#x}"),
            call_original,
            Regs::SENTINEL,
            &regions(),
            0,
            &|| {
                put(MARK_IN, first);
                put(MARK_OUT, second);
                put(CLIP_BLOCKS, 0xAAAA_AAAA);
                fill(CLIP_PLACES, TABLE_LEN, 5);
                fill(CLIP_CAMERAS, TABLE_LEN, 6);
            },
            &copy_marked,
        );
    }
    drop(saved);
    t.finish()
}
