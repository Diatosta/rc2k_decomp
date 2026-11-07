//! `service::clear_repair_flags` (0x470DC3): sets every entry of the flag table
//! (`dword_759B6E`) to 0, up to its -1 end mark: nothing is chosen any more.
//!
//! In:  nothing
//! Out: every register preserved
//! The first entry is cleared without being checked for the end mark, as in the original.
//! Refuses a table whose end mark is not where it belongs.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::service::state::{flags, store_flags};

// ---- Rust version ----

pub fn clear_repair_flags(menus: &Menus) -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::repairs::clear`.
    let tables = &menus.tables;
    let mut t = flags(tables);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_flags(tables, &t);
    Ok(())
}
