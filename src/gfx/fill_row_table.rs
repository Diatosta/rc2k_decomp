//! `gfx::fill_row_table` (0x40586A): fills the table of row addresses with a start address
//! and so many steps on from it.
//!
//! In:  eax = the first row's address, ebx = the bytes from one row to the next, ecx = rows
//! Out: every register preserved
//! The table is `dword_4F3FC4`, which has room for 602 rows. Four callers, the rescalers
//! and two picture functions not yet read.
//! Refuses a count of 0 (the original counts through all 32 bits) and one above the table's
//! room. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::fill_row_table", rva(0x40586A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const ROW_TABLE_RVA: usize = rva(0x4F3FC4); // dword_4F3FC4
pub const ROW_TABLE_ROOM: u32 = 602;

/// The address the table holds for a row.
pub unsafe fn row_address(row: u32) -> u32 {
    unsafe {
        (game_addr(ROW_TABLE_RVA) as *const u32)
            .add(row as usize)
            .read_unaligned()
    }
}

// ---- Rust version ----

pub unsafe fn fill_row_table(first: u32, step: u32, rows: u32) -> Result<(), BadInput> {
    if rows == 0 || rows > ROW_TABLE_ROOM {
        return Err(BadInput("a row count the row table has no room for"));
    }
    let table = game_addr(ROW_TABLE_RVA) as *mut u32;
    let mut address = first;
    for row in 0..rows as usize {
        unsafe { table.add(row).write_unaligned(address) };
        address = address.wrapping_add(step);
    }
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_row_table(r.eax, r.ebx, r.ecx) }?;
    Ok(Left::nothing())
}

fn region(rows: u32) -> [(u32, usize); 1] {
    [(
        game_addr(ROW_TABLE_RVA) as u32,
        rows.min(ROW_TABLE_ROOM) as usize * 4,
    )]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &region(r.ecx), &rust);
}

// ---- self-test: the game's own table, which is put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let whole = region(ROW_TABLE_ROOM);
    let saved = capture(&whole);
    for (first, step, rows) in [
        (0x1000u32, 4u32, 1u32),
        (0x0123_4560, 3200, 600),
        (0xFFFF_FF00, 0x80, 10),
        (0x5000, 0, 7),
        (0x9000, (-64i32) as u32, ROW_TABLE_ROOM),
    ] {
        let before = Regs {
            eax: first,
            ebx: step,
            ecx: rows,
            ..Regs::SENTINEL
        };
        check_state(
            &mut t,
            &format!("first={first:#x} step={step:#x} rows={rows}"),
            call_original,
            before,
            &|| restore(&whole, &vec![0xAA; whole[0].1]),
            &whole,
            &rust,
        );
    }
    restore(&whole, &saved);
    t.finish()
}
