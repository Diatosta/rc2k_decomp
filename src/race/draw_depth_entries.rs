//! `race::draw_depth_entries` (0x454E84): each entry of `g_depthEntries` (in its order) drawn by
//! `race::draw_list_entry` of its second word (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: ecx the count, esi the last entry, ebx its word (none: kept) (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x455876), each view drawn. Refuses (before anything) an entry
//! of 0x4000000 or more. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose entry call goes to a stand-in that notes ebx, esi and ecx, against the Rust
//! version with the same; lists of 0, 1 and 5 entries; all put back.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::race::frame_draw::*;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_depth_entries", rva(0x454E84), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ENTRIES: u32 = 0x9D3730; // g_depthEntries

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done = unsafe {
        draw_entries(r, DEPTH_COUNT_RVA, ENTRIES, &mut |e| {
            draw_list_entry(e, &mut entry_part)
        })
    };
    match done {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLED: Mutex<Vec<(u32, u32, u32)>> = Mutex::new(Vec::new());

fn stand_in(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((r.ebx, r.esi, r.ecx));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x454E84,
            len: 0x27,
            entry: &[0x83, 0x3D, 0x4C, 0xDC, 0x63, 0x00, 0x00, 0x74],
            calls: &[0x18],
            jumps_out: &[],
        },
        &|c| (c == rva(0x454FF0)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let list = game_addr(rva(ENTRIES as usize)) as u32;
    let regions: Vec<Region> = vec![at(DEPTH_COUNT_RVA), (list, 0x40)];
    let saved = capture(&regions);
    for count in [0u32, 1, 5] {
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        let prepare = || unsafe {
            game_write(DEPTH_COUNT_RVA, count);
            for k in 0..5u32 {
                ((list + 8 * k) as *mut u32).write_unaligned(0x100 - k);
                ((list + 8 * k + 4) as *mut u32).write_unaligned(
                    [3u32, 0x100_0002, 0x200_0001, 0x300_0004, 0x8000_0007][k as usize],
                );
            }
            CALLED.lock().unwrap().clear();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (regs(&ro), std::mem::take(&mut *CALLED.lock().unwrap()));
        prepare();
        let mine = unsafe {
            draw_entries(&Regs::SENTINEL, DEPTH_COUNT_RVA, ENTRIES, &mut |e| {
                CALLED.lock().unwrap().push((e.ebx, e.esi, e.ecx));
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *CALLED.lock().unwrap()),
        );
        t.check(theirs == mine, || {
            format!("count {count}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
