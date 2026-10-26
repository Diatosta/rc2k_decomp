//! `race::add_sort_entry` (0x454D57): appends an entry to `dword_9DB730` (0x1000 of 8
//! bytes, the count `dword_63DC50`): +4 ebx; +0 (eax << 16) | bx when (ebx & 0x7FFFFFFF)
//! is below 0x1000000, else 0.
//!
//! In:  eax (a kind byte), ebx (a key)
//! Out: edx the entry, ebx & 0x7FFFFFFF, eax as written (when written), CF clear (`clc`;
//!      its callers in `sub_454C99` jump on and pop ebx: not read)
//!
//! See `race/frame_draw.rs`. A reading: the faces to draw, sorted later by key. Refuses
//! (before anything) an entry past the 0x1000. Shadow mode: by state (the entry and the
//! count). Self-test: counts 0, 5, 0xFFF, keys below and above 0x1000000 and with the top
//! bit; all put back; 0x1000 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::add_sort_entry", rva(0x454D57), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn entry_region() -> Vec<Region> {
    let n: u32 = unsafe { game_read(SORT_COUNT_RVA) };
    let mut v = vec![at(SORT_COUNT_RVA)];
    if n < SORT_MAX {
        v.push((game_addr(SORT_ENTRIES_RVA) as u32 + 8 * n, 8));
    }
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_sort_entry(r.eax, r.ebx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &entry_region(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = vec![
        at(SORT_COUNT_RVA),
        (game_addr(SORT_ENTRIES_RVA) as u32, 8 * SORT_MAX as usize),
    ];
    let saved = capture(&regions);
    for (count, eax, ebx) in [
        (0u32, 0x12u32, 0x1234u32),
        (5, 0xFF, 0x00FF_FFFF),
        (0xFFF, 0x1_0007, 0x0100_0000),
        (7, 3, 0x8000_0042),
        (9, 3, 0xFFFF_FFFF),
    ] {
        check_state(
            &mut t,
            &format!("count {count:#x} eax {eax:#x} ebx {ebx:#x}"),
            call_original,
            Regs {
                eax,
                ebx,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                game_write(SORT_COUNT_RVA, count);
                let at = game_addr(SORT_ENTRIES_RVA) as u32 + 8 * count;
                std::ptr::write_bytes(at as *mut u8, 0xAA, 8);
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(SORT_COUNT_RVA, SORT_MAX);
        add_sort_entry(0, 0).is_err()
    };
    restore(&regions, &saved);
    t.check(refused, || "entry 0x1000 not refused".into());
    t.finish()
}
