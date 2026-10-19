//! `race::mark_viewer_items` (0x41D456): the replay viewer's items marked for what can be
//! done now: the marks cleared (`menu::list_clear_marks`), then bit 6 of item 0x14 when
//! the replay is the last race's (`dword_5C2544`), bit 7 (not usable) of: 0xD .. 0x10 and
//! 0x17 .. 0x19 at the last block; 0x11 when the marks are not in order; 0x12 with an
//! empty clipboard or one too big to paste (count + clipboard past 0x708); 0x13 with
//! nothing to undo; 0xA at or before the first mark, 2 at or past the second; with no
//! blocks 0 .. 0x11 and 0x15 .. 0x19; 0x18, 0x19 when the block's camera byte 5 is 0, 0xD,
//! 0xE when it is 1; 0xF for one player's replay (`dword_719B34`).
//!
//! In:  edi the list (`unk_5C255C`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer` (two). Shadow mode compares. Self-test: the viewer's own
//! list (saved and put back), the replay's numbers set eleven ways, against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERAS, CLIP_BLOCKS, LAST_RACE_VIEWED, MARK_IN, MARK_OUT, MENU, ONE_PLAYER,
    Saved, UNDO_BLOCKS, check_original, dispatch_regs, dword, gb, get, put, region,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mark_viewer_items", rva(0x41D456), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub unsafe fn mark_viewer_items(list: u32) {
    unsafe { crate::menu::list_clear_marks::list_clear_marks(list) };
    let mark = |item: u32, bit: u8| unsafe {
        let at = (list + 0x18 + 0x10 * item) as *mut u8;
        at.write(at.read() | bit);
    };
    let block = get(BLOCK);
    let blocks = get(BLOCKS);
    if get(LAST_RACE_VIEWED) == 1 {
        mark(0x14, 0x40);
    }
    if block == blocks {
        for item in [0xF, 0x10, 0xD, 0xE, 0x19, 0x18, 0x17] {
            mark(item, 0x80);
        }
    }
    if (get(MARK_OUT) as i32) <= (get(MARK_IN) as i32) {
        mark(0x11, 0x80);
    }
    let clip = get(CLIP_BLOCKS);
    if clip == 0 || blocks.wrapping_add(clip) > 0x708 {
        mark(0x12, 0x80);
    }
    if get(UNDO_BLOCKS) == 0 {
        mark(0x13, 0x80);
    }
    if block <= get(MARK_IN) {
        mark(0xA, 0x80);
    }
    if block >= get(MARK_OUT) {
        mark(2, 0x80);
    }
    if blocks == 0 {
        for item in (0..=0x11).chain(0x15..=0x19) {
            mark(item, 0x80);
        }
    }
    let camera = gb((CAMERAS + 5).wrapping_add(block.wrapping_mul(8) as usize));
    if camera == 0 {
        mark(0x18, 0x80);
        mark(0x19, 0x80);
    }
    if camera == 1 {
        mark(0xE, 0x80);
        mark(0xD, 0x80);
    }
    if get(ONE_PLAYER) == 1 {
        mark(0xF, 0x80);
    }
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { mark_viewer_items(r.edi) };
    Ok(*r)
}

/// The list's bytes (26 items at least) and its choices.
unsafe fn regions_of(list: u32) -> Vec<Region> {
    let mut all = unsafe { crate::menu::list_clear_marks::regions_of(list) };
    all.push((list, 0x18 + 0x10 * 0x1A));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = unsafe { regions_of(r.edi) };
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let list = crate::physics::flow::a(MENU);
    if gb(MENU + 1) == 0 {
        return t.skip("the viewer's list has no items");
    }
    let regions = unsafe { regions_of(list) };
    let mut all = regions.clone();
    all.extend([
        dword(BLOCK),
        dword(BLOCKS),
        dword(MARK_IN),
        dword(MARK_OUT),
        dword(CLIP_BLOCKS),
        dword(UNDO_BLOCKS),
        dword(LAST_RACE_VIEWED),
        dword(ONE_PLAYER),
        region(CAMERAS, 0x80),
    ]);
    let saved = Saved::new(&all);
    let first = crate::service::harness::capture(&regions);
    // (block, blocks, marks, clipboard, undo, last race, camera byte 5, one player)
    for (block, blocks, (m_in, m_out), clip, undo, last, camera, one) in [
        (3u32, 9u32, (1u32, 6u32), 2u32, 0u32, 1u32, 2u8, 0u32),
        (9, 9, (0, 9), 0, 4, 0, 0, 1),
        (0, 0, (0, 0), 0, 0, 0, 1, 0),
        (2, 9, (2, 2), 0x700, 1, 1, 1, 1),
        (6, 9, (7, 3), 1, 0, 0, 3, 0),
        (5, 0x708, (0, 0x708), 1, 2, 0, 0, 0),
        (1, 9, (0xFFFF_FFFF, 1), 3, 0, 0, 2, 0),
    ] {
        let prepare = || {
            crate::service::harness::restore(&regions, &first);
            put(BLOCK, block);
            put(BLOCKS, blocks);
            put(MARK_IN, m_in);
            put(MARK_OUT, m_out);
            put(CLIP_BLOCKS, clip);
            put(UNDO_BLOCKS, undo);
            put(LAST_RACE_VIEWED, last);
            put(ONE_PLAYER, one);
            crate::race::viewer::pb(CAMERAS + 8 * block as usize + 5, camera);
        };
        check_original(
            &mut t,
            &format!("block {block} of {blocks}, marks {m_in:#x} {m_out:#x}, clipboard {clip:#x}"),
            call_original,
            Regs {
                edi: list,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &prepare,
            &rust,
        );
    }
    drop(saved);
    t.finish()
}
