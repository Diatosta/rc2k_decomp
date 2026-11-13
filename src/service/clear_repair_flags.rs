//! Sets every entry of the flag table (`inspection_flags_a`) to 0, up to its -1 end mark:
//! nothing is chosen any more.
//!
//! The first entry is cleared without being checked for the end mark. Refuses a table whose end
//! mark is not where it belongs.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::service::state::{flags, store_flags};

pub fn clear_repair_flags(menus: &Menus) -> Result<(), BadInput> {
    let tables = &menus.tables;
    let mut t = flags(tables);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_flags(tables, &t);
    Ok(())
}
