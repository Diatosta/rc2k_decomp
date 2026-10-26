//! `service::begin_suspension_stiffness_edit` (0x46B777): the item action that opens the suspension stiffness edit page, if the
//! item is chosen or its time fits (see `service/begin_edits.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! In the service area's menu table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::service::begin_edits::{SUSPENSION_STIFFNESS, edit_entry};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    edit_entry(r, SUSPENSION_STIFFNESS)
}
