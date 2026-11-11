//! `strings::expand_entry` (0x401FE1): appends entry `id` of the packed string table, expanding
//! child entries recursively.
//!
//! Takes id, cursor. Returns new cursor. The original also clobbers three registers; its callers
//! save those around the call.

use crate::strings::string_table::table;

/// Whether a bad entry has been logged (once).
static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Entry `id` of the string table, given to `emit` (converted:
/// `rc2k_core::strings::table::StringTable::expand`). A bad entry, or no table yet, is logged
/// once and gives nothing (the original read wherever it pointed: fixed, `notes/bugs.md`).
pub fn expand_entry(id: u32, emit: &mut impl FnMut(&[u8])) {
    let mut out = Vec::new();
    let done = match table() {
        Some(t) => t.expand(id, &mut out).map_err(|e| e.0),
        None => Err("no string table yet"),
    };
    match done {
        Ok(()) => emit(&out),
        Err(why) => {
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                crate::framework::log(&format!(
                    "strings::expand_entry: id={id}: {why}; nothing written"
                ));
            }
        }
    }
}
