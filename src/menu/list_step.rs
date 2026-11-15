//! `menu::list_step`: the highlight of a list menu (the replay viewer's, `install::run_page`'s)
//! moved by a key: 0x66 and 0x6B back and on, 0x68 and 0x69 the other way.
//! With a table of neighbours (4 bytes an item: 0x66's, 0x6B's, 0x68's, 0x69's) the item's
//! entry, unless negative; without one 0x66 and 0x6B go round the list and 0x68 and 0x69 do
//! nothing. The item goes into the item table.
//!
//! Another key does not move it. Callers: `race::replay_viewer` (two), which read whether it
//! moved.

use crate::place::BytePlace;
use rc2k_core::menu::list_menu::{MOVES, Step, step};

/// `key` moving the highlight of the list menu `list` by the neighbours table `table` (None
/// for none): whether it moved.
pub fn list_step(
    list_state: &crate::menu::list_state::ListState,
    key: u8,
    table: Option<BytePlace>,
    list: crate::list_handle::ListMenu,
) -> Result<bool, crate::errors::BadInput> {
    let count = list.count();
    if !MOVES.contains(&key) {
        return Ok(false);
    }
    let ecx = list_state.choice.get();
    let fault = crate::menu::event_flow::Fault::default();
    // Asked only with a table (`step` is told whether there is one).
    let entry_of = |k: usize| match table {
        Some(table) => fault.ok(table
            .add(ecx.wrapping_mul(4))
            .add(k as u32)
            .read(1, "a neighbours table")
            .map(|bytes| bytes[0])),
        None => 0,
    };
    let with_cl = |cl: u8| ecx & !0xFF | cl as u32;
    let stepped = step(key, ecx as u8, count, table.is_some(), entry_of);
    fault.check()?;
    let Step::Moved(cl) = stepped else {
        return Ok(false);
    };
    list_state.choice.set(with_cl(cl));
    Ok(true)
}
