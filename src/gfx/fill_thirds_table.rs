//! `gfx::fill_thirds_table` (0x405580): the table of n / 3 for n = 0 .. 0x2FF.
//!
//! In:  nothing
//! Out: every register preserved
//! 0x300 bytes at `byte_4F4B30`: the mean of three components by their sum. One caller,
//! `start`. Who reads the table has not been looked up.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_ptr, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::fill_thirds_table", rva(0x405580), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLE_RVA: usize = rva(0x4F4B30); // byte_4F4B30
const TABLE_LEN: usize = 0x300;

// ---- Rust version ----

pub unsafe fn fill_thirds_table() {
    // Converted: `rc2k_core::gfx::pixels::thirds_table`.
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    unsafe { game_write(TABLE_RVA, table) };
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_ptr::<u8>(TABLE_RVA) as u32, TABLE_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { fill_thirds_table() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "the table",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &vec![0xAA; saved.len()]),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
