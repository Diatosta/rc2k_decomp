//! `race::copy_rally_times` (0x48C997): the rally stage's ten best times (0x9C-byte
//! records, by `dword_7558C8`, `dword_7558D0`, `dword_7558CC`) copied into the race's
//! (0x50-byte records at `dword_7556F4`), each logged.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/rally.rs`. One caller, `race::single_rally_race` (with ebx 3). Refuses,
//! before anything, a null pointer on the way. Shadow mode runs the original alone (it
//! logs). Self-test: a copy whose 50 log calls note themselves, against the Rust version
//! on the same, on the game's rally tables (skipped without them), the records filled.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::rally::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_rally_times", rva(0x48C997), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { copy_rally_times(r, &mut log_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

const CALLS: &[usize] = &[
    0x7E, 0x8B, 0x98, 0xA1, 0xAE, 0xB8, 0xC5, 0xCF, 0xD9, 0xE3, 0xED, 0xF7, 0x101, 0x10B, 0x115,
    0x11F, 0x12C, 0x139, 0x146, 0x153, 0x160, 0x16D, 0x17A, 0x187, 0x194, 0x1A1, 0x1AE, 0x1B8,
    0x1C1, 0x1CE, 0x1D8, 0x1E2, 0x1EC, 0x1F6, 0x200, 0x20A, 0x214, 0x21E, 0x228, 0x232, 0x23C,
    0x246, 0x250, 0x25A, 0x264, 0x26E, 0x27B, 0x288, 0x295, 0x2A2,
];
const PLACES: &[(usize, u32)] = &[
    (0x401427, WRITE),
    (0x40142B, WRITE_NL),
    (0x401472, INT),
    (0x401476, INT_NL),
];

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { rally_times_places() }.is_none() {
        return t.skip("no rally times");
    }
    let copy = match copy(
        0x48C997,
        0x2BA,
        &[0x60, 0xA1, 0xC8, 0x58, 0x75, 0x00],
        CALLS,
        PLACES,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = times_regions();
    let saved = capture(&regions);
    for fill in [0u8, 0x5A] {
        run_both(
            &mut t,
            &format!("filled {fill:#x}"),
            &copy,
            &regions,
            &|| {
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, fill, len) };
                }
            },
            &|r| unsafe { copy_rally_times(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
