//! `service::clear_repair_marks` (0x46BA15): sets every entry of the second flag table
//! (`dword_759BC2`) to 0, up to its -1 end mark.
//!
//! In:  nothing
//! Out: every register preserved
//! The first entry is cleared without being checked for the end mark, as in the original.
//! Refuses a table whose end mark is not where it belongs. What the marks mean has not been
//! worked out: the name is a reading.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::owner::Tables;
use crate::service::state::{marks, store_marks};

// ---- Rust version ----

pub fn clear_repair_marks(tables: &Tables) -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::repairs::clear`.
    let mut t = marks(tables);
    rc2k_core::service::repairs::clear(&mut t)?;
    store_marks(tables, &t);
    Ok(())
}
