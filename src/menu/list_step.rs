//! `menu::list_step` (0x414A55): the highlight of a list menu (the replay viewer's,
//! `sub_4144CF`'s) moved by a key: 0x66 and 0x6B back and on, 0x68 and 0x69 the other way.
//! With a table of neighbours (ebx, 4 bytes an item: 0x66's, 0x6B's, 0x68's, 0x69's) the
//! item's entry, unless negative; without one (ebx 0) 0x66 and 0x6B go round the list
//! (`[edi+1]` items) and 0x68 and 0x69 do nothing. The item into `dword_51D407`.
//!
//! In:  al the key, ebx the table or 0, edi the list
//! Out: moved: ZF set, carry clear, ecx the item. Not moved: ZF clear; ecx kept for
//! another key (carry: al below 0x69), else `dword_51D407` with cl the table's entry
//! (carry clear), or for 0x68 / 0x69 without a table `dword_51D407` (carry set)
//!
//! Callers: `race::replay_viewer` (two), `sub_4144CF` (one), reading ZF.

use crate::pointers::GamePtr;
use rc2k_core::menu::list_menu::{MOVES, Step, step};

/// `key` (al) moving the highlight of the list menu `list` (edi) by the neighbours table
/// `table` (ebx; None for none): whether it moved (the zero flag the original leaves; its
/// carry and ecx no caller reads).
pub fn list_step(
    key: u8,
    table: Option<GamePtr>,
    list: crate::menu::list_handle::ListMenu,
) -> Result<bool, crate::errors::BadInput> {
    // Converted: `rc2k_core::menu::list_menu::step`.
    let count = list.count();
    if !MOVES.contains(&key) {
        return Ok(false);
    }
    let pages = &crate::install::owner::install().pages;
    let ecx = pages.choice.get();
    let fault = crate::menu::event_flow::Fault::default();
    // Asked only with a table (`step` is told whether there is one).
    let entry_of = |k: usize| match table {
        Some(table) => fault.read_at::<u8>(
            table.offset(ecx.wrapping_mul(4)).offset(k as u32),
            "a neighbours table",
        ),
        None => 0,
    };
    let with_cl = |cl: u8| ecx & !0xFF | cl as u32;
    let stepped = step(key, ecx as u8, count, table.is_some(), entry_of);
    fault.check()?;
    let Step::Moved(cl) = stepped else {
        return Ok(false);
    };
    pages.choice.set(with_cl(cl));
    Ok(true)
}
