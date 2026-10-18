//! `race::sort_depth_entries` (0x454ED4): with two entries or more, `g_depthEntries` sorted by their
//! first dword with the game's quicksort (`sub_454EF7`, recursive, covered here: its
//! callers are this function and itself), then a `jmp` to a `nullsub` (a `retn`).
//!
//! In:  nothing
//! Out: the registers as the sort leaves them (a `call` follows)
//!
//! The sort is the original's step for step (`quick_sort` in `race/frame_draw.rs`): its
//! first scan moves on only past an entry above the pivot, and equal entries after a swap
//! move the left end on; the result keeps the original's order of equal keys. Caller:
//! `sub_4557AB`. Shadow mode: by state (the list). Self-test: the list filled from seeds
//! (keys with repeats, all equal, sorted, reversed), counts 0, 1, 2, 3, 100, 0x400 (all
//! equal there: 1023 calls deep in the original, a list of returns in Rust); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::sort_depth_entries", rva(0x454ED4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const ENTRIES: u32 = 0x9D3730; // g_depthEntries

fn regions() -> Vec<Region> {
    vec![(game_addr(rva(ENTRIES as usize)) as u32, 0x8000)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe {
        sort_entries(r, DEPTH_COUNT_RVA, ENTRIES)
    }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(DEPTH_COUNT_RVA));
    let saved = capture(&kept);
    let list = game_addr(rva(ENTRIES as usize)) as u32;
    for (count, kind) in [
        (0u32, 0u32),
        (1, 0),
        (2, 3),
        (3, 0),
        (100, 0),
        (100, 1),
        (100, 2),
        (100, 3),
        (0x400, 0),
        (0x400, 4),
        (0x400, 1),
    ] {
        check_state(
            &mut t,
            &format!("count {count} kind {kind}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(DEPTH_COUNT_RVA, count);
                let mut x = 0x1234_5678u32 ^ count;
                for k in 0..count {
                    x ^= x << 13;
                    x ^= x >> 17;
                    x ^= x << 5;
                    let key = match kind {
                        0 => x % 50,
                        1 => 7,
                        2 => k,
                        3 => count - k,
                        _ => x,
                    };
                    ((list + 8 * k) as *mut u32).write_unaligned(key);
                    ((list + 8 * k + 4) as *mut u32).write_unaligned(k);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
