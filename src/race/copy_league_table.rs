//! `race::copy_league_table` (0x4650B7): from the records at `dword_75ACD0` (ten of
//! 0x50; none when 0) each one's +8 into `dword_60410C` and its sixteen dwords from
//! +0x10 into its column of the sixteen rows at 0x604134 (0x28 apart).
//!
//! In:  nothing
//! Out: esi = past the records (0 without), ecx = 10, edx = 16, edi = past the last
//!      column, eax = the last dword; the rest preserved
//!
//! See `race/race_shell.rs`. One caller, `sub_446EBE` (`pusha` follows). Shadow mode:
//! the Rust version, the tables put back, the original's, compared. Self-test: records of
//! the test's and none, the game's tables put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::race_shell::*;
use crate::race::start_place::left_changed;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::copy_league_table", rva(0x4650B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    league_regions()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { copy_league_table(r) };
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

#[allow(unused_variables)] // not every test uses every helper
fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let _ = (
        game_addr(0),
        at(0),
        game_write::<u32> as unsafe fn(usize, u32),
    );
    let regions = regions();
    let source = at(rva(0x75ACD0));
    let mut kept = regions.clone();
    kept.push(source);
    let saved = capture(&kept);
    let records: Vec<u32> = (0..10 * 0x14u32)
        .map(|n| n.wrapping_mul(0x0100_0193))
        .collect();
    for with in [true, false] {
        check_state(
            &mut t,
            &format!("records {with}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0x5A, len);
                }
                game_write(
                    rva(0x75ACD0),
                    if with { records.as_ptr() as u32 } else { 0 },
                );
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
