//! `strings::expand_entry` (0x401FE1): appends entry `id` of the packed string table, expanding
//! child entries recursively.
//!
//! In:  eax = id, edi = cursor
//! Out: edi = new cursor
//! The original also clobbers eax, ecx and esi; its callers save those around the call.
//! Hook removed 2026-10-21 (conversion phase 4): Rust callers call this directly; the
//! register use above is the original's, for reference.

use crate::strings::string_table::table;

// ---- Rust version ----

/// Whether a bad entry has been logged (once).
static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Entry `id` of the string table `g_stringTable` points at, given to `emit` (converted:
/// `rc2k_core::strings::table::StringTable::expand`). `blob` is that pointer, as the callers read
/// it. A bad entry or a table the core does not know is logged once and gives nothing (the
/// original read wherever it pointed: fixed, `notes/bugs.md`).
pub unsafe fn expand_entry(blob: *const u8, id: u32, emit: &mut impl FnMut(*const u8, usize)) {
    let mut out = Vec::new();
    let done = match table() {
        Some(t) if t.bytes.as_ptr() == blob => t.expand(id, &mut out).map_err(|e| e.0),
        _ => Err("the string table is not the lexicon's"),
    };
    match done {
        Ok(()) => emit(out.as_ptr(), out.len()),
        Err(why) => {
            if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                crate::framework::log(&format!(
                    "strings::expand_entry: id={id}: {why}; nothing written"
                ));
            }
        }
    }
}
