//! `install::value_by_number` (0x4148B7): an item of a value (kind 3) set by a number key to
//! its least plus (most - least) x ecx / 8 (`imul`, `idiv` by 8, towards 0).
//!
//! Takes the item (+8 the value's byte, +0xC the least, +0xD the most) and the number.
//! Callers `install::key_number` (ecx the key less 2: 0 .. 8).
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments.

use crate::errors::Refused;
use crate::menu::list_handle::ListItem;

/// `number` the eighths (ecx).
pub fn value_by_number(item: ListItem, number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::items::value_by_number`.
    let value =
        rc2k_core::install::items::value_by_number(item.value_least(), item.value_most(), number)?;
    item.value().set(value);
    Ok(())
}
