//! An item of a value (kind 3) set by a number key to its least plus (most - least) x number / 8
//! (towards 0). The number is the key less 2: 0 .. 8.

use crate::errors::Refused;
use crate::memory::list_handle::ListItem;

/// `number` the eighths.
pub fn value_by_number(
    install: &'static crate::install::owner::Install,
    item: ListItem,
    number: u32,
) -> Result<(), Refused> {
    let value =
        rc2k_core::install::items::value_by_number(item.value_least(), item.value_most(), number)?;
    item.value(install).set(value);
    Ok(())
}
