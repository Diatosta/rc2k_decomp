//! The highlight of the viewer's list moved by a key: 0x66 and 0x6B back and on, 0x68 and 0x69
//! the other way.
//! With a table of neighbours (4 bytes an item: 0x66's, 0x6B's, 0x68's, 0x69's) the item's
//! entry, unless negative; without one 0x66 and 0x6B go round the list and 0x68 and 0x69 do
//! nothing. The item goes into the item table.
//!
//! Another key does not move it.

use crate::memory::place::BytePlace;
use crate::race::viewer_list::ITEMS;
use rc2k_core::menu::list_menu::{MOVES, Step, step};

/// `key` moving the highlight of the viewer's list by the neighbours table `table` (None
/// for none): whether it moved.
pub fn list_step(
    list_state: &crate::menu::list_state::ListState,
    key: u8,
    table: Option<BytePlace>,
) -> Result<bool, crate::errors::BadInput> {
    let count = ITEMS as u8;
    if !MOVES.contains(&key) {
        return Ok(false);
    }
    let choice = list_state.choice.get();
    let fault = crate::errors::Fault::default();
    // Asked only with a table (`step` is told whether there is one).
    let entry_of = |k: usize| match table {
        Some(table) => fault.ok(table
            .add(choice.wrapping_mul(4))
            .add(k as u32)
            .read(1, "a neighbours table")
            .map(|bytes| bytes[0])),
        None => 0,
    };
    let stepped = step(key, choice as u8, count, table.is_some(), entry_of);
    fault.check()?;
    let Step::Moved(item) = stepped else {
        return Ok(false);
    };
    // Only the low byte is the item; the rest is kept.
    list_state.choice.set(choice & !0xFF | item as u32);
    Ok(true)
}
