//! `saves::allocate_best_times` (0x47C4D8): takes the memory of the best times' tables,
//! once.
//!
//! In:  nothing
//! Out: every register preserved
//! Nothing if `dword_747610` is 1; else that becomes 1 and nine blocks are taken on the
//! list `stru_7475D0`, each block's size stored before it is asked for and its address
//! after, in the pairs from `dword_747618` on: 0x1C20, 0xBB8, stages * 0x618, 0x15E0,
//! 0xDB60, 0x4C2C4, 0x3A984, 0xDB60, stages * 0x618 bytes. The sixth and the seventh get
//! -1 in their last dword. "Stages" is the number of records of the championship's table
//! (`g_recordTable`) whose tag is 0 (`saves::count_stages`, 0x47C95D, a plain function
//! here). Six of the nine are what `saves::load_best_times` reads `var\hst.dat` into.
//! Callers: `saves::load_best_times` and `sub_47C63F`. The original has a fatal error for each
//! block that cannot be taken (0x807010F1, 0x807010FB, 0x80701000 .. 0x80701005, 0x8070100A); none
//! can come, the allocator reporting its own first. Refuses a record table without its end, and
//! what the allocator refuses.

use crate::errors::{BadInput, Refused};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use rc2k_core::saves::tables::{MARKED, TABLES};

// ---- game data ----

// g_recordTable
const RECORD_LEN: usize = 0xA4;

/// Where table `n`'s address is among the variables; its size follows.
pub fn pair(vars: *mut u32, n: usize) -> *mut u32 {
    vars.wrapping_add(2 + 2 * n)
}

// ---- Rust version ----

/// `saves::count_stages` (0x47C95D): the records with tag 0, up to the table's -1 (the
/// first record is not looked at for the end).
pub unsafe fn count_stages(table: *const u8) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::saves::tables::count_stages`.
    Ok(rc2k_core::saves::tables::count_stages(|n| unsafe {
        (table.add(n as usize * RECORD_LEN) as *const u32).read_unaligned()
    })?)
}

/// The nine sizes for a table with that many stages. Converted:
/// `rc2k_core::saves::tables::sizes`.
pub fn sizes(stages: u32) -> [u32; TABLES] {
    rc2k_core::saves::tables::sizes(stages)
}

pub unsafe fn allocate_best_times(
    vars: *mut u32,
    list: *mut MemoryList,
    records: *const u8,
) -> Result<(), Refused> {
    unsafe {
        if vars.read_unaligned() == 1 {
            return Ok(());
        }
        let sizes = sizes(count_stages(records)?);
        vars.write_unaligned(1);
        for (n, size) in sizes.into_iter().enumerate() {
            let pair = pair(vars, n);
            pair.add(1).write_unaligned(size);
            let memory = allocate(list, size)?;
            pair.write_unaligned(memory as u32);
            if MARKED.contains(&n) {
                (memory.add(size as usize - 4) as *mut u32).write_unaligned(u32::MAX);
            }
        }
        Ok(())
    }
}
