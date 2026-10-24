//! `menu::copy_standings_summary` (0x470466): 0x5A entries of standings table A copied, in
//! order, into the summary table (`dword_7556F4`, 0x50 each: +0 / +4 / +8 / +0xC from the
//! entry's +0xC / +8 / +0x10 / +0x14): with `dword_75AB40` 2 the entries with a group
//! (+0x1C not 0), else those without.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses null tables and a table A that runs out (the original walks on
//! past its end), before anything.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::menu::standings::*;

const SUMMARY_RVA: usize = rva(0x7556F4); // dword_7556F4
const MODE_RVA: usize = rva(0x75AB40); // dword_75AB40
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn copy_standings_summary() -> Result<(), Refused> {
    let (summary, first): (u32, u32) = unsafe { (game_read(SUMMARY_RVA), game_read(TABLE_A_RVA)) };
    if summary == 0 || first == 0 {
        return Err(BadInput("no standings tables").into());
    }
    let grouped = unsafe { game_read::<u32>(MODE_RVA) } == 2;
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::summary_rows`.
    let t = unsafe { table(first, A_ENTRY, all.len() as u32) };
    let Some(rows) = rc2k_core::menu::standings::summary_rows(&t, grouped) else {
        return Err(BadInput("standings table A runs out").into());
    };
    for (n, row) in rows.into_iter().enumerate() {
        let s = summary + 0x50 * n as u32;
        for (k, v) in row.into_iter().enumerate() {
            put(s + 4 * k as u32, v);
        }
    }
    Ok(())
}
