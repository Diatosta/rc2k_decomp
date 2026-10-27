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
use crate::saves::best_times_store::BEST_TIMES;
use crate::system::checked_reads::CheckedReads;
use rc2k_core::saves::tables::{MARKED, TABLES};

// ---- game data ----

// g_recordTable
const RECORD_LEN: usize = 0xA4;

/// Where table `n`'s address is among the variables; its size follows.
pub fn count_stages(table: u32) -> Result<u32, BadInput> {
    let reads = CheckedReads::new();
    let answer = rc2k_core::saves::tables::count_stages(|n| {
        reads.dword(
            table.wrapping_add(n * RECORD_LEN as u32),
            "a stage record's tag",
        )
    });
    reads.finish(answer)?.map_err(BadInput::from)
}

pub fn sizes(stages: u32) -> [u32; TABLES] {
    rc2k_core::saves::tables::sizes(stages)
}

/// `saves::allocate_best_times` (0x47C4D8): once, the nine tables taken from `list`
/// (`BEST_TIMES.tables`, a block and a size each: `memory::blocks`, Rust's own since phase
/// 12; the typed ones, `saves::best_times`, keep only the size), the marked ones ending in
/// -1.
pub fn allocate_best_times(list: *mut MemoryList, records: u32) -> Result<(), Refused> {
    if BEST_TIMES.allocated.get() == 1 {
        return Ok(());
    }
    let sizes = sizes(count_stages(records)?);
    BEST_TIMES.allocated.set(1);
    for (n, size) in sizes.into_iter().enumerate() {
        let place = &BEST_TIMES.tables[n];
        place.size.set(size);
        if let Some(typed) = crate::saves::best_times::typed(n) {
            typed.take(list, size)?;
            continue;
        }
        let block = crate::memory::blocks::take(list, size)?;
        place.block.set(block);
        if MARKED.contains(&n) {
            crate::memory::blocks::At::start(block)
                .add(size.wrapping_sub(4))
                .write(u32::MAX, "a table of best times")?;
        }
    }
    Ok(())
}
