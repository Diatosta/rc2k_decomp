//! Appends entry `id` of the packed string table, expanding child entries recursively.

use crate::strings::string_table::table;

/// Whether a bad entry has been logged (once).
static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Entry `id` of the string table, given to `emit`
/// (`rc2k_core::strings::table::StringTable::expand`). A bad entry, or no table yet, is logged once
/// and gives nothing (fixed: "Text lookup with a bad string id reads anywhere",
/// `wiki/Fixed-bugs.md`).
pub fn expand_entry(
    strings: &crate::strings::owner::Strings,
    id: u32,
    emit: &mut impl FnMut(&[u8]),
) {
    let mut out = Vec::new();
    let done = match table(strings) {
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
